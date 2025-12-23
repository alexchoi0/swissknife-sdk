use crate::{Error, Result, File, Folder, FileStorageProvider};
use crate::google::GoogleClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const DRIVE_URL: &str = "https://www.googleapis.com/drive/v3";
const UPLOAD_URL: &str = "https://www.googleapis.com/upload/drive/v3";

impl GoogleClient {
    pub async fn list_drive_files(&self, folder_id: Option<&str>, page_size: Option<u32>) -> Result<DriveFilesResponse> {
        let mut query_parts = vec!["trashed = false".to_string()];
        if let Some(fid) = folder_id {
            query_parts.push(format!("'{}' in parents", fid));
        }

        let response = self.client()
            .get(format!("{}/files", DRIVE_URL))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[
                ("q", query_parts.join(" and ")),
                ("pageSize", page_size.unwrap_or(100).to_string()),
                ("fields", "files(id,name,mimeType,size,webViewLink,parents,createdTime,modifiedTime)".to_string()),
            ])
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

        let files_response: DriveFilesResponse = response.json().await?;
        Ok(files_response)
    }

    pub async fn get_drive_file(&self, file_id: &str) -> Result<DriveFile> {
        let response = self.client()
            .get(format!("{}/files/{}", DRIVE_URL, file_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[("fields", "id,name,mimeType,size,webViewLink,parents,createdTime,modifiedTime")])
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

        let file: DriveFile = response.json().await?;
        Ok(file)
    }

    pub async fn download_drive_file(&self, file_id: &str) -> Result<Vec<u8>> {
        let response = self.client()
            .get(format!("{}/files/{}", DRIVE_URL, file_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[("alt", "media")])
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

    pub async fn upload_drive_file(&self, folder_id: Option<&str>, name: &str, content: &[u8], mime_type: &str) -> Result<DriveFile> {
        let metadata = serde_json::json!({
            "name": name,
            "parents": folder_id.map(|f| vec![f]).unwrap_or_default()
        });

        let boundary = "swissknife_boundary";
        let mut body = Vec::new();

        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
        body.extend_from_slice(serde_json::to_string(&metadata)?.as_bytes());
        body.extend_from_slice(b"\r\n");

        body.extend_from_slice(format!("--{}\r\n", boundary).as_bytes());
        body.extend_from_slice(format!("Content-Type: {}\r\n\r\n", mime_type).as_bytes());
        body.extend_from_slice(content);
        body.extend_from_slice(b"\r\n");

        body.extend_from_slice(format!("--{}--", boundary).as_bytes());

        let response = self.client()
            .post(format!("{}/files", UPLOAD_URL))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("Content-Type", format!("multipart/related; boundary={}", boundary))
            .query(&[("uploadType", "multipart")])
            .body(body)
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

        let file: DriveFile = response.json().await?;
        Ok(file)
    }

    pub async fn create_drive_folder(&self, parent_id: Option<&str>, name: &str) -> Result<DriveFile> {
        let metadata = serde_json::json!({
            "name": name,
            "mimeType": "application/vnd.google-apps.folder",
            "parents": parent_id.map(|f| vec![f]).unwrap_or_default()
        });

        let response = self.client()
            .post(format!("{}/files", DRIVE_URL))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&metadata)
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

        let file: DriveFile = response.json().await?;
        Ok(file)
    }

    pub async fn delete_drive_file(&self, file_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/files/{}", DRIVE_URL, file_id))
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct DriveFilesResponse {
    pub files: Vec<DriveFile>,
    #[serde(rename = "nextPageToken")]
    pub next_page_token: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DriveFile {
    pub id: String,
    pub name: String,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub size: Option<String>,
    #[serde(rename = "webViewLink")]
    pub web_view_link: Option<String>,
    pub parents: Option<Vec<String>>,
    #[serde(rename = "createdTime")]
    pub created_time: Option<String>,
    #[serde(rename = "modifiedTime")]
    pub modified_time: Option<String>,
}

#[async_trait]
impl FileStorageProvider for GoogleClient {
    async fn list_files(&self, folder_id: Option<&str>) -> Result<Vec<File>> {
        let response = self.list_drive_files(folder_id, None).await?;
        let files = response.files.into_iter()
            .filter(|f| f.mime_type != "application/vnd.google-apps.folder")
            .map(|f| File {
                id: f.id,
                name: f.name,
                mime_type: f.mime_type,
                size: f.size.and_then(|s| s.parse().ok()),
                url: f.web_view_link,
                parent_id: f.parents.and_then(|p| p.first().cloned()),
                created_at: f.created_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
                updated_at: f.modified_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            })
            .collect();

        Ok(files)
    }

    async fn get_file(&self, file_id: &str) -> Result<File> {
        let f = self.get_drive_file(file_id).await?;
        Ok(File {
            id: f.id,
            name: f.name,
            mime_type: f.mime_type,
            size: f.size.and_then(|s| s.parse().ok()),
            url: f.web_view_link,
            parent_id: f.parents.and_then(|p| p.first().cloned()),
            created_at: f.created_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: f.modified_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
        })
    }

    async fn download_file(&self, file_id: &str) -> Result<Vec<u8>> {
        self.download_drive_file(file_id).await
    }

    async fn upload_file(&self, folder_id: Option<&str>, name: &str, content: &[u8], mime_type: &str) -> Result<File> {
        let f = self.upload_drive_file(folder_id, name, content, mime_type).await?;
        Ok(File {
            id: f.id,
            name: f.name,
            mime_type: f.mime_type,
            size: f.size.and_then(|s| s.parse().ok()),
            url: f.web_view_link,
            parent_id: f.parents.and_then(|p| p.first().cloned()),
            created_at: f.created_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: f.modified_time.and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
        })
    }

    async fn delete_file(&self, file_id: &str) -> Result<()> {
        self.delete_drive_file(file_id).await
    }

    async fn create_folder(&self, parent_id: Option<&str>, name: &str) -> Result<Folder> {
        let f = self.create_drive_folder(parent_id, name).await?;
        Ok(Folder {
            id: f.id,
            name: f.name,
            parent_id: f.parents.and_then(|p| p.first().cloned()),
            url: f.web_view_link,
        })
    }
}
