use reqwest::Client;

pub struct SqsClient {
    access_key_id: String,
    secret_access_key: String,
    region: String,
    client: Client,
}

impl SqsClient {
    pub fn new(access_key_id: &str, secret_access_key: &str, region: &str) -> Self {
        Self {
            access_key_id: access_key_id.to_string(),
            secret_access_key: secret_access_key.to_string(),
            region: region.to_string(),
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn access_key_id(&self) -> &str {
        &self.access_key_id
    }

    pub(crate) fn secret_access_key(&self) -> &str {
        &self.secret_access_key
    }

    pub(crate) fn region(&self) -> &str {
        &self.region
    }

    pub(crate) fn endpoint(&self) -> String {
        format!("https://sqs.{}.amazonaws.com", self.region)
    }
}
