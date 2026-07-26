pub mod auth;
#[cfg(feature = "web-runtime")]
pub mod routes;

#[cfg(feature = "web-runtime")]
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::PathBuf,
};

#[cfg(feature = "web-runtime")]
use anyhow::Context;
#[cfg(feature = "web-runtime")]
use tokio::net::TcpListener;
#[cfg(feature = "web-runtime")]
use tower_http::services::ServeDir;

#[cfg(feature = "web-runtime")]
pub struct LocalWebServer {
    listener: TcpListener,
    auth: auth::LocalAuth,
}

#[cfg(feature = "web-runtime")]
impl LocalWebServer {
    pub async fn bind() -> anyhow::Result<Self> {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).await?;
        let port = listener.local_addr()?.port();
        Ok(Self {
            listener,
            auth: auth::LocalAuth::new(port),
        })
    }

    pub fn address(&self) -> SocketAddr {
        self.listener.local_addr().expect("bound listener")
    }

    pub fn url(&self) -> String {
        format!("http://{}#token={}", self.address(), self.auth.token())
    }

    pub async fn serve(self, assets: PathBuf) -> anyhow::Result<()> {
        let app = routes::router(self.auth)
            .fallback_service(ServeDir::new(assets).append_index_html_on_directories(true));
        axum::serve(self.listener, app)
            .await
            .context("local browser server failed")
    }
}
