use crate::{Error, Result, File, Folder, FileStorageProvider};
use crate::microsoft::MicrosoftClient;
use async_trait::async_trait;
use serde::Deserialize;

impl MicrosoftClient {
    pub async fn list_drive_items(&self, folder_id: Option<&str>) -> Result<DriveItemsResponse> {
        let path = match folder_id {
            Some(id) => format!("{}/me/drive/items/{}/children", self.base_url(), id),
            None => format!("{}/me/drive/root/children", self.base_url()),
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

        let items: DriveItemsResponse = response.json().await?;
        Ok(items)
    }

    pub async fn get_drive_item(&self, item_id: &str) -> Result<DriveItem> {
        let response = self.client()
            .get(format!("{}/me/drive/items/{}", self.base_url(), item_id))
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

        let item: DriveItem = response.json().await?;
        Ok(item)
    }

    pub async fn download_drive_item(&self, item_id: &str) -> Result<Vec<u8>> {
        let response = self.client()
            .get(format!("{}/me/drive/items/{}/content", self.base_url(), item_id))
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

        let bytes = response.bytes().await?.to_vec();
        Ok(bytes)
    }

    pub async fn upload_drive_item(&self, folder_id: Option<&str>, name: &str, content: &[u8]) -> Result<DriveItem> {
        let path = match folder_id {
            Some(id) => format!("{}/me/drive/items/{}:/{}:/content", self.base_url(), id, urlencoding::encode(name)),
            None => format!("{}/me/drive/root:/{}:/content", self.base_url(), urlencoding::encode(name)),
        };

        let response = self.client()
            .put(&path)
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("Content-Type", "application/octet-stream")
            .body(content.to_vec())
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

        let item: DriveItem = response.json().await?;
        Ok(item)
    }

    pub async fn create_onedrive_folder(&self, parent_id: Option<&str>, name: &str) -> Result<DriveItem> {
        let path = match parent_id {
            Some(id) => format!("{}/me/drive/items/{}/children", self.base_url(), id),
            None => format!("{}/me/drive/root/children", self.base_url()),
        };

        let body = serde_json::json!({
            "name": name,
            "folder": {},
            "@microsoft.graph.conflictBehavior": "rename"
        });

        let response = self.client()
            .post(&path)
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

        let item: DriveItem = response.json().await?;
        Ok(item)
    }

    pub async fn delete_drive_item(&self, item_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/me/drive/items/{}", self.base_url(), item_id))
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

    pub async fn search_drive(&self, query: &str) -> Result<DriveItemsResponse> {
        let response = self.client()
            .get(format!("{}/me/drive/root/search(q='{}')", self.base_url(), urlencoding::encode(query)))
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

        let items: DriveItemsResponse = response.json().await?;
        Ok(items)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct DriveItemsResponse {
    pub value: Vec<DriveItem>,
    #[serde(rename = "@odata.nextLink")]
    pub next_link: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DriveItem {
    pub id: String,
    pub name: String,
    pub size: Option<i64>,
    #[serde(rename = "webUrl")]
    pub web_url: Option<String>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
    #[serde(rename = "lastModifiedDateTime")]
    pub last_modified_date_time: Option<String>,
    pub file: Option<FileInfo>,
    pub folder: Option<FolderInfo>,
    #[serde(rename = "parentReference")]
    pub parent_reference: Option<ParentReference>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    #[serde(rename = "mimeType")]
    pub mime_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FolderInfo {
    #[serde(rename = "childCount")]
    pub child_count: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParentReference {
    pub id: Option<String>,
    pub path: Option<String>,
}

pub struct OneDriveProvider {
    client: MicrosoftClient,
}

impl OneDriveProvider {
    pub fn new(access_token: &str) -> Self {
        Self {
            client: MicrosoftClient::new(access_token),
        }
    }
}

#[async_trait]
impl FileStorageProvider for OneDriveProvider {
    async fn list_files(&self, folder_id: Option<&str>) -> Result<Vec<File>> {
        let response = self.client.list_drive_items(folder_id).await?;

        let files = response.value.into_iter()
            .filter(|item| item.file.is_some())
            .map(|item| File {
                id: item.id,
                name: item.name,
                mime_type: item.file.and_then(|f| f.mime_type).unwrap_or_default(),
                size: item.size.map(|s| s as u64),
                url: item.web_url,
                parent_id: item.parent_reference.and_then(|p| p.id),
                created_at: item.created_date_time
                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                updated_at: item.last_modified_date_time
                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
            })
            .collect();

        Ok(files)
    }

    async fn get_file(&self, file_id: &str) -> Result<File> {
        let item = self.client.get_drive_item(file_id).await?;

        Ok(File {
            id: item.id,
            name: item.name,
            mime_type: item.file.and_then(|f| f.mime_type).unwrap_or_default(),
            size: item.size.map(|s| s as u64),
            url: item.web_url,
            parent_id: item.parent_reference.and_then(|p| p.id),
            created_at: item.created_date_time
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: item.last_modified_date_time
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        })
    }

    async fn download_file(&self, file_id: &str) -> Result<Vec<u8>> {
        self.client.download_drive_item(file_id).await
    }

    async fn upload_file(&self, folder_id: Option<&str>, name: &str, content: &[u8], _mime_type: &str) -> Result<File> {
        let item = self.client.upload_drive_item(folder_id, name, content).await?;

        Ok(File {
            id: item.id,
            name: item.name,
            mime_type: item.file.and_then(|f| f.mime_type).unwrap_or_default(),
            size: item.size.map(|s| s as u64),
            url: item.web_url,
            parent_id: item.parent_reference.and_then(|p| p.id),
            created_at: item.created_date_time
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: item.last_modified_date_time
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
        })
    }

    async fn delete_file(&self, file_id: &str) -> Result<()> {
        self.client.delete_drive_item(file_id).await
    }

    async fn create_folder(&self, parent_id: Option<&str>, name: &str) -> Result<Folder> {
        let item = self.client.create_onedrive_folder(parent_id, name).await?;

        Ok(Folder {
            id: item.id,
            name: item.name,
            parent_id: item.parent_reference.and_then(|p| p.id),
            url: item.web_url,
        })
    }
}
