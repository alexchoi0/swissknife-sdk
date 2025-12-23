use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "queue")]
use swissknife_queue_sdk as queue;

#[derive(Clone)]
pub struct QueueTools {
    #[cfg(feature = "sqs")]
    pub sqs: Option<queue::sqs::SqsClient>,
    #[cfg(feature = "kafka")]
    pub kafka: Option<queue::kafka::KafkaClient>,
}

impl QueueTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "sqs")]
            sqs: None,
            #[cfg(feature = "kafka")]
            kafka: None,
        }
    }

    #[cfg(feature = "sqs")]
    pub fn with_sqs(mut self, client: queue::sqs::SqsClient) -> Self {
        self.sqs = Some(client);
        self
    }

    #[cfg(feature = "kafka")]
    pub fn with_kafka(mut self, client: queue::kafka::KafkaClient) -> Self {
        self.kafka = Some(client);
        self
    }
}

impl Default for QueueTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SqsSendMessageRequest {
    pub queue_url: String,
    pub message_body: String,
    #[serde(default)]
    pub delay_seconds: Option<u32>,
    #[serde(default)]
    pub message_group_id: Option<String>,
    #[serde(default)]
    pub message_deduplication_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SqsReceiveMessagesRequest {
    pub queue_url: String,
    #[serde(default)]
    pub max_number_of_messages: Option<u32>,
    #[serde(default)]
    pub visibility_timeout: Option<u32>,
    #[serde(default)]
    pub wait_time_seconds: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SqsDeleteMessageRequest {
    pub queue_url: String,
    pub receipt_handle: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SqsGetQueueAttributesRequest {
    pub queue_url: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SqsPurgeQueueRequest {
    pub queue_url: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SqsChangeVisibilityRequest {
    pub queue_url: String,
    pub receipt_handle: String,
    pub visibility_timeout: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KafkaProduceRequest {
    pub topic: String,
    pub value: serde_json::Value,
    #[serde(default)]
    pub key: Option<serde_json::Value>,
    #[serde(default)]
    pub partition: Option<i32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KafkaProduceBatchRequest {
    pub topic: String,
    pub messages: Vec<KafkaMessageInput>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KafkaMessageInput {
    pub value: serde_json::Value,
    #[serde(default)]
    pub key: Option<serde_json::Value>,
    #[serde(default)]
    pub partition: Option<i32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KafkaListTopicsRequest {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KafkaGetTopicRequest {
    pub cluster_id: String,
    pub topic_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KafkaCreateTopicRequest {
    pub cluster_id: String,
    pub topic_name: String,
    #[serde(default)]
    pub partitions: Option<i32>,
    #[serde(default)]
    pub replication_factor: Option<i32>,
    #[serde(default)]
    pub retention_ms: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KafkaDeleteTopicRequest {
    pub cluster_id: String,
    pub topic_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KafkaListConsumerGroupsRequest {
    pub cluster_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KafkaGetConsumerGroupRequest {
    pub cluster_id: String,
    pub group_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct KafkaGetConsumerLagRequest {
    pub cluster_id: String,
    pub group_id: String,
}

#[tool_box]
impl QueueTools {
    #[cfg(feature = "sqs")]
    #[rmcp::tool(description = "Send a message to an Amazon SQS queue")]
    pub async fn sqs_send_message(
        &self,
        #[rmcp::tool(aggr)] req: SqsSendMessageRequest,
    ) -> Result<String, String> {
        let client = self.sqs.as_ref()
            .ok_or_else(|| "SQS client not configured".to_string())?;

        let options = queue::SendMessageOptions {
            delay_seconds: req.delay_seconds,
            message_group_id: req.message_group_id,
            message_deduplication_id: req.message_deduplication_id,
            ..Default::default()
        };

        let message_id = client.send_message(&req.queue_url, &req.message_body, &options).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "message_id": message_id
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "sqs")]
    #[rmcp::tool(description = "Receive messages from an Amazon SQS queue")]
    pub async fn sqs_receive_messages(
        &self,
        #[rmcp::tool(aggr)] req: SqsReceiveMessagesRequest,
    ) -> Result<String, String> {
        let client = self.sqs.as_ref()
            .ok_or_else(|| "SQS client not configured".to_string())?;

        let options = queue::ReceiveMessageOptions {
            max_number_of_messages: req.max_number_of_messages,
            visibility_timeout: req.visibility_timeout,
            wait_time_seconds: req.wait_time_seconds,
            ..Default::default()
        };

        let messages = client.receive_messages(&req.queue_url, &options).await
            .map_err(|e| e.to_string())?;

        let result: Vec<_> = messages.iter().map(|m| {
            serde_json::json!({
                "id": m.id,
                "body": m.body,
                "receipt_handle": m.receipt_handle,
                "attributes": m.attributes
            })
        }).collect();

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "sqs")]
    #[rmcp::tool(description = "Delete a message from an Amazon SQS queue")]
    pub async fn sqs_delete_message(
        &self,
        #[rmcp::tool(aggr)] req: SqsDeleteMessageRequest,
    ) -> Result<String, String> {
        let client = self.sqs.as_ref()
            .ok_or_else(|| "SQS client not configured".to_string())?;

        client.delete_message(&req.queue_url, &req.receipt_handle).await
            .map_err(|e| e.to_string())?;

        Ok("Message deleted successfully".to_string())
    }

    #[cfg(feature = "sqs")]
    #[rmcp::tool(description = "Get attributes of an Amazon SQS queue")]
    pub async fn sqs_get_queue_attributes(
        &self,
        #[rmcp::tool(aggr)] req: SqsGetQueueAttributesRequest,
    ) -> Result<String, String> {
        let client = self.sqs.as_ref()
            .ok_or_else(|| "SQS client not configured".to_string())?;

        let attrs = client.get_queue_attributes(&req.queue_url).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "approximate_number_of_messages": attrs.approximate_number_of_messages,
            "approximate_number_of_messages_not_visible": attrs.approximate_number_of_messages_not_visible,
            "approximate_number_of_messages_delayed": attrs.approximate_number_of_messages_delayed,
            "visibility_timeout": attrs.visibility_timeout,
            "maximum_message_size": attrs.maximum_message_size,
            "message_retention_period": attrs.message_retention_period,
            "delay_seconds": attrs.delay_seconds,
            "fifo_queue": attrs.fifo_queue
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "sqs")]
    #[rmcp::tool(description = "Purge all messages from an Amazon SQS queue")]
    pub async fn sqs_purge_queue(
        &self,
        #[rmcp::tool(aggr)] req: SqsPurgeQueueRequest,
    ) -> Result<String, String> {
        let client = self.sqs.as_ref()
            .ok_or_else(|| "SQS client not configured".to_string())?;

        client.purge_queue(&req.queue_url).await
            .map_err(|e| e.to_string())?;

        Ok("Queue purged successfully".to_string())
    }

    #[cfg(feature = "sqs")]
    #[rmcp::tool(description = "Change the visibility timeout of an SQS message")]
    pub async fn sqs_change_message_visibility(
        &self,
        #[rmcp::tool(aggr)] req: SqsChangeVisibilityRequest,
    ) -> Result<String, String> {
        let client = self.sqs.as_ref()
            .ok_or_else(|| "SQS client not configured".to_string())?;

        client.change_message_visibility(&req.queue_url, &req.receipt_handle, req.visibility_timeout).await
            .map_err(|e| e.to_string())?;

        Ok("Visibility timeout updated successfully".to_string())
    }

    #[cfg(feature = "kafka")]
    #[rmcp::tool(description = "Produce a message to a Kafka topic")]
    pub async fn kafka_produce(
        &self,
        #[rmcp::tool(aggr)] req: KafkaProduceRequest,
    ) -> Result<String, String> {
        let client = self.kafka.as_ref()
            .ok_or_else(|| "Kafka client not configured".to_string())?;

        let mut record = queue::kafka::ProduceRecord::new(req.value);
        if let Some(key) = req.key {
            record = record.with_key(key);
        }
        if let Some(partition) = req.partition {
            record = record.with_partition(partition);
        }

        let request = queue::kafka::ProduceRequest::new().with_record(record);
        let response = client.produce(&req.topic, &request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "offsets": response.offsets.iter().map(|o| {
                serde_json::json!({
                    "partition": o.partition,
                    "offset": o.offset
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "kafka")]
    #[rmcp::tool(description = "Produce multiple messages to a Kafka topic in a batch")]
    pub async fn kafka_produce_batch(
        &self,
        #[rmcp::tool(aggr)] req: KafkaProduceBatchRequest,
    ) -> Result<String, String> {
        let client = self.kafka.as_ref()
            .ok_or_else(|| "Kafka client not configured".to_string())?;

        let records: Vec<queue::kafka::ProduceRecord> = req.messages
            .into_iter()
            .map(|m| {
                let mut record = queue::kafka::ProduceRecord::new(m.value);
                if let Some(key) = m.key {
                    record = record.with_key(key);
                }
                if let Some(partition) = m.partition {
                    record = record.with_partition(partition);
                }
                record
            })
            .collect();

        let request = queue::kafka::ProduceRequest::new().with_records(records);
        let response = client.produce(&req.topic, &request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "offsets": response.offsets.iter().map(|o| {
                serde_json::json!({
                    "partition": o.partition,
                    "offset": o.offset
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "kafka")]
    #[rmcp::tool(description = "List all Kafka topics")]
    pub async fn kafka_list_topics(
        &self,
        #[rmcp::tool(aggr)] _req: KafkaListTopicsRequest,
    ) -> Result<String, String> {
        let client = self.kafka.as_ref()
            .ok_or_else(|| "Kafka client not configured".to_string())?;

        let topics = client.list_topics().await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "topics": topics
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "kafka")]
    #[rmcp::tool(description = "Get details of a Kafka topic")]
    pub async fn kafka_get_topic(
        &self,
        #[rmcp::tool(aggr)] req: KafkaGetTopicRequest,
    ) -> Result<String, String> {
        let client = self.kafka.as_ref()
            .ok_or_else(|| "Kafka client not configured".to_string())?;

        let topic = client.get_topic(&req.cluster_id, &req.topic_name).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "topic_name": topic.topic_name,
            "cluster_id": topic.cluster_id,
            "is_internal": topic.is_internal,
            "replication_factor": topic.replication_factor,
            "partitions_count": topic.partitions_count
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "kafka")]
    #[rmcp::tool(description = "Create a new Kafka topic")]
    pub async fn kafka_create_topic(
        &self,
        #[rmcp::tool(aggr)] req: KafkaCreateTopicRequest,
    ) -> Result<String, String> {
        let client = self.kafka.as_ref()
            .ok_or_else(|| "Kafka client not configured".to_string())?;

        let mut create_req = queue::kafka::CreateTopicRequest::new(&req.topic_name);
        if let Some(p) = req.partitions {
            create_req = create_req.with_partitions(p);
        }
        if let Some(rf) = req.replication_factor {
            create_req = create_req.with_replication_factor(rf);
        }
        if let Some(retention) = req.retention_ms {
            create_req = create_req.with_retention_ms(retention);
        }

        let topic = client.create_topic(&req.cluster_id, &create_req).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "topic_name": topic.topic_name,
            "cluster_id": topic.cluster_id,
            "partitions_count": topic.partitions_count,
            "replication_factor": topic.replication_factor
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "kafka")]
    #[rmcp::tool(description = "Delete a Kafka topic")]
    pub async fn kafka_delete_topic(
        &self,
        #[rmcp::tool(aggr)] req: KafkaDeleteTopicRequest,
    ) -> Result<String, String> {
        let client = self.kafka.as_ref()
            .ok_or_else(|| "Kafka client not configured".to_string())?;

        client.delete_topic(&req.cluster_id, &req.topic_name).await
            .map_err(|e| e.to_string())?;

        Ok(format!("Topic '{}' deleted successfully", req.topic_name))
    }

    #[cfg(feature = "kafka")]
    #[rmcp::tool(description = "List all Kafka consumer groups")]
    pub async fn kafka_list_consumer_groups(
        &self,
        #[rmcp::tool(aggr)] req: KafkaListConsumerGroupsRequest,
    ) -> Result<String, String> {
        let client = self.kafka.as_ref()
            .ok_or_else(|| "Kafka client not configured".to_string())?;

        let groups = client.list_consumer_groups(&req.cluster_id).await
            .map_err(|e| e.to_string())?;

        let result: Vec<_> = groups.data.iter().map(|g| {
            serde_json::json!({
                "consumer_group_id": g.consumer_group_id,
                "state": g.state,
                "partition_assignor": g.partition_assignor
            })
        }).collect();

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "kafka")]
    #[rmcp::tool(description = "Get details of a Kafka consumer group")]
    pub async fn kafka_get_consumer_group(
        &self,
        #[rmcp::tool(aggr)] req: KafkaGetConsumerGroupRequest,
    ) -> Result<String, String> {
        let client = self.kafka.as_ref()
            .ok_or_else(|| "Kafka client not configured".to_string())?;

        let group = client.get_consumer_group(&req.cluster_id, &req.group_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "consumer_group_id": group.consumer_group_id,
            "cluster_id": group.cluster_id,
            "state": group.state,
            "partition_assignor": group.partition_assignor,
            "is_simple": group.is_simple
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "kafka")]
    #[rmcp::tool(description = "Get consumer lag summary for a Kafka consumer group")]
    pub async fn kafka_get_consumer_lag(
        &self,
        #[rmcp::tool(aggr)] req: KafkaGetConsumerLagRequest,
    ) -> Result<String, String> {
        let client = self.kafka.as_ref()
            .ok_or_else(|| "Kafka client not configured".to_string())?;

        let lag = client.get_consumer_lag_summary(&req.cluster_id, &req.group_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "consumer_group_id": lag.consumer_group_id,
            "total_lag": lag.total_lag,
            "max_lag": lag.max_lag,
            "max_lag_topic": lag.max_lag_topic_name,
            "max_lag_partition": lag.max_lag_partition_id
        })).map_err(|e| e.to_string())
    }
}
