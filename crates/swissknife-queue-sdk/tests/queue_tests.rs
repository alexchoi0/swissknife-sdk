#[cfg(feature = "sqs")]
mod sqs_tests {
    use swissknife_queue_sdk::sqs::{
        SqsClient, QueueAttributes, MessageAttributeValue,
    };
    use std::collections::HashMap;

    #[test]
    fn test_sqs_client_creation() {
        let client = SqsClient::new(
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "us-east-1",
        );
        assert!(true);
    }

    #[test]
    fn test_sqs_client_with_endpoint() {
        let client = SqsClient::with_endpoint(
            "AKIAIOSFODNN7EXAMPLE",
            "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY",
            "us-east-1",
            "http://localhost:4566",
        );
        assert!(true);
    }

    #[test]
    fn test_queue_attributes_builder() {
        let mut attrs = QueueAttributes::default();
        attrs.delay_seconds = Some(5);
        attrs.maximum_message_size = Some(262144);
        attrs.message_retention_period = Some(345600);
        attrs.visibility_timeout = Some(30);
        attrs.receive_message_wait_time_seconds = Some(20);

        assert_eq!(attrs.delay_seconds, Some(5));
        assert_eq!(attrs.maximum_message_size, Some(262144));
        assert_eq!(attrs.message_retention_period, Some(345600));
        assert_eq!(attrs.visibility_timeout, Some(30));
        assert_eq!(attrs.receive_message_wait_time_seconds, Some(20));
    }

    #[test]
    fn test_queue_attributes_fifo() {
        let mut attrs = QueueAttributes::default();
        attrs.fifo_queue = Some(true);
        attrs.content_based_deduplication = Some(true);

        assert_eq!(attrs.fifo_queue, Some(true));
        assert_eq!(attrs.content_based_deduplication, Some(true));
    }

    #[test]
    fn test_message_attribute_string() {
        let attr = MessageAttributeValue::string("test-value");

        assert_eq!(attr.data_type, "String");
        assert_eq!(attr.string_value, Some("test-value".to_string()));
        assert!(attr.binary_value.is_none());
    }

    #[test]
    fn test_message_attribute_number() {
        let attr = MessageAttributeValue::number(42);

        assert_eq!(attr.data_type, "Number");
        assert_eq!(attr.string_value, Some("42".to_string()));
    }

    #[test]
    fn test_message_attribute_binary() {
        let data = vec![0x01, 0x02, 0x03, 0x04];
        let attr = MessageAttributeValue::binary(data.clone());

        assert_eq!(attr.data_type, "Binary");
        assert_eq!(attr.binary_value, Some(data));
        assert!(attr.string_value.is_none());
    }

    #[test]
    fn test_queue_attributes_defaults() {
        let attrs = QueueAttributes::default();

        assert!(attrs.delay_seconds.is_none());
        assert!(attrs.maximum_message_size.is_none());
        assert!(attrs.message_retention_period.is_none());
        assert!(attrs.visibility_timeout.is_none());
        assert!(attrs.fifo_queue.is_none());
    }

    #[test]
    fn test_message_attributes_collection() {
        let mut attrs: HashMap<String, MessageAttributeValue> = HashMap::new();
        attrs.insert("Priority".to_string(), MessageAttributeValue::string("high"));
        attrs.insert("Timestamp".to_string(), MessageAttributeValue::number(1234567890));

        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains_key("Priority"));
        assert!(attrs.contains_key("Timestamp"));
    }
}

mod error_tests {
    use swissknife_queue_sdk::Error;

    #[test]
    fn test_error_display() {
        let api_error = Error::Api {
            message: "Queue not found".to_string(),
            code: Some("AWS.SimpleQueueService.NonExistentQueue".to_string()),
        };

        let error_string = format!("{}", api_error);
        assert!(error_string.contains("Queue not found"));
    }

    #[test]
    fn test_queue_not_found_error() {
        let error = Error::QueueNotFound("my-queue".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("my-queue"));
    }

    #[test]
    fn test_message_not_found_error() {
        let error = Error::MessageNotFound("msg-123".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("msg-123"));
    }

    #[test]
    fn test_invalid_parameter_error() {
        let error = Error::InvalidParameter("MaxNumberOfMessages must be between 1 and 10".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("MaxNumberOfMessages"));
    }
}

mod signing_tests {
    #[test]
    fn test_region_parsing() {
        let regions = vec![
            "us-east-1",
            "us-west-2",
            "eu-west-1",
            "ap-southeast-1",
            "sa-east-1",
        ];

        for region in regions {
            assert!(!region.is_empty());
            assert!(region.contains("-"));
        }
    }

    #[test]
    fn test_service_name() {
        let service = "sqs";
        assert_eq!(service, "sqs");
    }

    #[test]
    fn test_aws_date_format() {
        use chrono::Utc;

        let now = Utc::now();
        let date_stamp = now.format("%Y%m%d").to_string();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();

        assert_eq!(date_stamp.len(), 8);
        assert_eq!(amz_date.len(), 16);
        assert!(amz_date.ends_with("Z"));
    }
}

#[cfg(feature = "kafka")]
mod kafka_tests {
    use swissknife_queue_sdk::kafka::{
        KafkaClient, ProduceRequest, ProduceRecord, RecordHeader,
        CreateConsumerRequest, SubscriptionRequest, TopicPartition,
        CreateTopicRequest,
    };

    #[test]
    fn test_kafka_client_creation() {
        let client = KafkaClient::new("http://localhost:8082");
        assert!(true);
    }

    #[test]
    fn test_kafka_client_with_auth() {
        let client = KafkaClient::new("https://pkc-xxxxx.us-east-1.aws.confluent.cloud")
            .with_auth("api-key", "api-secret");
        assert!(true);
    }

    #[test]
    fn test_produce_record_creation() {
        let record = ProduceRecord::new(serde_json::json!({"message": "hello"}));

        assert!(record.key.is_none());
        assert!(record.partition.is_none());
        assert!(record.headers.is_none());
    }

    #[test]
    fn test_produce_record_with_key() {
        let record = ProduceRecord::new(serde_json::json!({"message": "hello"}))
            .with_key(serde_json::json!("user-123"));

        assert!(record.key.is_some());
        assert_eq!(record.key.unwrap(), serde_json::json!("user-123"));
    }

    #[test]
    fn test_produce_record_with_partition() {
        let record = ProduceRecord::new(serde_json::json!({"message": "hello"}))
            .with_partition(3);

        assert_eq!(record.partition, Some(3));
    }

    #[test]
    fn test_produce_record_with_headers() {
        let record = ProduceRecord::new(serde_json::json!({"message": "hello"}))
            .with_header("correlation-id", "abc-123")
            .with_header("source", "test");

        let headers = record.headers.unwrap();
        assert_eq!(headers.len(), 2);
        assert_eq!(headers[0].key, "correlation-id");
        assert_eq!(headers[0].value, "abc-123");
    }

    #[test]
    fn test_produce_request_creation() {
        let request = ProduceRequest::new()
            .with_record(ProduceRecord::new(serde_json::json!({"id": 1})))
            .with_record(ProduceRecord::new(serde_json::json!({"id": 2})));

        assert_eq!(request.records.len(), 2);
    }

    #[test]
    fn test_produce_request_batch() {
        let records = vec![
            ProduceRecord::new(serde_json::json!({"id": 1})),
            ProduceRecord::new(serde_json::json!({"id": 2})),
            ProduceRecord::new(serde_json::json!({"id": 3})),
        ];

        let request = ProduceRequest::new().with_records(records);
        assert_eq!(request.records.len(), 3);
    }

    #[test]
    fn test_create_consumer_request() {
        let request = CreateConsumerRequest::new("my-consumer")
            .earliest()
            .with_auto_commit(false)
            .with_timeout(30000);

        assert_eq!(request.name, "my-consumer");
        assert_eq!(request.auto_offset_reset, Some("earliest".to_string()));
        assert_eq!(request.auto_commit_enable, Some(false));
        assert_eq!(request.consumer_request_timeout_ms, Some(30000));
    }

    #[test]
    fn test_create_consumer_request_latest() {
        let request = CreateConsumerRequest::new("my-consumer").latest();

        assert_eq!(request.auto_offset_reset, Some("latest".to_string()));
    }

    #[test]
    fn test_subscription_request_single() {
        let request = SubscriptionRequest::single("my-topic");

        assert_eq!(request.topics.len(), 1);
        assert_eq!(request.topics[0], "my-topic");
    }

    #[test]
    fn test_subscription_request_multiple() {
        let request = SubscriptionRequest::new(vec![
            "topic-1".to_string(),
            "topic-2".to_string(),
            "topic-3".to_string(),
        ]);

        assert_eq!(request.topics.len(), 3);
    }

    #[test]
    fn test_topic_partition() {
        let tp = TopicPartition {
            topic: "my-topic".to_string(),
            partition: 0,
        };

        assert_eq!(tp.topic, "my-topic");
        assert_eq!(tp.partition, 0);
    }

    #[test]
    fn test_create_topic_request() {
        let request = CreateTopicRequest::new("my-topic")
            .with_partitions(6)
            .with_replication_factor(3)
            .with_retention_ms(86400000);

        assert_eq!(request.topic_name, "my-topic");
        assert_eq!(request.partitions_count, Some(6));
        assert_eq!(request.replication_factor, Some(3));
        assert!(request.configs.is_some());
    }

    #[test]
    fn test_create_topic_compacted() {
        let request = CreateTopicRequest::new("my-compacted-topic")
            .with_partitions(3)
            .compacted();

        let configs = request.configs.unwrap();
        assert!(configs.iter().any(|c| c.name == "cleanup.policy" && c.value == "compact"));
    }

    #[test]
    fn test_record_header() {
        let header = RecordHeader {
            key: "trace-id".to_string(),
            value: "xyz-789".to_string(),
        };

        assert_eq!(header.key, "trace-id");
        assert_eq!(header.value, "xyz-789");
    }
}
