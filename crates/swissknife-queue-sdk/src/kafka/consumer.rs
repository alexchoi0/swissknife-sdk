use crate::Result;
use super::KafkaClient;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct CreateConsumerRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
    #[serde(rename = "auto.offset.reset", skip_serializing_if = "Option::is_none")]
    pub auto_offset_reset: Option<String>,
    #[serde(rename = "auto.commit.enable", skip_serializing_if = "Option::is_none")]
    pub auto_commit_enable: Option<bool>,
    #[serde(rename = "fetch.min.bytes", skip_serializing_if = "Option::is_none")]
    pub fetch_min_bytes: Option<i32>,
    #[serde(rename = "consumer.request.timeout.ms", skip_serializing_if = "Option::is_none")]
    pub consumer_request_timeout_ms: Option<i32>,
}

impl CreateConsumerRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            format: Some("json".to_string()),
            auto_offset_reset: None,
            auto_commit_enable: None,
            fetch_min_bytes: None,
            consumer_request_timeout_ms: None,
        }
    }

    pub fn with_format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    pub fn with_auto_offset_reset(mut self, reset: impl Into<String>) -> Self {
        self.auto_offset_reset = Some(reset.into());
        self
    }

    pub fn earliest(self) -> Self {
        self.with_auto_offset_reset("earliest")
    }

    pub fn latest(self) -> Self {
        self.with_auto_offset_reset("latest")
    }

    pub fn with_auto_commit(mut self, enable: bool) -> Self {
        self.auto_commit_enable = Some(enable);
        self
    }

    pub fn with_fetch_min_bytes(mut self, bytes: i32) -> Self {
        self.fetch_min_bytes = Some(bytes);
        self
    }

    pub fn with_timeout(mut self, timeout_ms: i32) -> Self {
        self.consumer_request_timeout_ms = Some(timeout_ms);
        self
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateConsumerResponse {
    pub instance_id: String,
    pub base_uri: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionRequest {
    pub topics: Vec<String>,
}

impl SubscriptionRequest {
    pub fn new(topics: Vec<String>) -> Self {
        Self { topics }
    }

    pub fn single(topic: impl Into<String>) -> Self {
        Self {
            topics: vec![topic.into()],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct TopicPatternRequest {
    pub topic_pattern: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PartitionAssignment {
    pub partitions: Vec<TopicPartition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicPartition {
    pub topic: String,
    pub partition: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SeekRequest {
    pub offsets: Vec<PartitionOffsetRequest>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PartitionOffsetRequest {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsumedRecord {
    pub topic: String,
    pub key: Option<serde_json::Value>,
    pub value: serde_json::Value,
    pub partition: i32,
    pub offset: i64,
    #[serde(default)]
    pub timestamp: Option<i64>,
    #[serde(default)]
    pub headers: Option<Vec<super::producer::RecordHeader>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommitOffsetsRequest {
    pub offsets: Vec<PartitionOffsetRequest>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsumerOffsets {
    pub offsets: Vec<CommittedOffset>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommittedOffset {
    pub topic: String,
    pub partition: i32,
    pub offset: i64,
    #[serde(default)]
    pub metadata: Option<String>,
}

pub struct KafkaConsumer {
    client: KafkaClient,
    group_id: String,
    instance_id: String,
    base_uri: String,
}

impl KafkaConsumer {
    pub fn group_id(&self) -> &str {
        &self.group_id
    }

    pub fn instance_id(&self) -> &str {
        &self.instance_id
    }

    pub async fn subscribe(&self, topics: Vec<String>) -> Result<()> {
        let request = SubscriptionRequest::new(topics);
        let path = format!(
            "/consumers/{}/instances/{}/subscription",
            self.group_id, self.instance_id
        );
        self.client.request_no_response(reqwest::Method::POST, &path, Some(&request)).await
    }

    pub async fn subscribe_pattern(&self, pattern: impl Into<String>) -> Result<()> {
        let request = TopicPatternRequest {
            topic_pattern: pattern.into(),
        };
        let path = format!(
            "/consumers/{}/instances/{}/subscription",
            self.group_id, self.instance_id
        );
        self.client.request_no_response(reqwest::Method::POST, &path, Some(&request)).await
    }

    pub async fn get_subscription(&self) -> Result<Vec<String>> {
        let path = format!(
            "/consumers/{}/instances/{}/subscription",
            self.group_id, self.instance_id
        );
        #[derive(Deserialize)]
        struct Response {
            topics: Vec<String>,
        }
        let response: Response = self.client.request(reqwest::Method::GET, &path, None::<&()>).await?;
        Ok(response.topics)
    }

    pub async fn unsubscribe(&self) -> Result<()> {
        let path = format!(
            "/consumers/{}/instances/{}/subscription",
            self.group_id, self.instance_id
        );
        self.client.request_no_response(reqwest::Method::DELETE, &path, None::<&()>).await
    }

    pub async fn assign(&self, partitions: Vec<TopicPartition>) -> Result<()> {
        let request = PartitionAssignment { partitions };
        let path = format!(
            "/consumers/{}/instances/{}/assignments",
            self.group_id, self.instance_id
        );
        self.client.request_no_response(reqwest::Method::POST, &path, Some(&request)).await
    }

    pub async fn get_assignments(&self) -> Result<Vec<TopicPartition>> {
        let path = format!(
            "/consumers/{}/instances/{}/assignments",
            self.group_id, self.instance_id
        );
        let response: PartitionAssignment = self.client.request(reqwest::Method::GET, &path, None::<&()>).await?;
        Ok(response.partitions)
    }

    pub async fn poll(&self, timeout_ms: Option<i32>, max_bytes: Option<i32>) -> Result<Vec<ConsumedRecord>> {
        let mut path = format!(
            "/consumers/{}/instances/{}/records",
            self.group_id, self.instance_id
        );

        let mut params = Vec::new();
        if let Some(t) = timeout_ms {
            params.push(format!("timeout={}", t));
        }
        if let Some(m) = max_bytes {
            params.push(format!("max_bytes={}", m));
        }
        if !params.is_empty() {
            path.push('?');
            path.push_str(&params.join("&"));
        }

        self.client.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn seek(&self, offsets: Vec<PartitionOffsetRequest>) -> Result<()> {
        let request = SeekRequest { offsets };
        let path = format!(
            "/consumers/{}/instances/{}/positions",
            self.group_id, self.instance_id
        );
        self.client.request_no_response(reqwest::Method::POST, &path, Some(&request)).await
    }

    pub async fn seek_to_beginning(&self, partitions: Vec<TopicPartition>) -> Result<()> {
        let request = PartitionAssignment { partitions };
        let path = format!(
            "/consumers/{}/instances/{}/positions/beginning",
            self.group_id, self.instance_id
        );
        self.client.request_no_response(reqwest::Method::POST, &path, Some(&request)).await
    }

    pub async fn seek_to_end(&self, partitions: Vec<TopicPartition>) -> Result<()> {
        let request = PartitionAssignment { partitions };
        let path = format!(
            "/consumers/{}/instances/{}/positions/end",
            self.group_id, self.instance_id
        );
        self.client.request_no_response(reqwest::Method::POST, &path, Some(&request)).await
    }

    pub async fn commit(&self) -> Result<()> {
        let path = format!(
            "/consumers/{}/instances/{}/offsets",
            self.group_id, self.instance_id
        );
        self.client.request_no_response(reqwest::Method::POST, &path, None::<&()>).await
    }

    pub async fn commit_offsets(&self, offsets: Vec<PartitionOffsetRequest>) -> Result<()> {
        let request = CommitOffsetsRequest { offsets };
        let path = format!(
            "/consumers/{}/instances/{}/offsets",
            self.group_id, self.instance_id
        );
        self.client.request_no_response(reqwest::Method::POST, &path, Some(&request)).await
    }

    pub async fn get_committed_offsets(&self, partitions: Vec<TopicPartition>) -> Result<ConsumerOffsets> {
        let request = PartitionAssignment { partitions };
        let path = format!(
            "/consumers/{}/instances/{}/offsets",
            self.group_id, self.instance_id
        );
        self.client.request(reqwest::Method::GET, &path, Some(&request)).await
    }

    pub async fn close(self) -> Result<()> {
        let path = format!(
            "/consumers/{}/instances/{}",
            self.group_id, self.instance_id
        );
        self.client.request_no_response(reqwest::Method::DELETE, &path, None::<&()>).await
    }
}

impl KafkaClient {
    pub async fn create_consumer(
        &self,
        group_id: &str,
        request: &CreateConsumerRequest,
    ) -> Result<KafkaConsumer> {
        let path = format!("/consumers/{}", group_id);
        let response: CreateConsumerResponse = self.request(reqwest::Method::POST, &path, Some(request)).await?;

        Ok(KafkaConsumer {
            client: self.clone(),
            group_id: group_id.to_string(),
            instance_id: response.instance_id,
            base_uri: response.base_uri,
        })
    }

    pub async fn delete_consumer(&self, group_id: &str, instance_id: &str) -> Result<()> {
        let path = format!("/consumers/{}/instances/{}", group_id, instance_id);
        self.request_no_response(reqwest::Method::DELETE, &path, None::<&()>).await
    }
}
