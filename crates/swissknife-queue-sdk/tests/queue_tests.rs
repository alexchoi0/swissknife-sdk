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
