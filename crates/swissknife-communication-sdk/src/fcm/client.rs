use reqwest::Client;

use crate::{Error, Result};

const FCM_API_BASE: &str = "https://fcm.googleapis.com/v1/projects";

pub struct FcmClient {
    pub(crate) http: Client,
    pub(crate) project_id: String,
    pub(crate) access_token: String,
}

impl FcmClient {
    pub fn new(
        project_id: impl Into<String>,
        access_token: impl Into<String>,
    ) -> Result<Self> {
        let project_id = project_id.into();
        let access_token = access_token.into();

        if project_id.is_empty() || access_token.is_empty() {
            return Err(Error::Config(
                "Project ID and access token are required".into(),
            ));
        }

        let http = Client::builder()
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http,
            project_id,
            access_token,
        })
    }

    pub(crate) fn send_url(&self) -> String {
        format!("{}/{}/messages:send", FCM_API_BASE, self.project_id)
    }

    pub(crate) fn auth_header(&self) -> String {
        format!("Bearer {}", self.access_token)
    }
}
