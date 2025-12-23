use crate::{DirectoryListing, Error, FileInfo, Result, TransferOptions};
use crate::sftp::SftpClient;
use serde::Deserialize;

impl SftpClient {
    pub async fn list(&self, path: &str) -> Result<DirectoryListing> {
        let body = serde_json::json!({
            "connection": self.connection_info(),
            "path": path
        });

        let response = self.client()
            .post(format!("{}/sftp/list", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        let result: ListResponse = response.json().await
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(DirectoryListing {
            path: path.to_string(),
            entries: result.entries.into_iter().map(Into::into).collect(),
        })
    }

    pub async fn stat(&self, path: &str) -> Result<FileInfo> {
        let body = serde_json::json!({
            "connection": self.connection_info(),
            "path": path
        });

        let response = self.client()
            .post(format!("{}/sftp/stat", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(Error::NotFound(path.to_string()));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        let result: FileEntryResponse = response.json().await
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(result.into())
    }

    pub async fn read(&self, path: &str) -> Result<Vec<u8>> {
        let body = serde_json::json!({
            "connection": self.connection_info(),
            "path": path
        });

        let response = self.client()
            .post(format!("{}/sftp/read", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(Error::NotFound(path.to_string()));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        let result: ReadResponse = response.json().await
            .map_err(|e| Error::Io(e.to_string()))?;

        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(&result.content)
            .map_err(|e| Error::Io(e.to_string()))
    }

    pub async fn write(&self, path: &str, data: &[u8], options: &TransferOptions) -> Result<()> {
        use base64::Engine;
        let content = base64::engine::general_purpose::STANDARD.encode(data);

        let body = serde_json::json!({
            "connection": self.connection_info(),
            "path": path,
            "content": content,
            "overwrite": options.overwrite
        });

        let response = self.client()
            .post(format!("{}/sftp/write", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 409 {
                return Err(Error::AlreadyExists(path.to_string()));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        Ok(())
    }

    pub async fn delete(&self, path: &str) -> Result<()> {
        let body = serde_json::json!({
            "connection": self.connection_info(),
            "path": path
        });

        let response = self.client()
            .post(format!("{}/sftp/delete", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(Error::NotFound(path.to_string()));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        Ok(())
    }

    pub async fn mkdir(&self, path: &str, recursive: bool) -> Result<()> {
        let body = serde_json::json!({
            "connection": self.connection_info(),
            "path": path,
            "recursive": recursive
        });

        let response = self.client()
            .post(format!("{}/sftp/mkdir", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        Ok(())
    }

    pub async fn rmdir(&self, path: &str, recursive: bool) -> Result<()> {
        let body = serde_json::json!({
            "connection": self.connection_info(),
            "path": path,
            "recursive": recursive
        });

        let response = self.client()
            .post(format!("{}/sftp/rmdir", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        Ok(())
    }

    pub async fn rename(&self, from: &str, to: &str) -> Result<()> {
        let body = serde_json::json!({
            "connection": self.connection_info(),
            "from": from,
            "to": to
        });

        let response = self.client()
            .post(format!("{}/sftp/rename", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        Ok(())
    }

    pub async fn chmod(&self, path: &str, mode: u32) -> Result<()> {
        let body = serde_json::json!({
            "connection": self.connection_info(),
            "path": path,
            "mode": mode
        });

        let response = self.client()
            .post(format!("{}/sftp/chmod", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        Ok(())
    }

    pub async fn exists(&self, path: &str) -> Result<bool> {
        match self.stat(path).await {
            Ok(_) => Ok(true),
            Err(Error::NotFound(_)) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ListResponse {
    entries: Vec<FileEntryResponse>,
}

#[derive(Debug, Deserialize)]
struct FileEntryResponse {
    name: String,
    path: String,
    size: u64,
    #[serde(rename = "isDirectory")]
    is_directory: bool,
    #[serde(rename = "isSymlink")]
    is_symlink: Option<bool>,
    permissions: Option<String>,
    owner: Option<String>,
    group: Option<String>,
    #[serde(rename = "modifiedAt")]
    modified_at: Option<String>,
    #[serde(rename = "accessedAt")]
    accessed_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ReadResponse {
    content: String,
}

impl From<FileEntryResponse> for FileInfo {
    fn from(entry: FileEntryResponse) -> Self {
        Self {
            name: entry.name,
            path: entry.path,
            size: entry.size,
            is_directory: entry.is_directory,
            is_symlink: entry.is_symlink.unwrap_or(false),
            permissions: entry.permissions,
            owner: entry.owner,
            group: entry.group,
            modified_at: entry.modified_at,
            accessed_at: entry.accessed_at,
            created_at: None,
        }
    }
}
