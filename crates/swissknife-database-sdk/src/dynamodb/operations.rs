use crate::{Error, Result, Document, FindOptions, InsertResult, UpdateResult, DeleteResult, DocumentDatabaseProvider};
use crate::dynamodb::DynamoDbClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl DynamoDbClient {
    pub async fn list_tables(&self, limit: Option<i32>) -> Result<ListTablesResponse> {
        let mut body = serde_json::json!({});
        if let Some(l) = limit {
            body["Limit"] = serde_json::json!(l);
        }

        let result = self.sign_and_send("DynamoDB_20120810.ListTables", &body).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn describe_table(&self, table_name: &str) -> Result<DescribeTableResponse> {
        let body = serde_json::json!({
            "TableName": table_name
        });

        let result = self.sign_and_send("DynamoDB_20120810.DescribeTable", &body).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn get_item(&self, table_name: &str, key: HashMap<String, AttributeValue>) -> Result<GetItemResponse> {
        let body = serde_json::json!({
            "TableName": table_name,
            "Key": key
        });

        let result = self.sign_and_send("DynamoDB_20120810.GetItem", &body).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn put_item(&self, table_name: &str, item: HashMap<String, AttributeValue>) -> Result<PutItemResponse> {
        let body = serde_json::json!({
            "TableName": table_name,
            "Item": item
        });

        let result = self.sign_and_send("DynamoDB_20120810.PutItem", &body).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn delete_item(&self, table_name: &str, key: HashMap<String, AttributeValue>) -> Result<DeleteItemResponse> {
        let body = serde_json::json!({
            "TableName": table_name,
            "Key": key
        });

        let result = self.sign_and_send("DynamoDB_20120810.DeleteItem", &body).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn update_item(&self, table_name: &str, key: HashMap<String, AttributeValue>, update_expression: &str, expression_attribute_values: HashMap<String, AttributeValue>) -> Result<UpdateItemResponse> {
        let body = serde_json::json!({
            "TableName": table_name,
            "Key": key,
            "UpdateExpression": update_expression,
            "ExpressionAttributeValues": expression_attribute_values,
            "ReturnValues": "ALL_NEW"
        });

        let result = self.sign_and_send("DynamoDB_20120810.UpdateItem", &body).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn query(&self, request: QueryRequest) -> Result<QueryResponse> {
        let result = self.sign_and_send("DynamoDB_20120810.Query", &serde_json::to_value(&request)?).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn scan(&self, request: ScanRequest) -> Result<ScanResponse> {
        let result = self.sign_and_send("DynamoDB_20120810.Scan", &serde_json::to_value(&request)?).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn batch_get_item(&self, request_items: HashMap<String, BatchGetRequest>) -> Result<BatchGetItemResponse> {
        let body = serde_json::json!({
            "RequestItems": request_items
        });

        let result = self.sign_and_send("DynamoDB_20120810.BatchGetItem", &body).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn batch_write_item(&self, request_items: HashMap<String, Vec<WriteRequest>>) -> Result<BatchWriteItemResponse> {
        let body = serde_json::json!({
            "RequestItems": request_items
        });

        let result = self.sign_and_send("DynamoDB_20120810.BatchWriteItem", &body).await?;
        Ok(serde_json::from_value(result)?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    S { S: String },
    N { N: String },
    B { B: String },
    SS { SS: Vec<String> },
    NS { NS: Vec<String> },
    BS { BS: Vec<String> },
    M { M: HashMap<String, AttributeValue> },
    L { L: Vec<AttributeValue> },
    NULL { NULL: bool },
    BOOL { BOOL: bool },
}

impl AttributeValue {
    pub fn string(s: impl Into<String>) -> Self {
        AttributeValue::S { S: s.into() }
    }

    pub fn number(n: impl ToString) -> Self {
        AttributeValue::N { N: n.to_string() }
    }

    pub fn bool(b: bool) -> Self {
        AttributeValue::BOOL { BOOL: b }
    }

    pub fn null() -> Self {
        AttributeValue::NULL { NULL: true }
    }

    pub fn list(items: Vec<AttributeValue>) -> Self {
        AttributeValue::L { L: items }
    }

    pub fn map(items: HashMap<String, AttributeValue>) -> Self {
        AttributeValue::M { M: items }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct ListTablesResponse {
    #[serde(rename = "TableNames")]
    pub table_names: Vec<String>,
    #[serde(rename = "LastEvaluatedTableName")]
    pub last_evaluated_table_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DescribeTableResponse {
    #[serde(rename = "Table")]
    pub table: TableDescription,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TableDescription {
    #[serde(rename = "TableName")]
    pub table_name: String,
    #[serde(rename = "TableStatus")]
    pub table_status: Option<String>,
    #[serde(rename = "KeySchema")]
    pub key_schema: Option<Vec<KeySchemaElement>>,
    #[serde(rename = "AttributeDefinitions")]
    pub attribute_definitions: Option<Vec<AttributeDefinition>>,
    #[serde(rename = "ItemCount")]
    pub item_count: Option<i64>,
    #[serde(rename = "TableSizeBytes")]
    pub table_size_bytes: Option<i64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KeySchemaElement {
    #[serde(rename = "AttributeName")]
    pub attribute_name: String,
    #[serde(rename = "KeyType")]
    pub key_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttributeDefinition {
    #[serde(rename = "AttributeName")]
    pub attribute_name: String,
    #[serde(rename = "AttributeType")]
    pub attribute_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GetItemResponse {
    #[serde(rename = "Item")]
    pub item: Option<HashMap<String, AttributeValue>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PutItemResponse {
    #[serde(rename = "Attributes")]
    pub attributes: Option<HashMap<String, AttributeValue>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteItemResponse {
    #[serde(rename = "Attributes")]
    pub attributes: Option<HashMap<String, AttributeValue>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateItemResponse {
    #[serde(rename = "Attributes")]
    pub attributes: Option<HashMap<String, AttributeValue>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct QueryRequest {
    #[serde(rename = "TableName")]
    pub table_name: String,
    #[serde(rename = "KeyConditionExpression")]
    pub key_condition_expression: String,
    #[serde(rename = "ExpressionAttributeValues")]
    pub expression_attribute_values: HashMap<String, AttributeValue>,
    #[serde(rename = "Limit", skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(rename = "ScanIndexForward", skip_serializing_if = "Option::is_none")]
    pub scan_index_forward: Option<bool>,
    #[serde(rename = "ExclusiveStartKey", skip_serializing_if = "Option::is_none")]
    pub exclusive_start_key: Option<HashMap<String, AttributeValue>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QueryResponse {
    #[serde(rename = "Items")]
    pub items: Vec<HashMap<String, AttributeValue>>,
    #[serde(rename = "Count")]
    pub count: Option<i32>,
    #[serde(rename = "ScannedCount")]
    pub scanned_count: Option<i32>,
    #[serde(rename = "LastEvaluatedKey")]
    pub last_evaluated_key: Option<HashMap<String, AttributeValue>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ScanRequest {
    #[serde(rename = "TableName")]
    pub table_name: String,
    #[serde(rename = "FilterExpression", skip_serializing_if = "Option::is_none")]
    pub filter_expression: Option<String>,
    #[serde(rename = "ExpressionAttributeValues", skip_serializing_if = "Option::is_none")]
    pub expression_attribute_values: Option<HashMap<String, AttributeValue>>,
    #[serde(rename = "Limit", skip_serializing_if = "Option::is_none")]
    pub limit: Option<i32>,
    #[serde(rename = "ExclusiveStartKey", skip_serializing_if = "Option::is_none")]
    pub exclusive_start_key: Option<HashMap<String, AttributeValue>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScanResponse {
    #[serde(rename = "Items")]
    pub items: Vec<HashMap<String, AttributeValue>>,
    #[serde(rename = "Count")]
    pub count: Option<i32>,
    #[serde(rename = "ScannedCount")]
    pub scanned_count: Option<i32>,
    #[serde(rename = "LastEvaluatedKey")]
    pub last_evaluated_key: Option<HashMap<String, AttributeValue>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BatchGetRequest {
    #[serde(rename = "Keys")]
    pub keys: Vec<HashMap<String, AttributeValue>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchGetItemResponse {
    #[serde(rename = "Responses")]
    pub responses: HashMap<String, Vec<HashMap<String, AttributeValue>>>,
    #[serde(rename = "UnprocessedKeys")]
    pub unprocessed_keys: Option<HashMap<String, BatchGetRequest>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum WriteRequest {
    PutRequest {
        #[serde(rename = "PutRequest")]
        put_request: PutRequest,
    },
    DeleteRequest {
        #[serde(rename = "DeleteRequest")]
        delete_request: DeleteRequest,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct PutRequest {
    #[serde(rename = "Item")]
    pub item: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteRequest {
    #[serde(rename = "Key")]
    pub key: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchWriteItemResponse {
    #[serde(rename = "UnprocessedItems")]
    pub unprocessed_items: Option<HashMap<String, Vec<WriteRequest>>>,
}
