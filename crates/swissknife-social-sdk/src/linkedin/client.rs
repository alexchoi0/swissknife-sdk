use reqwest::Client;

use crate::{Error, Result};

const LINKEDIN_API_BASE: &str = "https://api.linkedin.com/v2";

pub struct LinkedInClient {
    pub(crate) http: Client,
    pub(crate) access_token: String,
    pub(crate) person_urn: String,
}

impl LinkedInClient {
    pub fn new(
        access_token: impl Into<String>,
        person_urn: impl Into<String>,
    ) -> Result<Self> {
        let access_token = access_token.into();
        let person_urn = person_urn.into();

        if access_token.is_empty() || person_urn.is_empty() {
            return Err(Error::Config(
                "Access token and person URN are required".into(),
            ));
        }

        let http = Client::builder()
            .build()
            .map_err(|e| Error::Config(format!("Failed to create HTTP client: {}", e)))?;

        Ok(Self {
            http,
            access_token,
            person_urn,
        })
    }

    pub fn for_organization(
        access_token: impl Into<String>,
        org_id: impl Into<String>,
    ) -> Result<Self> {
        let org_id = org_id.into();
        Self::new(access_token, format!("urn:li:organization:{}", org_id))
    }

    pub fn for_person(
        access_token: impl Into<String>,
        person_id: impl Into<String>,
    ) -> Result<Self> {
        let person_id = person_id.into();
        Self::new(access_token, format!("urn:li:person:{}", person_id))
    }

    pub(crate) fn posts_url(&self) -> String {
        format!("{}/posts", LINKEDIN_API_BASE)
    }

    pub(crate) fn auth_header(&self) -> String {
        format!("Bearer {}", self.access_token)
    }
}
