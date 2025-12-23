use reqwest::Client;
use chrono::Utc;
use std::collections::HashMap;

pub struct DynamoDbClient {
    region: String,
    access_key_id: String,
    secret_access_key: String,
    session_token: Option<String>,
    client: Client,
}

impl DynamoDbClient {
    pub fn new(region: &str, access_key_id: &str, secret_access_key: &str) -> Self {
        Self {
            region: region.to_string(),
            access_key_id: access_key_id.to_string(),
            secret_access_key: secret_access_key.to_string(),
            session_token: None,
            client: Client::new(),
        }
    }

    pub fn with_session_token(mut self, token: &str) -> Self {
        self.session_token = Some(token.to_string());
        self
    }

    pub(crate) fn client(&self) -> &Client {
        &self.client
    }

    pub(crate) fn endpoint(&self) -> String {
        format!("https://dynamodb.{}.amazonaws.com", self.region)
    }

    pub(crate) fn region(&self) -> &str {
        &self.region
    }

    pub(crate) async fn sign_and_send(&self, target: &str, body: &serde_json::Value) -> crate::Result<serde_json::Value> {
        let body_str = serde_json::to_string(body)?;
        let endpoint = self.endpoint();
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();

        let content_type = "application/x-amz-json-1.0";
        let host = format!("dynamodb.{}.amazonaws.com", self.region);

        let canonical_headers = format!(
            "content-type:{}\nhost:{}\nx-amz-date:{}\nx-amz-target:{}\n",
            content_type, host, amz_date, target
        );
        let signed_headers = "content-type;host;x-amz-date;x-amz-target";

        let payload_hash = sha256_hex(&body_str);
        let canonical_request = format!(
            "POST\n/\n\n{}\n{}\n{}",
            canonical_headers, signed_headers, payload_hash
        );

        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{}/{}/dynamodb/aws4_request", date_stamp, self.region);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm,
            amz_date,
            credential_scope,
            sha256_hex(&canonical_request)
        );

        let k_date = hmac_sha256(format!("AWS4{}", self.secret_access_key).as_bytes(), date_stamp.as_bytes());
        let k_region = hmac_sha256(&k_date, self.region.as_bytes());
        let k_service = hmac_sha256(&k_region, b"dynamodb");
        let k_signing = hmac_sha256(&k_service, b"aws4_request");
        let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()));

        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm,
            self.access_key_id,
            credential_scope,
            signed_headers,
            signature
        );

        let mut request = self.client
            .post(&endpoint)
            .header("Content-Type", content_type)
            .header("Host", host)
            .header("X-Amz-Date", amz_date)
            .header("X-Amz-Target", target)
            .header("Authorization", authorization);

        if let Some(token) = &self.session_token {
            request = request.header("X-Amz-Security-Token", token);
        }

        let response = request.body(body_str).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(crate::Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result)
    }
}

fn sha256_hex(data: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hex::encode(hasher.finalize())
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}
