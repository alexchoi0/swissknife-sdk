use crate::Result;
use super::KafkaClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ProduceRequest {
    pub records: Vec<ProduceRecord>,
}

impl ProduceRequest {
    pub fn new() -> Self {
        Self { records: Vec::new() }
    }

    pub fn with_record(mut self, record: ProduceRecord) -> Self {
        self.records.push(record);
        self
    }

    pub fn with_records(mut self, records: Vec<ProduceRecord>) -> Self {
        self.records.extend(records);
        self
    }
}

impl Default for ProduceRequest {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ProduceRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<serde_json::Value>,
    pub value: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partition: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Vec<RecordHeader>>,
}

impl ProduceRecord {
    pub fn new(value: serde_json::Value) -> Self {
        Self {
            key: None,
            value,
            partition: None,
            headers: None,
        }
    }

    pub fn with_key(mut self, key: serde_json::Value) -> Self {
        self.key = Some(key);
        self
    }

    pub fn with_partition(mut self, partition: i32) -> Self {
        self.partition = Some(partition);
        self
    }

    pub fn with_headers(mut self, headers: Vec<RecordHeader>) -> Self {
        self.headers = Some(headers);
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        let header = RecordHeader {
            key: key.into(),
            value: value.into(),
        };
        match &mut self.headers {
            Some(headers) => headers.push(header),
            None => self.headers = Some(vec![header]),
        }
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecordHeader {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProduceResponse {
    pub offsets: Vec<PartitionOffset>,
    #[serde(default)]
    pub key_schema_id: Option<i32>,
    #[serde(default)]
    pub value_schema_id: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartitionOffset {
    pub partition: i32,
    pub offset: i64,
    #[serde(default)]
    pub error_code: Option<i32>,
    #[serde(default)]
    pub error: Option<String>,
}

impl KafkaClient {
    pub async fn produce(&self, topic: &str, request: &ProduceRequest) -> Result<ProduceResponse> {
        let path = format!("/topics/{}", topic);
        self.request(reqwest::Method::POST, &path, Some(request)).await
    }

    pub async fn produce_single(
        &self,
        topic: &str,
        value: serde_json::Value,
    ) -> Result<ProduceResponse> {
        let record = ProduceRecord::new(value);
        let request = ProduceRequest::new().with_record(record);
        self.produce(topic, &request).await
    }

    pub async fn produce_with_key(
        &self,
        topic: &str,
        key: serde_json::Value,
        value: serde_json::Value,
    ) -> Result<ProduceResponse> {
        let record = ProduceRecord::new(value).with_key(key);
        let request = ProduceRequest::new().with_record(record);
        self.produce(topic, &request).await
    }

    pub async fn produce_to_partition(
        &self,
        topic: &str,
        partition: i32,
        records: Vec<ProduceRecord>,
    ) -> Result<ProduceResponse> {
        let path = format!("/topics/{}/partitions/{}", topic, partition);
        let request = ProduceRequest { records };
        self.request(reqwest::Method::POST, &path, Some(&request)).await
    }
}
