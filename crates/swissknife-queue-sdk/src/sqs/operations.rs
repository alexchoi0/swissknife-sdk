use crate::{Error, Message, MessageAttribute, QueueAttributes, ReceiveMessageOptions, Result, SendMessageOptions};
use crate::sqs::SqsClient;
use crate::sqs::signing::sign_request;
use serde::Deserialize;
use std::collections::HashMap;

impl SqsClient {
    pub async fn create_queue(&self, queue_name: &str, attributes: Option<HashMap<String, String>>) -> Result<String> {
        let mut params = vec![
            ("Action", "CreateQueue".to_string()),
            ("QueueName", queue_name.to_string()),
            ("Version", "2012-11-05".to_string()),
        ];

        if let Some(attrs) = attributes {
            for (i, (key, value)) in attrs.iter().enumerate() {
                params.push(("Attribute.{}.Name", key.clone()));
                params.push(("Attribute.{}.Value", value.clone()));
            }
        }

        let response = self.make_request(&params).await?;
        let result: CreateQueueResponse = quick_xml_parse(&response)?;
        Ok(result.create_queue_result.queue_url)
    }

    pub async fn delete_queue(&self, queue_url: &str) -> Result<()> {
        let params = vec![
            ("Action", "DeleteQueue".to_string()),
            ("QueueUrl", queue_url.to_string()),
            ("Version", "2012-11-05".to_string()),
        ];

        self.make_request(&params).await?;
        Ok(())
    }

    pub async fn list_queues(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let mut params = vec![
            ("Action", "ListQueues".to_string()),
            ("Version", "2012-11-05".to_string()),
        ];

        if let Some(p) = prefix {
            params.push(("QueueNamePrefix", p.to_string()));
        }

        let response = self.make_request(&params).await?;
        let result: ListQueuesResponse = quick_xml_parse(&response)?;
        Ok(result.list_queues_result.queue_urls.unwrap_or_default())
    }

    pub async fn get_queue_url(&self, queue_name: &str) -> Result<String> {
        let params = vec![
            ("Action", "GetQueueUrl".to_string()),
            ("QueueName", queue_name.to_string()),
            ("Version", "2012-11-05".to_string()),
        ];

        let response = self.make_request(&params).await?;
        let result: GetQueueUrlResponse = quick_xml_parse(&response)?;
        Ok(result.get_queue_url_result.queue_url)
    }

    pub async fn send_message(&self, queue_url: &str, body: &str, options: Option<SendMessageOptions>) -> Result<SendMessageResult> {
        let mut params = vec![
            ("Action", "SendMessage".to_string()),
            ("QueueUrl", queue_url.to_string()),
            ("MessageBody", body.to_string()),
            ("Version", "2012-11-05".to_string()),
        ];

        if let Some(opts) = options {
            if let Some(delay) = opts.delay_seconds {
                params.push(("DelaySeconds", delay.to_string()));
            }
            if let Some(dedup_id) = opts.message_deduplication_id {
                params.push(("MessageDeduplicationId", dedup_id));
            }
            if let Some(group_id) = opts.message_group_id {
                params.push(("MessageGroupId", group_id));
            }
            for (i, (name, attr)) in opts.message_attributes.iter().enumerate() {
                let n = i + 1;
                params.push((&format!("MessageAttribute.{}.Name", n), name.clone()));
                params.push((&format!("MessageAttribute.{}.Value.DataType", n), attr.data_type.clone()));
                if let Some(ref sv) = attr.string_value {
                    params.push((&format!("MessageAttribute.{}.Value.StringValue", n), sv.clone()));
                }
            }
        }

        let response = self.make_request(&params).await?;
        let result: SendMessageResponse = quick_xml_parse(&response)?;
        Ok(result.send_message_result)
    }

    pub async fn receive_messages(&self, queue_url: &str, options: Option<ReceiveMessageOptions>) -> Result<Vec<Message>> {
        let mut params = vec![
            ("Action", "ReceiveMessage".to_string()),
            ("QueueUrl", queue_url.to_string()),
            ("Version", "2012-11-05".to_string()),
        ];

        if let Some(opts) = options {
            if let Some(max) = opts.max_number_of_messages {
                params.push(("MaxNumberOfMessages", max.to_string()));
            }
            if let Some(vis) = opts.visibility_timeout {
                params.push(("VisibilityTimeout", vis.to_string()));
            }
            if let Some(wait) = opts.wait_time_seconds {
                params.push(("WaitTimeSeconds", wait.to_string()));
            }
            for (i, attr) in opts.attribute_names.iter().enumerate() {
                params.push((&format!("AttributeName.{}", i + 1), attr.clone()));
            }
            for (i, attr) in opts.message_attribute_names.iter().enumerate() {
                params.push((&format!("MessageAttributeName.{}", i + 1), attr.clone()));
            }
        }

        let response = self.make_request(&params).await?;
        let result: ReceiveMessageResponse = quick_xml_parse(&response)?;

        let messages = result.receive_message_result.messages.unwrap_or_default()
            .into_iter()
            .map(|m| Message {
                id: m.message_id,
                body: m.body,
                receipt_handle: Some(m.receipt_handle),
                attributes: m.attributes.unwrap_or_default(),
                message_attributes: HashMap::new(),
                md5_of_body: Some(m.md5_of_body),
            })
            .collect();

        Ok(messages)
    }

    pub async fn delete_message(&self, queue_url: &str, receipt_handle: &str) -> Result<()> {
        let params = vec![
            ("Action", "DeleteMessage".to_string()),
            ("QueueUrl", queue_url.to_string()),
            ("ReceiptHandle", receipt_handle.to_string()),
            ("Version", "2012-11-05".to_string()),
        ];

        self.make_request(&params).await?;
        Ok(())
    }

    pub async fn change_message_visibility(&self, queue_url: &str, receipt_handle: &str, visibility_timeout: u32) -> Result<()> {
        let params = vec![
            ("Action", "ChangeMessageVisibility".to_string()),
            ("QueueUrl", queue_url.to_string()),
            ("ReceiptHandle", receipt_handle.to_string()),
            ("VisibilityTimeout", visibility_timeout.to_string()),
            ("Version", "2012-11-05".to_string()),
        ];

        self.make_request(&params).await?;
        Ok(())
    }

    pub async fn purge_queue(&self, queue_url: &str) -> Result<()> {
        let params = vec![
            ("Action", "PurgeQueue".to_string()),
            ("QueueUrl", queue_url.to_string()),
            ("Version", "2012-11-05".to_string()),
        ];

        self.make_request(&params).await?;
        Ok(())
    }

    pub async fn get_queue_attributes(&self, queue_url: &str, attribute_names: Option<Vec<String>>) -> Result<HashMap<String, String>> {
        let mut params = vec![
            ("Action", "GetQueueAttributes".to_string()),
            ("QueueUrl", queue_url.to_string()),
            ("Version", "2012-11-05".to_string()),
        ];

        let attrs = attribute_names.unwrap_or_else(|| vec!["All".to_string()]);
        for (i, attr) in attrs.iter().enumerate() {
            params.push((&format!("AttributeName.{}", i + 1), attr.clone()));
        }

        let response = self.make_request(&params).await?;
        let result: GetQueueAttributesResponse = quick_xml_parse(&response)?;

        let mut attributes = HashMap::new();
        if let Some(attrs) = result.get_queue_attributes_result.attributes {
            for attr in attrs {
                attributes.insert(attr.name, attr.value);
            }
        }

        Ok(attributes)
    }

    pub async fn set_queue_attributes(&self, queue_url: &str, attributes: HashMap<String, String>) -> Result<()> {
        let mut params = vec![
            ("Action", "SetQueueAttributes".to_string()),
            ("QueueUrl", queue_url.to_string()),
            ("Version", "2012-11-05".to_string()),
        ];

        for (i, (key, value)) in attributes.iter().enumerate() {
            let n = i + 1;
            params.push((&format!("Attribute.{}.Name", n), key.clone()));
            params.push((&format!("Attribute.{}.Value", n), value.clone()));
        }

        self.make_request(&params).await?;
        Ok(())
    }

    async fn make_request(&self, params: &[(&str, String)]) -> Result<String> {
        let host = format!("sqs.{}.amazonaws.com", self.region());
        let uri = "/";

        let query_string: String = params
            .iter()
            .map(|(k, v)| format!("{}={}", urlencoding::encode(k), urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");

        let (auth_header, amz_date, _) = sign_request(
            "POST",
            &host,
            uri,
            "",
            &query_string,
            self.access_key_id(),
            self.secret_access_key(),
            self.region(),
            "sqs",
        );

        let response = self.client()
            .post(&self.endpoint())
            .header("Host", &host)
            .header("X-Amz-Date", &amz_date)
            .header("Authorization", &auth_header)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(query_string)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Aws {
                message: text,
                code: Some(status.to_string()),
            });
        }

        response.text().await.map_err(Error::Http)
    }
}

fn quick_xml_parse<T: for<'de> Deserialize<'de>>(xml: &str) -> Result<T> {
    quick_xml::de::from_str(xml).map_err(|e| Error::Aws {
        message: e.to_string(),
        code: None,
    })
}

fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

#[derive(Debug, Deserialize)]
#[serde(rename = "CreateQueueResponse")]
struct CreateQueueResponse {
    #[serde(rename = "CreateQueueResult")]
    create_queue_result: CreateQueueResult,
}

#[derive(Debug, Deserialize)]
struct CreateQueueResult {
    #[serde(rename = "QueueUrl")]
    queue_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "ListQueuesResponse")]
struct ListQueuesResponse {
    #[serde(rename = "ListQueuesResult")]
    list_queues_result: ListQueuesResult,
}

#[derive(Debug, Deserialize)]
struct ListQueuesResult {
    #[serde(rename = "QueueUrl")]
    queue_urls: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "GetQueueUrlResponse")]
struct GetQueueUrlResponse {
    #[serde(rename = "GetQueueUrlResult")]
    get_queue_url_result: GetQueueUrlResult,
}

#[derive(Debug, Deserialize)]
struct GetQueueUrlResult {
    #[serde(rename = "QueueUrl")]
    queue_url: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "SendMessageResponse")]
struct SendMessageResponse {
    #[serde(rename = "SendMessageResult")]
    send_message_result: SendMessageResult,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SendMessageResult {
    #[serde(rename = "MessageId")]
    pub message_id: String,
    #[serde(rename = "MD5OfMessageBody")]
    pub md5_of_message_body: String,
    #[serde(rename = "SequenceNumber")]
    pub sequence_number: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "ReceiveMessageResponse")]
struct ReceiveMessageResponse {
    #[serde(rename = "ReceiveMessageResult")]
    receive_message_result: ReceiveMessageResult,
}

#[derive(Debug, Deserialize)]
struct ReceiveMessageResult {
    #[serde(rename = "Message")]
    messages: Option<Vec<SqsMessage>>,
}

#[derive(Debug, Deserialize)]
struct SqsMessage {
    #[serde(rename = "MessageId")]
    message_id: String,
    #[serde(rename = "ReceiptHandle")]
    receipt_handle: String,
    #[serde(rename = "MD5OfBody")]
    md5_of_body: String,
    #[serde(rename = "Body")]
    body: String,
    #[serde(rename = "Attribute")]
    attributes: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "GetQueueAttributesResponse")]
struct GetQueueAttributesResponse {
    #[serde(rename = "GetQueueAttributesResult")]
    get_queue_attributes_result: GetQueueAttributesResult,
}

#[derive(Debug, Deserialize)]
struct GetQueueAttributesResult {
    #[serde(rename = "Attribute")]
    attributes: Option<Vec<QueueAttribute>>,
}

#[derive(Debug, Deserialize)]
struct QueueAttribute {
    #[serde(rename = "Name")]
    name: String,
    #[serde(rename = "Value")]
    value: String,
}
