use rand::{distributions::Alphanumeric, Rng};
use serde::Serialize;
use std::{
    net::TcpListener,
    path::PathBuf,
    process::{Child, Command},
};

#[derive(Default)]
pub struct AgentRuntime {
    child: Option<Child>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeConnection {
    pub ws_url: String,
    pub workspace_path: String,
}

impl AgentRuntime {
    pub fn start(
        &mut self,
        binary: PathBuf,
        workspace: PathBuf,
    ) -> Result<RuntimeConnection, String> {
        self.stop();
        if !binary.is_file() {
            return Err(format!("未找到内置智能体程序：{}", binary.display()));
        }
        let listener =
            TcpListener::bind("127.0.0.1:0").map_err(|e| format!("无法分配本地端口：{e}"))?;
        let port = listener.local_addr().map_err(|e| e.to_string())?.port();
        drop(listener);
        let secret: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(48)
            .map(char::from)
            .collect();
        let child = Command::new(binary)
            .args([
                "agent",
                "serve",
                "--bind",
                &format!("127.0.0.1:{port}"),
                "--secret",
                &secret,
            ])
            .current_dir(&workspace)
            .spawn()
            .map_err(|e| format!("无法启动智能体：{e}"))?;
        self.child = Some(child);
        Ok(RuntimeConnection {
            ws_url: format!("ws://127.0.0.1:{port}/ws?server-key={secret}"),
            workspace_path: workspace.display().to_string(),
        })
    }
    pub fn stop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

impl Drop for AgentRuntime {
    fn drop(&mut self) {
        self.stop();
    }
}
