use reqwest::Client;

pub struct MailgunClient {
    api_key: String,
    domain: String,
    region: MailgunRegion,
    client: Client,
}

#[derive(Clone, Copy)]
pub enum MailgunRegion {
    US,
    EU,
}

impl MailgunClient {
    pub fn new(api_key: &str, domain: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            domain: domain.to_string(),
            region: MailgunRegion::US,
            client: Client::new(),
        }
    }

    pub fn with_region(mut self, region: MailgunRegion) -> Self {
        self.region = region;
        self
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn api_key(&self) -> &str {
        &self.api_key
    }

    pub(crate) fn domain(&self) -> &str {
        &self.domain
    }

    pub(crate) fn base_url(&self) -> &str {
        match self.region {
            MailgunRegion::US => "https://api.mailgun.net/v3",
            MailgunRegion::EU => "https://api.eu.mailgun.net/v3",
        }
    }
}
