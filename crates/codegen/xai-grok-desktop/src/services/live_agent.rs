#![cfg(feature = "agent-runtime")]

use std::{
    collections::HashMap,
    path::PathBuf,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    thread,
};

use agent_client_protocol as acp;
use tokio::sync::{Mutex, oneshot};
use tokio_util::sync::CancellationToken;
use xai_acp_lib::{
    AcpAgentTx, AcpClientChannel, AcpClientMessage, AcpGatewayReceiver, AcpGatewaySender,
    AcpResult, acp_channels, acp_send,
};
use xai_grok_shell::{
    agent::{MvpAgent, config::Config as AgentConfig, models::RefreshStrategy},
    auth::AuthManager,
    util::grok_home::grok_home,
};

use crate::domain::chat::{FrontendEvent, FrontendEventKind};

type PendingApproval = (
    Vec<acp::PermissionOption>,
    oneshot::Sender<AcpResult<acp::RequestPermissionResponse>>,
);
type EventSink = Arc<dyn Fn(FrontendEvent) + Send + Sync>;

/// A live ACP client shared by the desktop command handlers.
pub struct LiveAgent {
    tx: AcpAgentTx,
    emit_event: EventSink,
    pending_approvals: Arc<Mutex<HashMap<String, PendingApproval>>>,
    approval_counter: AtomicU64,
    _cancel: CancellationToken,
}

impl LiveAgent {
    pub async fn connect(emit_event: EventSink) -> anyhow::Result<Arc<Self>> {
        let raw_config = xai_grok_shell::config::load_effective_config()
            .map_err(|error| anyhow::anyhow!("failed to load Grok config: {error}"))?;
        let agent_config = AgentConfig::new_from_toml_cfg(&raw_config)
            .map_err(|error| anyhow::anyhow!("failed to create agent config: {error}"))?;
        let cancel = CancellationToken::new();
        let channel = spawn_shell(agent_config, &cancel).await?;
        let tx = channel.tx;

        let initialize: acp::InitializeResponse = acp_send(
            acp::InitializeRequest::new(acp::ProtocolVersion::V1).client_capabilities(
                acp::ClientCapabilities::new().fs(acp::FileSystemCapabilities::new()),
            ),
            &tx,
        )
        .await?;
        let auth_method = initialize
            .auth_methods
            .first()
            .ok_or_else(|| anyhow::anyhow!("agent did not advertise an authentication method"))?
            .id()
            .clone();
        let _: acp::AuthenticateResponse =
            acp_send(acp::AuthenticateRequest::new(auth_method), &tx).await?;

        let agent = Arc::new(Self {
            tx,
            emit_event,
            pending_approvals: Arc::new(Mutex::new(HashMap::new())),
            approval_counter: AtomicU64::new(1),
            _cancel: cancel,
        });
        Self::run_event_loop(Arc::clone(&agent), channel.rx);
        Ok(agent)
    }

    pub async fn start_session(&self, cwd: PathBuf) -> anyhow::Result<String> {
        let response: acp::NewSessionResponse =
            acp_send(acp::NewSessionRequest::new(cwd), &self.tx).await?;
        let session_id = response.session_id.0.to_string();
        self.emit(
            FrontendEventKind::SessionChanged,
            Some(session_id.clone()),
            None,
        );
        Ok(session_id)
    }

    pub async fn send_message(&self, session_id: String, message: String) -> anyhow::Result<()> {
        let request = acp::PromptRequest::new(
            acp::SessionId::new(session_id),
            vec![acp::ContentBlock::Text(acp::TextContent::new(message))],
        );
        let tx = self.tx.clone();
        let emit_event = Arc::clone(&self.emit_event);
        tokio::spawn(async move {
            if let Err(error) = acp_send::<_, _>(request, &tx).await {
                emit_event(FrontendEvent {
                    kind: FrontendEventKind::Error,
                    text: Some(error.to_string()),
                    approval_id: None,
                    approved: false,
                });
            }
        });
        Ok(())
    }

    pub async fn respond_to_approval(
        &self,
        approval_id: String,
        approved: bool,
    ) -> anyhow::Result<()> {
        let (options, sender) = self
            .pending_approvals
            .lock()
            .await
            .remove(&approval_id)
            .ok_or_else(|| anyhow::anyhow!("approval request is no longer pending"))?;
        let outcome = if approved {
            options
                .iter()
                .find(|option| option.kind == acp::PermissionOptionKind::AllowOnce)
                .map(|option| {
                    acp::RequestPermissionOutcome::Selected(acp::SelectedPermissionOutcome::new(
                        option.option_id.clone(),
                    ))
                })
                .unwrap_or(acp::RequestPermissionOutcome::Cancelled)
        } else {
            acp::RequestPermissionOutcome::Cancelled
        };
        sender
            .send(Ok(acp::RequestPermissionResponse::new(outcome)))
            .map_err(|_| anyhow::anyhow!("agent cancelled the approval request"))?;
        Ok(())
    }

    fn run_event_loop(agent: Arc<Self>, mut rx: xai_acp_lib::AcpClientRx) {
        tokio::spawn(async move {
            while let Some(message) = rx.recv().await {
                match message {
                    AcpClientMessage::SessionNotification(notification) => {
                        if let acp::SessionUpdate::AgentMessageChunk(chunk) =
                            &notification.request.update
                            && let acp::ContentBlock::Text(text) = &chunk.content
                            && !text.text.is_empty()
                        {
                            agent.emit(
                                FrontendEventKind::AssistantDelta,
                                Some(text.text.clone()),
                                None,
                            );
                        }
                        let _ = notification.response_tx.send(Ok(()));
                    }
                    AcpClientMessage::RequestPermission(request) => {
                        let id = format!(
                            "approval-{}",
                            agent.approval_counter.fetch_add(1, Ordering::Relaxed)
                        );
                        agent.pending_approvals.lock().await.insert(
                            id.clone(),
                            (request.request.options.clone(), request.response_tx),
                        );
                        agent.emit(
                            FrontendEventKind::ApprovalRequested,
                            Some("Tool requests permission".to_string()),
                            Some(id),
                        );
                    }
                    AcpClientMessage::ExtNotification(notification) => {
                        let _ = notification.response_tx.send(Ok(()));
                    }
                    AcpClientMessage::ReadTextFile(request) => {
                        let _ = request
                            .response_tx
                            .send(Err(acp::Error::method_not_found()));
                    }
                    AcpClientMessage::WriteTextFile(request) => {
                        let _ = request
                            .response_tx
                            .send(Err(acp::Error::method_not_found()));
                    }
                    AcpClientMessage::CreateTerminal(request) => {
                        let _ = request
                            .response_tx
                            .send(Err(acp::Error::method_not_found()));
                    }
                    AcpClientMessage::TerminalOutput(request) => {
                        let _ = request
                            .response_tx
                            .send(Err(acp::Error::method_not_found()));
                    }
                    AcpClientMessage::ReleaseTerminal(request) => {
                        let _ = request
                            .response_tx
                            .send(Err(acp::Error::method_not_found()));
                    }
                    AcpClientMessage::WaitForTerminalExit(request) => {
                        let _ = request
                            .response_tx
                            .send(Err(acp::Error::method_not_found()));
                    }
                    AcpClientMessage::KillTerminalCommand(request) => {
                        let _ = request
                            .response_tx
                            .send(Err(acp::Error::method_not_found()));
                    }
                    AcpClientMessage::ExtMethod(request) => {
                        let _ = request
                            .response_tx
                            .send(Err(acp::Error::method_not_found()));
                    }
                }
            }
        });
    }

    fn emit(&self, kind: FrontendEventKind, text: Option<String>, approval_id: Option<String>) {
        (self.emit_event)(FrontendEvent {
            kind,
            text,
            approval_id,
            approved: false,
        });
    }
}

async fn spawn_shell(
    agent_config: AgentConfig,
    cancel: &CancellationToken,
) -> anyhow::Result<AcpClientChannel> {
    let auth_manager = Arc::new(AuthManager::new(
        &grok_home(),
        agent_config.grok_com_config.clone(),
    ));
    auth_manager.configure_refresher(
        agent_config.grok_com_config.auth_provider_command.clone(),
        None,
    );
    let (agent_config, models) =
        xai_grok_shell::agent::init::bootstrap(&agent_config, &auth_manager, None)
            .map_err(|error| anyhow::anyhow!(error))?;
    models.list_models(RefreshStrategy::OnlineIfUncached).await;
    let (client, agent_channel) = acp_channels();
    let agent_cancel = cancel.child_token();
    thread::Builder::new()
        .name("grok-desktop-agent".to_string())
        .spawn(move || -> anyhow::Result<()> {
            let runtime = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()?;
            let local = tokio::task::LocalSet::new();
            local.block_on(&runtime, async move {
                let gateway = AcpGatewaySender::new(agent_channel.tx.clone());
                let agent = Rc::new(MvpAgent::with_models(
                    gateway,
                    &agent_config,
                    auth_manager,
                    models,
                ));
                tokio::task::spawn_local(AcpGatewayReceiver::new(agent_channel.rx, agent).run());
                agent_cancel.cancelled().await;
                Ok(())
            })
        })?;
    Ok(client)
}
