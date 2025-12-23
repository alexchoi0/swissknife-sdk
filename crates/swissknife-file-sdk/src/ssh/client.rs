use reqwest::Client;

pub struct SshClient {
    gateway_url: String,
    api_key: String,
    host: String,
    port: u16,
    username: String,
    auth: SshAuth,
    client: Client,
}

pub enum SshAuth {
    Password(String),
    PrivateKey { key: String, passphrase: Option<String> },
}

impl SshClient {
    pub fn with_password(
        gateway_url: &str,
        api_key: &str,
        host: &str,
        port: u16,
        username: &str,
        password: &str,
    ) -> Self {
        Self {
            gateway_url: gateway_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            host: host.to_string(),
            port,
            username: username.to_string(),
            auth: SshAuth::Password(password.to_string()),
            client: Client::new(),
        }
    }

    pub fn with_private_key(
        gateway_url: &str,
        api_key: &str,
        host: &str,
        port: u16,
        username: &str,
        private_key: &str,
        passphrase: Option<&str>,
    ) -> Self {
        Self {
            gateway_url: gateway_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            host: host.to_string(),
            port,
            username: username.to_string(),
            auth: SshAuth::PrivateKey {
                key: private_key.to_string(),
                passphrase: passphrase.map(|s| s.to_string()),
            },
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn gateway_url(&self) -> &str {
        &self.gateway_url
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.api_key
    }

    pub(crate) fn connection_info(&self) -> serde_json::Value {
        let mut info = serde_json::json!({
            "host": self.host,
            "port": self.port,
            "username": self.username,
        });

        match &self.auth {
            SshAuth::Password(password) => {
                info["password"] = serde_json::Value::String(password.clone());
            }
            SshAuth::PrivateKey { key, passphrase } => {
                info["privateKey"] = serde_json::Value::String(key.clone());
                if let Some(pass) = passphrase {
                    info["passphrase"] = serde_json::Value::String(pass.clone());
                }
            }
        }

        info
    }
}
