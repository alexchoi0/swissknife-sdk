use reqwest::Client;

const ARXIV_API_URL: &str = "http://export.arxiv.org/api";

pub struct ArxivClient {
    client: Client,
}

impl ArxivClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn base_url(&self) -> &str {
        ARXIV_API_URL
    }
}

impl Default for ArxivClient {
    fn default() -> Self {
        Self::new()
    }
}
