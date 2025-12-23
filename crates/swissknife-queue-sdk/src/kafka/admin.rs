use crate::Result;
use super::KafkaClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize)]
pub struct ClusterInfo {
    pub cluster_id: String,
    #[serde(default)]
    pub controller: Option<BrokerInfo>,
    #[serde(default)]
    pub brokers: Option<BrokersLink>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrokerInfo {
    pub broker_id: i32,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<i32>,
    #[serde(default)]
    pub rack: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrokersLink {
    pub related: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrokerList {
    pub data: Vec<BrokerData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrokerData {
    pub cluster_id: String,
    pub broker_id: i32,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub port: Option<i32>,
    #[serde(default)]
    pub rack: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopicList {
    #[serde(default)]
    pub data: Vec<TopicInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopicInfo {
    pub cluster_id: String,
    pub topic_name: String,
    pub is_internal: bool,
    pub replication_factor: i32,
    pub partitions_count: i32,
    #[serde(default)]
    pub partitions: Option<PartitionsLink>,
    #[serde(default)]
    pub configs: Option<ConfigsLink>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartitionsLink {
    pub related: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConfigsLink {
    pub related: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateTopicRequest {
    pub topic_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partitions_count: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replication_factor: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configs: Option<Vec<TopicConfig>>,
}

impl CreateTopicRequest {
    pub fn new(topic_name: impl Into<String>) -> Self {
        Self {
            topic_name: topic_name.into(),
            partitions_count: None,
            replication_factor: None,
            configs: None,
        }
    }

    pub fn with_partitions(mut self, count: i32) -> Self {
        self.partitions_count = Some(count);
        self
    }

    pub fn with_replication_factor(mut self, factor: i32) -> Self {
        self.replication_factor = Some(factor);
        self
    }

    pub fn with_config(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let config = TopicConfig {
            name: name.into(),
            value: value.into(),
        };
        match &mut self.configs {
            Some(configs) => configs.push(config),
            None => self.configs = Some(vec![config]),
        }
        self
    }

    pub fn with_retention_ms(self, ms: i64) -> Self {
        self.with_config("retention.ms", ms.to_string())
    }

    pub fn with_cleanup_policy(self, policy: impl Into<String>) -> Self {
        self.with_config("cleanup.policy", policy)
    }

    pub fn compacted(self) -> Self {
        self.with_cleanup_policy("compact")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartitionList {
    pub data: Vec<PartitionInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartitionInfo {
    pub cluster_id: String,
    pub topic_name: String,
    pub partition_id: i32,
    #[serde(default)]
    pub leader: Option<ReplicaInfo>,
    #[serde(default)]
    pub replicas: Option<ReplicasLink>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReplicaInfo {
    pub broker_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReplicasLink {
    pub related: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopicConfigList {
    pub data: Vec<TopicConfigInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopicConfigInfo {
    pub cluster_id: String,
    pub topic_name: String,
    pub name: String,
    #[serde(default)]
    pub value: Option<String>,
    pub is_default: bool,
    pub is_read_only: bool,
    pub is_sensitive: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateTopicConfigRequest {
    pub data: Vec<UpdateConfigEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateConfigEntry {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsumerGroupList {
    pub data: Vec<ConsumerGroupInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsumerGroupInfo {
    pub cluster_id: String,
    pub consumer_group_id: String,
    pub is_simple: bool,
    pub partition_assignor: String,
    pub state: String,
    #[serde(default)]
    pub coordinator: Option<CoordinatorInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CoordinatorInfo {
    pub broker_id: i32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsumerGroupDetail {
    pub cluster_id: String,
    pub consumer_group_id: String,
    pub is_simple: bool,
    pub partition_assignor: String,
    pub state: String,
    #[serde(default)]
    pub coordinator: Option<CoordinatorInfo>,
    #[serde(default)]
    pub consumers: Option<ConsumersLink>,
    #[serde(default)]
    pub lag_summary: Option<LagSummaryLink>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsumersLink {
    pub related: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LagSummaryLink {
    pub related: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ConsumerLagSummary {
    pub cluster_id: String,
    pub consumer_group_id: String,
    pub max_lag_consumer_id: String,
    pub max_lag_instance_id: Option<String>,
    pub max_lag_client_id: String,
    pub max_lag_topic_name: String,
    pub max_lag_partition_id: i32,
    pub max_lag: i64,
    pub total_lag: i64,
}

impl KafkaClient {
    pub async fn get_cluster(&self, cluster_id: &str) -> Result<ClusterInfo> {
        let path = format!("/v3/clusters/{}", cluster_id);
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn list_brokers(&self, cluster_id: &str) -> Result<BrokerList> {
        let path = format!("/v3/clusters/{}/brokers", cluster_id);
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn get_broker(&self, cluster_id: &str, broker_id: i32) -> Result<BrokerData> {
        let path = format!("/v3/clusters/{}/brokers/{}", cluster_id, broker_id);
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn list_topics_v3(&self, cluster_id: &str) -> Result<TopicList> {
        let path = format!("/v3/clusters/{}/topics", cluster_id);
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn list_topics(&self) -> Result<Vec<String>> {
        let path = "/topics";
        self.request(reqwest::Method::GET, path, None::<&()>).await
    }

    pub async fn get_topic(&self, cluster_id: &str, topic_name: &str) -> Result<TopicInfo> {
        let path = format!("/v3/clusters/{}/topics/{}", cluster_id, topic_name);
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn get_topic_metadata(&self, topic_name: &str) -> Result<TopicMetadata> {
        let path = format!("/topics/{}", topic_name);
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn create_topic(&self, cluster_id: &str, request: &CreateTopicRequest) -> Result<TopicInfo> {
        let path = format!("/v3/clusters/{}/topics", cluster_id);
        self.request(reqwest::Method::POST, &path, Some(request)).await
    }

    pub async fn delete_topic(&self, cluster_id: &str, topic_name: &str) -> Result<()> {
        let path = format!("/v3/clusters/{}/topics/{}", cluster_id, topic_name);
        self.request_no_response(reqwest::Method::DELETE, &path, None::<&()>).await
    }

    pub async fn list_partitions(&self, cluster_id: &str, topic_name: &str) -> Result<PartitionList> {
        let path = format!("/v3/clusters/{}/topics/{}/partitions", cluster_id, topic_name);
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn get_partition(
        &self,
        cluster_id: &str,
        topic_name: &str,
        partition_id: i32,
    ) -> Result<PartitionInfo> {
        let path = format!(
            "/v3/clusters/{}/topics/{}/partitions/{}",
            cluster_id, topic_name, partition_id
        );
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn list_topic_configs(&self, cluster_id: &str, topic_name: &str) -> Result<TopicConfigList> {
        let path = format!("/v3/clusters/{}/topics/{}/configs", cluster_id, topic_name);
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn get_topic_config(
        &self,
        cluster_id: &str,
        topic_name: &str,
        config_name: &str,
    ) -> Result<TopicConfigInfo> {
        let path = format!(
            "/v3/clusters/{}/topics/{}/configs/{}",
            cluster_id, topic_name, config_name
        );
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn update_topic_config(
        &self,
        cluster_id: &str,
        topic_name: &str,
        config_name: &str,
        value: &str,
    ) -> Result<()> {
        let path = format!(
            "/v3/clusters/{}/topics/{}/configs/{}",
            cluster_id, topic_name, config_name
        );
        #[derive(Serialize)]
        struct Body {
            value: String,
        }
        let body = Body {
            value: value.to_string(),
        };
        self.request_no_response(reqwest::Method::PUT, &path, Some(&body)).await
    }

    pub async fn batch_update_topic_configs(
        &self,
        cluster_id: &str,
        topic_name: &str,
        configs: HashMap<String, String>,
    ) -> Result<()> {
        let path = format!(
            "/v3/clusters/{}/topics/{}/configs:alter",
            cluster_id, topic_name
        );
        let data: Vec<UpdateConfigEntry> = configs
            .into_iter()
            .map(|(name, value)| UpdateConfigEntry { name, value })
            .collect();
        let request = UpdateTopicConfigRequest { data };
        self.request_no_response(reqwest::Method::POST, &path, Some(&request)).await
    }

    pub async fn list_consumer_groups(&self, cluster_id: &str) -> Result<ConsumerGroupList> {
        let path = format!("/v3/clusters/{}/consumer-groups", cluster_id);
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn get_consumer_group(&self, cluster_id: &str, group_id: &str) -> Result<ConsumerGroupDetail> {
        let path = format!("/v3/clusters/{}/consumer-groups/{}", cluster_id, group_id);
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }

    pub async fn get_consumer_lag_summary(
        &self,
        cluster_id: &str,
        group_id: &str,
    ) -> Result<ConsumerLagSummary> {
        let path = format!(
            "/v3/clusters/{}/consumer-groups/{}/lag-summary",
            cluster_id, group_id
        );
        self.request(reqwest::Method::GET, &path, None::<&()>).await
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TopicMetadata {
    pub name: String,
    pub configs: HashMap<String, String>,
    pub partitions: Vec<PartitionMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PartitionMetadata {
    pub partition: i32,
    pub leader: i32,
    pub replicas: Vec<ReplicaMetadata>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReplicaMetadata {
    pub broker: i32,
    pub leader: bool,
    pub in_sync: bool,
}
