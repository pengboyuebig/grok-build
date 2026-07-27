use std::{path::PathBuf, sync::Arc};

use tokio::sync::{Mutex, broadcast};

use crate::{domain::chat::FrontendEvent, services::live_agent::LiveAgent};

pub struct SessionService {
    agent: Mutex<Option<Arc<LiveAgent>>>,
    sessions: Mutex<Vec<String>>,
    events: broadcast::Sender<FrontendEvent>,
}

impl Default for SessionService {
    fn default() -> Self {
        let (events, _) = broadcast::channel(128);
        Self {
            agent: Mutex::new(None),
            sessions: Mutex::new(Vec::new()),
            events,
        }
    }
}

impl SessionService {
    async fn agent(&self) -> anyhow::Result<Arc<LiveAgent>> {
        let mut current = self.agent.lock().await;
        if let Some(agent) = current.as_ref() {
            return Ok(Arc::clone(agent));
        }
        let events = self.events.clone();
        let agent = LiveAgent::connect(Arc::new(move |event| {
            let _ = events.send(event);
        }))
        .await?;
        *current = Some(Arc::clone(&agent));
        Ok(agent)
    }

    pub async fn start_session(&self, cwd: PathBuf) -> anyhow::Result<String> {
        if !cwd.is_dir() {
            anyhow::bail!("workspace directory does not exist");
        }
        let session_id = self.agent().await?.start_session(cwd).await?;
        self.sessions.lock().await.push(session_id.clone());
        Ok(session_id)
    }

    pub async fn send_message(&self, session_id: String, message: String) -> anyhow::Result<()> {
        if session_id.trim().is_empty() || message.trim().is_empty() {
            anyhow::bail!("session id and message cannot be empty");
        }
        if !self.sessions.lock().await.contains(&session_id) {
            anyhow::bail!("unknown session");
        }
        self.agent().await?.send_message(session_id, message).await
    }

    pub async fn respond_to_approval(
        &self,
        approval_id: String,
        approved: bool,
    ) -> anyhow::Result<()> {
        self.agent()
            .await?
            .respond_to_approval(approval_id, approved)
            .await
    }

    pub async fn list_sessions(&self) -> Vec<String> {
        self.sessions.lock().await.clone()
    }

    pub fn subscribe_events(&self) -> broadcast::Receiver<FrontendEvent> {
        self.events.subscribe()
    }
}
