use std::path::PathBuf;

use xai_grok_desktop::web::LocalWebServer;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let assets = std::env::var_os("GROK_WEB_ASSETS")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("crates/codegen/xai-grok-desktop/ui/dist"));
    if !assets.join("index.html").is_file() {
        anyhow::bail!(
            "web assets not found at {}; run `npm --prefix crates/codegen/xai-grok-desktop/ui run build` first",
            assets.display()
        );
    }
    let server = LocalWebServer::bind().await?;
    println!("Open {}", server.url());
    server.serve(assets).await
}
