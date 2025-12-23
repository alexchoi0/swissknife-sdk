use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "file")]
use swissknife_file_sdk as file;

#[derive(Clone)]
pub struct FileTools {
    #[cfg(feature = "sftp")]
    pub sftp: Option<file::sftp::SftpClient>,
}

impl FileTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "sftp")]
            sftp: None,
        }
    }

    #[cfg(feature = "sftp")]
    pub fn with_sftp(mut self, client: file::sftp::SftpClient) -> Self {
        self.sftp = Some(client);
        self
    }
}

impl Default for FileTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SftpListRequest {
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SftpReadRequest {
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SftpWriteRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SftpDeleteRequest {
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SftpMkdirRequest {
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SftpMoveRequest {
    pub source: String,
    pub destination: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SftpStatRequest {
    pub path: String,
}

#[tool_box]
impl FileTools {
    #[cfg(feature = "sftp")]
    #[rmcp::tool(description = "List files in a directory via SFTP")]
    pub async fn sftp_list(
        &self,
        #[rmcp::tool(aggr)] req: SftpListRequest,
    ) -> Result<String, String> {
        let client = self.sftp.as_ref()
            .ok_or_else(|| "SFTP client not configured".to_string())?;

        let files = client.list(&req.path).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&files).map_err(|e| e.to_string())
    }

    #[cfg(feature = "sftp")]
    #[rmcp::tool(description = "Read a file via SFTP")]
    pub async fn sftp_read(
        &self,
        #[rmcp::tool(aggr)] req: SftpReadRequest,
    ) -> Result<String, String> {
        let client = self.sftp.as_ref()
            .ok_or_else(|| "SFTP client not configured".to_string())?;

        let content = client.read(&req.path).await
            .map_err(|e| e.to_string())?;

        String::from_utf8(content)
            .map_err(|_| "File content is not valid UTF-8".to_string())
    }

    #[cfg(feature = "sftp")]
    #[rmcp::tool(description = "Write a file via SFTP")]
    pub async fn sftp_write(
        &self,
        #[rmcp::tool(aggr)] req: SftpWriteRequest,
    ) -> Result<String, String> {
        let client = self.sftp.as_ref()
            .ok_or_else(|| "SFTP client not configured".to_string())?;

        client.write(&req.path, req.content.as_bytes()).await
            .map_err(|e| e.to_string())?;

        Ok(format!("File written to {}", req.path))
    }

    #[cfg(feature = "sftp")]
    #[rmcp::tool(description = "Delete a file via SFTP")]
    pub async fn sftp_delete(
        &self,
        #[rmcp::tool(aggr)] req: SftpDeleteRequest,
    ) -> Result<String, String> {
        let client = self.sftp.as_ref()
            .ok_or_else(|| "SFTP client not configured".to_string())?;

        client.delete(&req.path).await
            .map_err(|e| e.to_string())?;

        Ok(format!("File {} deleted", req.path))
    }

    #[cfg(feature = "sftp")]
    #[rmcp::tool(description = "Create a directory via SFTP")]
    pub async fn sftp_mkdir(
        &self,
        #[rmcp::tool(aggr)] req: SftpMkdirRequest,
    ) -> Result<String, String> {
        let client = self.sftp.as_ref()
            .ok_or_else(|| "SFTP client not configured".to_string())?;

        client.mkdir(&req.path).await
            .map_err(|e| e.to_string())?;

        Ok(format!("Directory {} created", req.path))
    }

    #[cfg(feature = "sftp")]
    #[rmcp::tool(description = "Move/rename a file via SFTP")]
    pub async fn sftp_move(
        &self,
        #[rmcp::tool(aggr)] req: SftpMoveRequest,
    ) -> Result<String, String> {
        let client = self.sftp.as_ref()
            .ok_or_else(|| "SFTP client not configured".to_string())?;

        client.rename(&req.source, &req.destination).await
            .map_err(|e| e.to_string())?;

        Ok(format!("Moved {} to {}", req.source, req.destination))
    }

    #[cfg(feature = "sftp")]
    #[rmcp::tool(description = "Get file/directory info via SFTP")]
    pub async fn sftp_stat(
        &self,
        #[rmcp::tool(aggr)] req: SftpStatRequest,
    ) -> Result<String, String> {
        let client = self.sftp.as_ref()
            .ok_or_else(|| "SFTP client not configured".to_string())?;

        let stat = client.stat(&req.path).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&stat).map_err(|e| e.to_string())
    }
}
