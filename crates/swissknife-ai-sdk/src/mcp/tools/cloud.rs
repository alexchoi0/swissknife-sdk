use rmcp::{tool_router, handler::server::router::tool::ToolRouter};
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "cloud")]
use swissknife_cloud_sdk as cloud;

#[derive(Clone)]
pub struct CloudTools {
    #[cfg(feature = "s3")]
    pub s3: Option<cloud::s3::S3Client>,
    #[cfg(feature = "gcs")]
    pub gcs: Option<cloud::gcs::GcsClient>,
    #[cfg(feature = "dropbox")]
    pub dropbox: Option<cloud::dropbox::DropboxClient>,
}

impl CloudTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "s3")]
            s3: None,
            #[cfg(feature = "gcs")]
            gcs: None,
            #[cfg(feature = "dropbox")]
            dropbox: None,
        }
    }

    #[cfg(feature = "s3")]
    pub fn with_s3(mut self, client: cloud::s3::S3Client) -> Self {
        self.s3 = Some(client);
        self
    }

    #[cfg(feature = "gcs")]
    pub fn with_gcs(mut self, client: cloud::gcs::GcsClient) -> Self {
        self.gcs = Some(client);
        self
    }

    #[cfg(feature = "dropbox")]
    pub fn with_dropbox(mut self, client: cloud::dropbox::DropboxClient) -> Self {
        self.dropbox = Some(client);
        self
    }
}

impl Default for CloudTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct S3ListObjectsRequest {
    pub bucket: String,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub max_keys: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct S3GetObjectRequest {
    pub bucket: String,
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct S3PutObjectRequest {
    pub bucket: String,
    pub key: String,
    pub body: String,
    #[serde(default)]
    pub content_type: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct S3DeleteObjectRequest {
    pub bucket: String,
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct S3GetPresignedUrlRequest {
    pub bucket: String,
    pub key: String,
    #[serde(default)]
    pub expires_in: Option<u64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GcsListObjectsRequest {
    pub bucket: String,
    #[serde(default)]
    pub prefix: Option<String>,
    #[serde(default)]
    pub max_results: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GcsGetObjectRequest {
    pub bucket: String,
    pub object: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GcsUploadObjectRequest {
    pub bucket: String,
    pub object: String,
    pub data: String,
    #[serde(default)]
    pub content_type: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GcsDeleteObjectRequest {
    pub bucket: String,
    pub object: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DropboxListFolderRequest {
    pub path: String,
    #[serde(default)]
    pub recursive: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DropboxDownloadRequest {
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DropboxUploadRequest {
    pub path: String,
    pub content: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DropboxDeleteRequest {
    pub path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DropboxCreateFolderRequest {
    pub path: String,
}

#[tool_router]
impl CloudTools {
    #[cfg(feature = "s3")]
    #[rmcp::tool(description = "List objects in an S3 bucket")]
    pub async fn s3_list_objects(
        &self,
        #[rmcp::tool(aggr)] req: S3ListObjectsRequest,
    ) -> Result<String, String> {
        let client = self.s3.as_ref()
            .ok_or_else(|| "S3 client not configured".to_string())?;

        let objects = client.list_objects(&req.bucket, req.prefix.as_deref(), req.max_keys).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&objects).map_err(|e| e.to_string())
    }

    #[cfg(feature = "s3")]
    #[rmcp::tool(description = "Get an object from S3")]
    pub async fn s3_get_object(
        &self,
        #[rmcp::tool(aggr)] req: S3GetObjectRequest,
    ) -> Result<String, String> {
        let client = self.s3.as_ref()
            .ok_or_else(|| "S3 client not configured".to_string())?;

        let data = client.get_object(&req.bucket, &req.key).await
            .map_err(|e| e.to_string())?;

        String::from_utf8(data)
            .map_err(|_| "Object content is not valid UTF-8".to_string())
    }

    #[cfg(feature = "s3")]
    #[rmcp::tool(description = "Upload an object to S3")]
    pub async fn s3_put_object(
        &self,
        #[rmcp::tool(aggr)] req: S3PutObjectRequest,
    ) -> Result<String, String> {
        let client = self.s3.as_ref()
            .ok_or_else(|| "S3 client not configured".to_string())?;

        client.put_object(
            &req.bucket,
            &req.key,
            req.body.as_bytes(),
            req.content_type.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        Ok(format!("Object uploaded to s3://{}/{}", req.bucket, req.key))
    }

    #[cfg(feature = "s3")]
    #[rmcp::tool(description = "Delete an object from S3")]
    pub async fn s3_delete_object(
        &self,
        #[rmcp::tool(aggr)] req: S3DeleteObjectRequest,
    ) -> Result<String, String> {
        let client = self.s3.as_ref()
            .ok_or_else(|| "S3 client not configured".to_string())?;

        client.delete_object(&req.bucket, &req.key).await
            .map_err(|e| e.to_string())?;

        Ok(format!("Object s3://{}/{} deleted", req.bucket, req.key))
    }

    #[cfg(feature = "s3")]
    #[rmcp::tool(description = "Generate a presigned URL for S3 object")]
    pub async fn s3_get_presigned_url(
        &self,
        #[rmcp::tool(aggr)] req: S3GetPresignedUrlRequest,
    ) -> Result<String, String> {
        let client = self.s3.as_ref()
            .ok_or_else(|| "S3 client not configured".to_string())?;

        let url = client.get_presigned_url(&req.bucket, &req.key, req.expires_in).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "url": url
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "gcs")]
    #[rmcp::tool(description = "List objects in a Google Cloud Storage bucket")]
    pub async fn gcs_list_objects(
        &self,
        #[rmcp::tool(aggr)] req: GcsListObjectsRequest,
    ) -> Result<String, String> {
        let client = self.gcs.as_ref()
            .ok_or_else(|| "GCS client not configured".to_string())?;

        let objects = client.list_objects(&req.bucket, req.prefix.as_deref(), req.max_results).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&objects).map_err(|e| e.to_string())
    }

    #[cfg(feature = "gcs")]
    #[rmcp::tool(description = "Get an object from Google Cloud Storage")]
    pub async fn gcs_get_object(
        &self,
        #[rmcp::tool(aggr)] req: GcsGetObjectRequest,
    ) -> Result<String, String> {
        let client = self.gcs.as_ref()
            .ok_or_else(|| "GCS client not configured".to_string())?;

        let data = client.get_object(&req.bucket, &req.object).await
            .map_err(|e| e.to_string())?;

        String::from_utf8(data)
            .map_err(|_| "Object content is not valid UTF-8".to_string())
    }

    #[cfg(feature = "gcs")]
    #[rmcp::tool(description = "Upload an object to Google Cloud Storage")]
    pub async fn gcs_upload_object(
        &self,
        #[rmcp::tool(aggr)] req: GcsUploadObjectRequest,
    ) -> Result<String, String> {
        let client = self.gcs.as_ref()
            .ok_or_else(|| "GCS client not configured".to_string())?;

        client.upload_object(
            &req.bucket,
            &req.object,
            req.data.as_bytes(),
            req.content_type.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        Ok(format!("Object uploaded to gs://{}/{}", req.bucket, req.object))
    }

    #[cfg(feature = "gcs")]
    #[rmcp::tool(description = "Delete an object from Google Cloud Storage")]
    pub async fn gcs_delete_object(
        &self,
        #[rmcp::tool(aggr)] req: GcsDeleteObjectRequest,
    ) -> Result<String, String> {
        let client = self.gcs.as_ref()
            .ok_or_else(|| "GCS client not configured".to_string())?;

        client.delete_object(&req.bucket, &req.object).await
            .map_err(|e| e.to_string())?;

        Ok(format!("Object gs://{}/{} deleted", req.bucket, req.object))
    }

    #[cfg(feature = "dropbox")]
    #[rmcp::tool(description = "List files in a Dropbox folder")]
    pub async fn dropbox_list_folder(
        &self,
        #[rmcp::tool(aggr)] req: DropboxListFolderRequest,
    ) -> Result<String, String> {
        let client = self.dropbox.as_ref()
            .ok_or_else(|| "Dropbox client not configured".to_string())?;

        let files = client.list_folder(&req.path, req.recursive.unwrap_or(false)).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&files).map_err(|e| e.to_string())
    }

    #[cfg(feature = "dropbox")]
    #[rmcp::tool(description = "Download a file from Dropbox")]
    pub async fn dropbox_download(
        &self,
        #[rmcp::tool(aggr)] req: DropboxDownloadRequest,
    ) -> Result<String, String> {
        let client = self.dropbox.as_ref()
            .ok_or_else(|| "Dropbox client not configured".to_string())?;

        let data = client.download(&req.path).await
            .map_err(|e| e.to_string())?;

        String::from_utf8(data)
            .map_err(|_| "File content is not valid UTF-8".to_string())
    }

    #[cfg(feature = "dropbox")]
    #[rmcp::tool(description = "Upload a file to Dropbox")]
    pub async fn dropbox_upload(
        &self,
        #[rmcp::tool(aggr)] req: DropboxUploadRequest,
    ) -> Result<String, String> {
        let client = self.dropbox.as_ref()
            .ok_or_else(|| "Dropbox client not configured".to_string())?;

        let result = client.upload(&req.path, req.content.as_bytes()).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "dropbox")]
    #[rmcp::tool(description = "Delete a file from Dropbox")]
    pub async fn dropbox_delete(
        &self,
        #[rmcp::tool(aggr)] req: DropboxDeleteRequest,
    ) -> Result<String, String> {
        let client = self.dropbox.as_ref()
            .ok_or_else(|| "Dropbox client not configured".to_string())?;

        client.delete(&req.path).await
            .map_err(|e| e.to_string())?;

        Ok(format!("File {} deleted", req.path))
    }

    #[cfg(feature = "dropbox")]
    #[rmcp::tool(description = "Create a folder in Dropbox")]
    pub async fn dropbox_create_folder(
        &self,
        #[rmcp::tool(aggr)] req: DropboxCreateFolderRequest,
    ) -> Result<String, String> {
        let client = self.dropbox.as_ref()
            .ok_or_else(|| "Dropbox client not configured".to_string())?;

        let result = client.create_folder(&req.path).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }
}
