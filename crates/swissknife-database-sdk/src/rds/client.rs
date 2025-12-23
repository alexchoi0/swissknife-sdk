use reqwest::Client;

pub struct RdsClient {
    region: String,
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,
    resource_arn: String,
    secret_arn: String,
    database: Option<String>,
    client: Client,
}

impl RdsClient {
    pub fn new(region: &str, access_key_id: &str, secret_access_key: &str, resource_arn: &str, secret_arn: &str) -> Self {
        Self {
            region: region.to_string(),
            access_key_id: access_key_id.to_string(),
            secret_access_key: secret_access_key.to_string(),
            session_token: None,
            resource_arn: resource_arn.to_string(),
            secret_arn: secret_arn.to_string(),
            database: None,
            client: Client::new(),
        }
    }

    pub fn with_session_token(mut self, token: &str) -> Self {
        self.session_token = Some(token.to_string());
        self
    }

    pub fn with_database(mut self, database: &str) -> Self {
        self.database = Some(database.to_string());
        self
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn endpoint(&self) -> String {
        format!("https://rds-data.{}.amazonaws.com", self.region)
    }

    pub(crate) fn region(&self) -> &str {
        &self.region
    }

    pub(crate) fn resource_arn(&self) -> &str {
        &self.resource_arn
    }

    pub(crate) fn secret_arn(&self) -> &str {
        &self.secret_arn
    }

    pub(crate) fn database(&self) -> Option<&str> {
        self.database.as_deref()
    }

    pub(crate) fn access_key_id(&self) -> &str {
        &self.access_key_id
    }

    pub(crate) fn secret_access_key(&self) -> &str {
        &self.secret_access_key
    }

    pub(crate) fn session_token(&self) -> Option<&str> {
        self.session_token.as_deref()
    }
}
