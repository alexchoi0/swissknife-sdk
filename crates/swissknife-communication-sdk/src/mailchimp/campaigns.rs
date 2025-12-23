use crate::{Error, Result};
use crate::mailchimp::MailchimpClient;
use serde::{Deserialize, Serialize};

impl MailchimpClient {
    pub async fn list_campaigns(&self, options: ListCampaignsOptions) -> Result<CampaignsResponse> {
        let mut params = vec![];
        if let Some(count) = options.count {
            params.push(("count", count.to_string()));
        }
        if let Some(offset) = options.offset {
            params.push(("offset", offset.to_string()));
        }
        if let Some(status) = options.status {
            params.push(("status", status));
        }

        let response = self.client()
            .get(format!("{}/campaigns", self.base_url()))
            .basic_auth("anystring", Some(self.api_key()))
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let campaigns: CampaignsResponse = response.json().await?;
        Ok(campaigns)
    }

    pub async fn get_campaign(&self, campaign_id: &str) -> Result<Campaign> {
        let response = self.client()
            .get(format!("{}/campaigns/{}", self.base_url(), campaign_id))
            .basic_auth("anystring", Some(self.api_key()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let campaign: Campaign = response.json().await?;
        Ok(campaign)
    }

    pub async fn create_campaign(&self, campaign: CreateCampaign) -> Result<Campaign> {
        let response = self.client()
            .post(format!("{}/campaigns", self.base_url()))
            .basic_auth("anystring", Some(self.api_key()))
            .json(&campaign)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let created: Campaign = response.json().await?;
        Ok(created)
    }

    pub async fn send_campaign(&self, campaign_id: &str) -> Result<()> {
        let response = self.client()
            .post(format!("{}/campaigns/{}/actions/send", self.base_url(), campaign_id))
            .basic_auth("anystring", Some(self.api_key()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        Ok(())
    }

    pub async fn list_audiences(&self) -> Result<AudiencesResponse> {
        let response = self.client()
            .get(format!("{}/lists", self.base_url()))
            .basic_auth("anystring", Some(self.api_key()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let audiences: AudiencesResponse = response.json().await?;
        Ok(audiences)
    }

    pub async fn add_member(&self, list_id: &str, member: AddMember) -> Result<Member> {
        let response = self.client()
            .post(format!("{}/lists/{}/members", self.base_url(), list_id))
            .basic_auth("anystring", Some(self.api_key()))
            .json(&member)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let created: Member = response.json().await?;
        Ok(created)
    }
}

#[derive(Default)]
pub struct ListCampaignsOptions {
    pub count: Option<u32>,
    pub offset: Option<u32>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CampaignsResponse {
    pub campaigns: Vec<Campaign>,
    pub total_items: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Campaign {
    pub id: String,
    pub web_id: Option<u64>,
    #[serde(rename = "type")]
    pub campaign_type: Option<String>,
    pub status: Option<String>,
    pub settings: Option<CampaignSettings>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CampaignSettings {
    pub subject_line: Option<String>,
    pub title: Option<String>,
    pub from_name: Option<String>,
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateCampaign {
    #[serde(rename = "type")]
    pub campaign_type: String,
    pub recipients: CampaignRecipients,
    pub settings: CreateCampaignSettings,
}

#[derive(Debug, Clone, Serialize)]
pub struct CampaignRecipients {
    pub list_id: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateCampaignSettings {
    pub subject_line: String,
    pub title: String,
    pub from_name: String,
    pub reply_to: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AudiencesResponse {
    pub lists: Vec<Audience>,
    pub total_items: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Audience {
    pub id: String,
    pub name: String,
    pub member_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AddMember {
    pub email_address: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_fields: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Member {
    pub id: String,
    pub email_address: String,
    pub status: String,
}
