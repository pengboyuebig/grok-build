#[derive(Debug, Clone)]
pub struct LocalAuth {
    token: String,
    origin: String,
}

impl LocalAuth {
    #[cfg(feature = "web-runtime")]
    pub fn new(port: u16) -> Self {
        use rand::{Rng, distr::Alphanumeric};

        let token = rand::rng()
            .sample_iter(Alphanumeric)
            .take(43)
            .map(char::from)
            .collect::<String>();
        Self::new_for_test(token, port)
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

    pub fn origin(&self) -> &str {
        &self.origin
    }

    pub fn authorizes(&self, token: Option<&str>, origin: Option<&str>) -> bool {
        token == Some(self.token.as_str()) && origin == Some(self.origin.as_str())
    }
}
