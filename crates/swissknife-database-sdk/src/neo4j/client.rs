use reqwest::Client;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

pub struct Neo4jClient {
    uri: String,
    username: String,
    password: String,
    database: String,
    client: Client,
}

impl Neo4jClient {
    pub fn new(uri: &str, username: &str, password: &str) -> Self {
        Self {
            uri: uri.to_string(),
            username: username.to_string(),
            password: password.to_string(),
            database: "neo4j".to_string(),
            client: Client::new(),
        }
    }

    pub fn with_database(mut self, database: &str) -> Self {
        self.database = database.to_string();
        self
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn base_url(&self) -> &str {
        &self.uri
    }

    pub(crate) fn database(&self) -> &str {
        &self.database
    }

    pub(crate) fn auth_header(&self) -> String {
        let credentials = format!("{}:{}", self.username, self.password);
        format!("Basic {}", BASE64.encode(credentials.as_bytes()))
    }
}
