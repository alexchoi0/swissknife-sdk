use reqwest::Client;

pub struct MailchimpClient {
    api_key: String,
    server: String,
    client: Client,
}

impl MailchimpClient {
    pub fn new(api_key: &str) -> Self {
        let server = api_key.split('-').last().unwrap_or("us1").to_string();
        Self {
            api_key: api_key.to_string(),
            server,
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.api_key
    }

    pub(crate) fn base_url(&self) -> String {
        format!("https://{}.api.mailchimp.com/3.0", self.server)
    }
}
