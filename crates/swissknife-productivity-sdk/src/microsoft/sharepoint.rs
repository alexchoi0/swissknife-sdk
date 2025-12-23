use crate::{Error, Result};
use crate::microsoft::MicrosoftClient;
use serde::Deserialize;

impl MicrosoftClient {
    pub async fn list_sharepoint_sites(&self) -> Result<SitesResponse> {
        let response = self.client()
            .get(format!("{}/sites?search=*", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let sites: SitesResponse = response.json().await?;
        Ok(sites)
    }

    pub async fn get_sharepoint_site(&self, site_id: &str) -> Result<SharePointSite> {
        let response = self.client()
            .get(format!("{}/sites/{}", self.base_url(), site_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let site: SharePointSite = response.json().await?;
        Ok(site)
    }

    pub async fn get_site_by_path(&self, hostname: &str, path: &str) -> Result<SharePointSite> {
        let response = self.client()
            .get(format!("{}/sites/{}:{}", self.base_url(), hostname, path))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let site: SharePointSite = response.json().await?;
        Ok(site)
    }

    pub async fn list_site_drives(&self, site_id: &str) -> Result<DrivesResponse> {
        let response = self.client()
            .get(format!("{}/sites/{}/drives", self.base_url(), site_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let drives: DrivesResponse = response.json().await?;
        Ok(drives)
    }

    pub async fn list_site_lists(&self, site_id: &str) -> Result<ListsResponse> {
        let response = self.client()
            .get(format!("{}/sites/{}/lists", self.base_url(), site_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let lists: ListsResponse = response.json().await?;
        Ok(lists)
    }

    pub async fn get_list_items(&self, site_id: &str, list_id: &str) -> Result<ListItemsResponse> {
        let response = self.client()
            .get(format!("{}/sites/{}/lists/{}/items?expand=fields", self.base_url(), site_id, list_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let items: ListItemsResponse = response.json().await?;
        Ok(items)
    }

    pub async fn create_list_item(&self, site_id: &str, list_id: &str, fields: serde_json::Value) -> Result<ListItem> {
        let body = serde_json::json!({
            "fields": fields
        });

        let response = self.client()
            .post(format!("{}/sites/{}/lists/{}/items", self.base_url(), site_id, list_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&body)
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

        let item: ListItem = response.json().await?;
        Ok(item)
    }

    pub async fn update_list_item(&self, site_id: &str, list_id: &str, item_id: &str, fields: serde_json::Value) -> Result<ListItem> {
        let response = self.client()
            .patch(format!("{}/sites/{}/lists/{}/items/{}/fields", self.base_url(), site_id, list_id, item_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&fields)
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

        let item: ListItem = response.json().await?;
        Ok(item)
    }

    pub async fn delete_list_item(&self, site_id: &str, list_id: &str, item_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/sites/{}/lists/{}/items/{}", self.base_url(), site_id, list_id, item_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

    pub async fn list_drive_items_sharepoint(&self, site_id: &str, drive_id: &str, folder_id: Option<&str>) -> Result<super::onedrive::DriveItemsResponse> {
        let path = match folder_id {
            Some(id) => format!("{}/sites/{}/drives/{}/items/{}/children", self.base_url(), site_id, drive_id, id),
            None => format!("{}/sites/{}/drives/{}/root/children", self.base_url(), site_id, drive_id),
        };

        let response = self.client()
            .get(&path)
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let items: super::onedrive::DriveItemsResponse = response.json().await?;
        Ok(items)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SitesResponse {
    pub value: Vec<SharePointSite>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SharePointSite {
    pub id: String,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "webUrl")]
    pub web_url: Option<String>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
    #[serde(rename = "lastModifiedDateTime")]
    pub last_modified_date_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DrivesResponse {
    pub value: Vec<SharePointDrive>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SharePointDrive {
    pub id: String,
    pub name: String,
    #[serde(rename = "driveType")]
    pub drive_type: Option<String>,
    #[serde(rename = "webUrl")]
    pub web_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListsResponse {
    pub value: Vec<SharePointList>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SharePointList {
    pub id: String,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "webUrl")]
    pub web_url: Option<String>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
    #[serde(rename = "lastModifiedDateTime")]
    pub last_modified_date_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListItemsResponse {
    pub value: Vec<ListItem>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListItem {
    pub id: String,
    pub fields: Option<serde_json::Value>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
    #[serde(rename = "lastModifiedDateTime")]
    pub last_modified_date_time: Option<String>,
    #[serde(rename = "webUrl")]
    pub web_url: Option<String>,
}
