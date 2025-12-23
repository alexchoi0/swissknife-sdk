use crate::error::Result;
use crate::tool::{get_i64_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_cloud_sdk::{ObjectStorageProvider, PresignedUrlProvider, ListOptions, PresignOptions};

pub struct S3ListObjectsTool;

impl Default for S3ListObjectsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for S3ListObjectsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "s3_list_objects",
            "S3 List Objects",
            "List objects in an S3 bucket (or compatible storage like GCS, Dropbox)",
            "cloud",
        )
        .with_param("access_key", ParameterSchema::string("Access key or API key").required().user_only())
        .with_param("secret_key", ParameterSchema::string("Secret key").user_only())
        .with_param("provider", ParameterSchema::string("Provider: s3, gcs, dropbox").with_default(serde_json::json!("s3")))
        .with_param("bucket", ParameterSchema::string("Bucket name").required())
        .with_param("prefix", ParameterSchema::string("Object key prefix"))
        .with_param("max_keys", ParameterSchema::integer("Maximum objects to return"))
        .with_param("region", ParameterSchema::string("AWS region").with_default(serde_json::json!("us-east-1")))
        .with_output("objects", OutputSchema::array("List of objects", OutputSchema::json("Object")))
        .with_output("is_truncated", OutputSchema::boolean("Whether results are truncated"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_key = get_required_string_param(&params, "access_key")?;
        let secret_key = get_string_param(&params, "secret_key");
        let provider = get_string_param(&params, "provider").unwrap_or_else(|| "s3".to_string());
        let bucket = get_required_string_param(&params, "bucket")?;
        let prefix = get_string_param(&params, "prefix");
        let max_keys = get_i64_param(&params, "max_keys").map(|v| v as u32);
        let _region = get_string_param(&params, "region");

        let options = ListOptions {
            prefix,
            max_keys,
            ..Default::default()
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "s3")]
            "s3" => {
                use swissknife_cloud_sdk::s3::S3Client;
                let secret = secret_key.ok_or_else(|| crate::Error::MissingParameter("secret_key".into()))?;
                let client = S3Client::new(&access_key, &secret, _region.as_deref());
                client.list_objects(&bucket, &options).await
            }
            #[cfg(feature = "gcs")]
            "gcs" => {
                use swissknife_cloud_sdk::gcs::GcsClient;
                let client = GcsClient::new(&access_key);
                client.list_objects(&bucket, &options).await
            }
            #[cfg(feature = "dropbox")]
            "dropbox" => {
                use swissknife_cloud_sdk::dropbox::DropboxClient;
                let client = DropboxClient::new(&access_key);
                client.list_objects(&bucket, &options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported cloud provider: {}", provider)));
            }
        };

        match result {
            Ok(list_result) => Ok(ToolResponse::success(serde_json::json!({
                "objects": list_result.objects.iter().map(|o| serde_json::json!({
                    "key": o.key,
                    "size": o.size,
                    "content_type": o.content_type,
                    "last_modified": o.last_modified.map(|t| t.to_rfc3339()),
                    "etag": o.etag,
                })).collect::<Vec<_>>(),
                "common_prefixes": list_result.common_prefixes,
                "is_truncated": list_result.is_truncated,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("List objects failed: {}", e))),
        }
    }
}

pub struct S3GetObjectTool;

impl Default for S3GetObjectTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for S3GetObjectTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "s3_get_object",
            "S3 Get Object",
            "Get an object from S3 (returns base64 encoded content)",
            "cloud",
        )
        .with_param("access_key", ParameterSchema::string("Access key").required().user_only())
        .with_param("secret_key", ParameterSchema::string("Secret key").user_only())
        .with_param("provider", ParameterSchema::string("Provider: s3, gcs, dropbox").with_default(serde_json::json!("s3")))
        .with_param("bucket", ParameterSchema::string("Bucket name").required())
        .with_param("key", ParameterSchema::string("Object key").required())
        .with_param("region", ParameterSchema::string("AWS region"))
        .with_output("content", OutputSchema::string("Base64 encoded content"))
        .with_output("content_type", OutputSchema::string("Content type"))
        .with_output("size", OutputSchema::number("Content size"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_key = get_required_string_param(&params, "access_key")?;
        let secret_key = get_string_param(&params, "secret_key");
        let provider = get_string_param(&params, "provider").unwrap_or_else(|| "s3".to_string());
        let bucket = get_required_string_param(&params, "bucket")?;
        let key = get_required_string_param(&params, "key")?;
        let _region = get_string_param(&params, "region");

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "s3")]
            "s3" => {
                use swissknife_cloud_sdk::s3::S3Client;
                let secret = secret_key.ok_or_else(|| crate::Error::MissingParameter("secret_key".into()))?;
                let client = S3Client::new(&access_key, &secret, _region.as_deref());
                client.get_object(&bucket, &key).await
            }
            #[cfg(feature = "gcs")]
            "gcs" => {
                use swissknife_cloud_sdk::gcs::GcsClient;
                let client = GcsClient::new(&access_key);
                client.get_object(&bucket, &key).await
            }
            #[cfg(feature = "dropbox")]
            "dropbox" => {
                use swissknife_cloud_sdk::dropbox::DropboxClient;
                let client = DropboxClient::new(&access_key);
                client.get_object(&bucket, &key).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported cloud provider: {}", provider)));
            }
        };

        match result {
            Ok(content) => {
                use base64::Engine;
                let encoded = base64::engine::general_purpose::STANDARD.encode(&content);
                Ok(ToolResponse::success(serde_json::json!({
                    "content": encoded,
                    "size": content.len(),
                })))
            }
            Err(e) => Ok(ToolResponse::error(format!("Get object failed: {}", e))),
        }
    }
}

pub struct S3PutObjectTool;

impl Default for S3PutObjectTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for S3PutObjectTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "s3_put_object",
            "S3 Put Object",
            "Upload an object to S3 (content should be base64 encoded)",
            "cloud",
        )
        .with_param("access_key", ParameterSchema::string("Access key").required().user_only())
        .with_param("secret_key", ParameterSchema::string("Secret key").user_only())
        .with_param("provider", ParameterSchema::string("Provider: s3, gcs, dropbox").with_default(serde_json::json!("s3")))
        .with_param("bucket", ParameterSchema::string("Bucket name").required())
        .with_param("key", ParameterSchema::string("Object key").required())
        .with_param("content", ParameterSchema::string("Base64 encoded content").required())
        .with_param("content_type", ParameterSchema::string("Content type"))
        .with_param("region", ParameterSchema::string("AWS region"))
        .with_output("etag", OutputSchema::string("ETag of uploaded object"))
        .with_output("success", OutputSchema::boolean("Whether upload succeeded"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_key = get_required_string_param(&params, "access_key")?;
        let secret_key = get_string_param(&params, "secret_key");
        let provider = get_string_param(&params, "provider").unwrap_or_else(|| "s3".to_string());
        let bucket = get_required_string_param(&params, "bucket")?;
        let key = get_required_string_param(&params, "key")?;
        let content_b64 = get_required_string_param(&params, "content")?;
        let content_type = get_string_param(&params, "content_type");
        let _region = get_string_param(&params, "region");

        use base64::Engine;
        let content = base64::engine::general_purpose::STANDARD
            .decode(&content_b64)
            .map_err(|_| crate::Error::InvalidParameter("content: invalid base64".into()))?;

        let options = swissknife_cloud_sdk::UploadOptions {
            content_type,
            ..Default::default()
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "s3")]
            "s3" => {
                use swissknife_cloud_sdk::s3::S3Client;
                let secret = secret_key.ok_or_else(|| crate::Error::MissingParameter("secret_key".into()))?;
                let client = S3Client::new(&access_key, &secret, _region.as_deref());
                client.put_object(&bucket, &key, &content, &options).await
            }
            #[cfg(feature = "gcs")]
            "gcs" => {
                use swissknife_cloud_sdk::gcs::GcsClient;
                let client = GcsClient::new(&access_key);
                client.put_object(&bucket, &key, &content, &options).await
            }
            #[cfg(feature = "dropbox")]
            "dropbox" => {
                use swissknife_cloud_sdk::dropbox::DropboxClient;
                let client = DropboxClient::new(&access_key);
                client.put_object(&bucket, &key, &content, &options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported cloud provider: {}", provider)));
            }
        };

        match result {
            Ok(upload_result) => Ok(ToolResponse::success(serde_json::json!({
                "etag": upload_result.etag,
                "key": upload_result.key,
                "success": true,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Put object failed: {}", e))),
        }
    }
}

pub struct S3DeleteObjectTool;

impl Default for S3DeleteObjectTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for S3DeleteObjectTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "s3_delete_object",
            "S3 Delete Object",
            "Delete an object from S3",
            "cloud",
        )
        .with_param("access_key", ParameterSchema::string("Access key").required().user_only())
        .with_param("secret_key", ParameterSchema::string("Secret key").user_only())
        .with_param("provider", ParameterSchema::string("Provider: s3, gcs, dropbox").with_default(serde_json::json!("s3")))
        .with_param("bucket", ParameterSchema::string("Bucket name").required())
        .with_param("key", ParameterSchema::string("Object key").required())
        .with_param("region", ParameterSchema::string("AWS region"))
        .with_output("success", OutputSchema::boolean("Whether deletion succeeded"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_key = get_required_string_param(&params, "access_key")?;
        let secret_key = get_string_param(&params, "secret_key");
        let provider = get_string_param(&params, "provider").unwrap_or_else(|| "s3".to_string());
        let bucket = get_required_string_param(&params, "bucket")?;
        let key = get_required_string_param(&params, "key")?;
        let _region = get_string_param(&params, "region");

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "s3")]
            "s3" => {
                use swissknife_cloud_sdk::s3::S3Client;
                let secret = secret_key.ok_or_else(|| crate::Error::MissingParameter("secret_key".into()))?;
                let client = S3Client::new(&access_key, &secret, _region.as_deref());
                client.delete_object(&bucket, &key).await
            }
            #[cfg(feature = "gcs")]
            "gcs" => {
                use swissknife_cloud_sdk::gcs::GcsClient;
                let client = GcsClient::new(&access_key);
                client.delete_object(&bucket, &key).await
            }
            #[cfg(feature = "dropbox")]
            "dropbox" => {
                use swissknife_cloud_sdk::dropbox::DropboxClient;
                let client = DropboxClient::new(&access_key);
                client.delete_object(&bucket, &key).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported cloud provider: {}", provider)));
            }
        };

        match result {
            Ok(()) => Ok(ToolResponse::success(serde_json::json!({
                "success": true,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Delete object failed: {}", e))),
        }
    }
}

pub struct S3GeneratePresignedUrlTool;

impl Default for S3GeneratePresignedUrlTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for S3GeneratePresignedUrlTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "s3_generate_presigned_url",
            "S3 Generate Presigned URL",
            "Generate a presigned URL for S3 object access",
            "cloud",
        )
        .with_param("access_key", ParameterSchema::string("Access key").required().user_only())
        .with_param("secret_key", ParameterSchema::string("Secret key").required().user_only())
        .with_param("bucket", ParameterSchema::string("Bucket name").required())
        .with_param("key", ParameterSchema::string("Object key").required())
        .with_param("operation", ParameterSchema::string("Operation: get, put").with_default(serde_json::json!("get")))
        .with_param("expires_in", ParameterSchema::integer("Expiration time in seconds").with_default(serde_json::json!(3600)))
        .with_param("region", ParameterSchema::string("AWS region"))
        .with_output("url", OutputSchema::string("Presigned URL"))
        .with_output("expires_at", OutputSchema::string("Expiration time"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let access_key = get_required_string_param(&params, "access_key")?;
        let secret_key = get_required_string_param(&params, "secret_key")?;
        let bucket = get_required_string_param(&params, "bucket")?;
        let key = get_required_string_param(&params, "key")?;
        let operation = get_string_param(&params, "operation").unwrap_or_else(|| "get".to_string());
        let expires_in = get_i64_param(&params, "expires_in").unwrap_or(3600) as u64;
        let _region = get_string_param(&params, "region");

        let options = PresignOptions {
            expires_in_seconds: expires_in,
            content_type: None,
        };

        #[cfg(feature = "s3")]
        {
            use swissknife_cloud_sdk::s3::S3Client;
            let client = S3Client::new(&access_key, &secret_key, _region.as_deref());
            let result = if operation == "put" {
                client.generate_presigned_put_url(&bucket, &key, &options).await
            } else {
                client.generate_presigned_get_url(&bucket, &key, &options).await
            };
            match result {
                Ok(presigned) => Ok(ToolResponse::success(serde_json::json!({
                    "url": presigned.url,
                    "method": presigned.method,
                    "expires_at": presigned.expires_at.to_rfc3339(),
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Generate presigned URL failed: {}", e))),
            }
        }
        #[cfg(not(feature = "s3"))]
        {
            let _ = (access_key, secret_key, bucket, key, operation, options, _region);
            Ok(ToolResponse::error("S3 feature not enabled"))
        }
    }
}
