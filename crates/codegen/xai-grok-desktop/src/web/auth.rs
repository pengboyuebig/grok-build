use std::sync::atomic::{AtomicU64, Ordering};

static TOKEN_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone)]
pub struct LocalAuth {
    token: String,
    origin: String,
}

impl LocalAuth {
    pub fn new(port: u16) -> Self {
        let counter = TOKEN_COUNTER.fetch_add(1, Ordering::Relaxed);
        Self::new_for_test(format!("{counter:032x}"), port)
    }

    pub fn new_for_test(token: impl Into<String>, port: u16) -> Self {
        Self {
            token: token.into(),
            origin: format!("http://127.0.0.1:{port}"),
        }
    }

    pub fn token(&self) -> &str {
        &self.token
    }

    pub fn authorizes(&self, token: Option<&str>, origin: Option<&str>) -> bool {
        token == Some(self.token.as_str()) && origin == Some(self.origin.as_str())
    }
}
