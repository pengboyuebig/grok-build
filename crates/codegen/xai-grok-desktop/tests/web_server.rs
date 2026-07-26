#![cfg(feature = "web-runtime")]

use std::net::{IpAddr, Ipv4Addr};

use xai_grok_desktop::web::LocalWebServer;

#[tokio::test]
async fn binds_only_to_loopback_and_includes_a_fragment_token() {
    let server = LocalWebServer::bind().await.unwrap();

    assert_eq!(server.address().ip(), IpAddr::V4(Ipv4Addr::LOCALHOST));
    assert!(server.url().contains("#token="));
}
