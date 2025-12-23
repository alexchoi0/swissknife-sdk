use crate::{Error, Result, QueryResult, ColumnInfo, TableInfo, SqlDatabaseProvider, QueryParams};
use crate::rds::RdsClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::Utc;

impl RdsClient {
    async fn sign_and_send(&self, path: &str, body: &serde_json::Value) -> Result<serde_json::Value> {
        let body_str = serde_json::to_string(body)?;
        let endpoint = format!("{}{}", self.endpoint(), path);
        let now = Utc::now();
        let amz_date = now.format("%Y%m%dT%H%M%SZ").to_string();
        let date_stamp = now.format("%Y%m%d").to_string();

        let content_type = "application/json";
        let host = format!("rds-data.{}.amazonaws.com", self.region());

        let canonical_headers = format!(
            "content-type:{}\nhost:{}\nx-amz-date:{}\n",
            content_type, host, amz_date
        );
        let signed_headers = "content-type;host;x-amz-date";

        let payload_hash = sha256_hex(&body_str);
        let canonical_request = format!(
            "POST\n{}\n\n{}\n{}\n{}",
            path, canonical_headers, signed_headers, payload_hash
        );

        let algorithm = "AWS4-HMAC-SHA256";
        let credential_scope = format!("{}/{}/rds-data/aws4_request", date_stamp, self.region());
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}",
            algorithm,
            amz_date,
            credential_scope,
            sha256_hex(&canonical_request)
        );

        let k_date = hmac_sha256(format!("AWS4{}", self.secret_access_key()).as_bytes(), date_stamp.as_bytes());
        let k_region = hmac_sha256(&k_date, self.region().as_bytes());
        let k_service = hmac_sha256(&k_region, b"rds-data");
        let k_signing = hmac_sha256(&k_service, b"aws4_request");
        let signature = hex::encode(hmac_sha256(&k_signing, string_to_sign.as_bytes()));

        let authorization = format!(
            "{} Credential={}/{}, SignedHeaders={}, Signature={}",
            algorithm,
            self.access_key_id(),
            credential_scope,
            signed_headers,
            signature
        );

        let mut request = self.client()
            .post(&endpoint)
            .header("Content-Type", content_type)
            .header("Host", host)
            .header("X-Amz-Date", amz_date)
            .header("Authorization", authorization);

        if let Some(token) = self.session_token() {
            request = request.header("X-Amz-Security-Token", token);
        }

        let response = request.body(body_str).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: serde_json::Value = response.json().await?;
        Ok(result)
    }

    pub async fn execute_statement(&self, sql: &str, parameters: Option<Vec<SqlParameter>>) -> Result<ExecuteStatementResponse> {
        let mut body = serde_json::json!({
            "resourceArn": self.resource_arn(),
            "secretArn": self.secret_arn(),
            "sql": sql
        });

        if let Some(db) = self.database() {
            body["database"] = serde_json::json!(db);
        }

        if let Some(params) = parameters {
            body["parameters"] = serde_json::to_value(params)?;
        }

        let result = self.sign_and_send("/Execute", &body).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn batch_execute_statement(&self, sql: &str, parameter_sets: Vec<Vec<SqlParameter>>) -> Result<BatchExecuteStatementResponse> {
        let mut body = serde_json::json!({
            "resourceArn": self.resource_arn(),
            "secretArn": self.secret_arn(),
            "sql": sql,
            "parameterSets": parameter_sets
        });

        if let Some(db) = self.database() {
            body["database"] = serde_json::json!(db);
        }

        let result = self.sign_and_send("/BatchExecute", &body).await?;
        Ok(serde_json::from_value(result)?)
    }

    pub async fn begin_transaction(&self) -> Result<String> {
        let mut body = serde_json::json!({
            "resourceArn": self.resource_arn(),
            "secretArn": self.secret_arn()
        });

        if let Some(db) = self.database() {
            body["database"] = serde_json::json!(db);
        }

        let result = self.sign_and_send("/BeginTransaction", &body).await?;
        let response: BeginTransactionResponse = serde_json::from_value(result)?;
        Ok(response.transaction_id)
    }

    pub async fn commit_transaction(&self, transaction_id: &str) -> Result<String> {
        let body = serde_json::json!({
            "resourceArn": self.resource_arn(),
            "secretArn": self.secret_arn(),
            "transactionId": transaction_id
        });

        let result = self.sign_and_send("/CommitTransaction", &body).await?;
        let response: CommitTransactionResponse = serde_json::from_value(result)?;
        Ok(response.transaction_status)
    }

    pub async fn rollback_transaction(&self, transaction_id: &str) -> Result<String> {
        let body = serde_json::json!({
            "resourceArn": self.resource_arn(),
            "secretArn": self.secret_arn(),
            "transactionId": transaction_id
        });

        let result = self.sign_and_send("/RollbackTransaction", &body).await?;
        let response: RollbackTransactionResponse = serde_json::from_value(result)?;
        Ok(response.transaction_status)
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

#[derive(Debug, Clone, Serialize)]
pub struct SqlParameter {
    pub name: String,
    pub value: FieldValue,
    #[serde(rename = "typeHint", skip_serializing_if = "Option::is_none")]
    pub type_hint: Option<String>,
}

impl SqlParameter {
    pub fn string(name: &str, value: &str) -> Self {
        Self {
            name: name.to_string(),
            value: FieldValue::StringValue(value.to_string()),
            type_hint: None,
        }
    }

    pub fn long(name: &str, value: i64) -> Self {
        Self {
            name: name.to_string(),
            value: FieldValue::LongValue(value),
            type_hint: None,
        }
    }

    pub fn double(name: &str, value: f64) -> Self {
        Self {
            name: name.to_string(),
            value: FieldValue::DoubleValue(value),
            type_hint: None,
        }
    }

    pub fn boolean(name: &str, value: bool) -> Self {
        Self {
            name: name.to_string(),
            value: FieldValue::BooleanValue(value),
            type_hint: None,
        }
    }

    pub fn null(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: FieldValue::IsNull(true),
            type_hint: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldValue {
    StringValue(String),
    LongValue(i64),
    DoubleValue(f64),
    BooleanValue(bool),
    BlobValue(String),
    IsNull(bool),
    ArrayValue { arrayValue: ArrayValue },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArrayValue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub string_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub long_values: Option<Vec<i64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub double_values: Option<Vec<f64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boolean_values: Option<Vec<bool>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ExecuteStatementResponse {
    #[serde(rename = "numberOfRecordsUpdated")]
    pub number_of_records_updated: Option<i64>,
    #[serde(rename = "generatedFields")]
    pub generated_fields: Option<Vec<FieldValue>>,
    pub records: Option<Vec<Vec<FieldValue>>>,
    #[serde(rename = "columnMetadata")]
    pub column_metadata: Option<Vec<ColumnMetadata>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ColumnMetadata {
    pub name: Option<String>,
    #[serde(rename = "typeName")]
    pub type_name: Option<String>,
    pub nullable: Option<i32>,
    pub label: Option<String>,
    #[serde(rename = "schemaName")]
    pub schema_name: Option<String>,
    #[serde(rename = "tableName")]
    pub table_name: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchExecuteStatementResponse {
    #[serde(rename = "updateResults")]
    pub update_results: Vec<UpdateResult>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateResult {
    #[serde(rename = "generatedFields")]
    pub generated_fields: Option<Vec<FieldValue>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BeginTransactionResponse {
    #[serde(rename = "transactionId")]
    pub transaction_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CommitTransactionResponse {
    #[serde(rename = "transactionStatus")]
    pub transaction_status: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RollbackTransactionResponse {
    #[serde(rename = "transactionStatus")]
    pub transaction_status: String,
}

fn field_value_to_json(fv: &FieldValue) -> serde_json::Value {
    match fv {
        FieldValue::StringValue(s) => serde_json::json!(s),
        FieldValue::LongValue(l) => serde_json::json!(l),
        FieldValue::DoubleValue(d) => serde_json::json!(d),
        FieldValue::BooleanValue(b) => serde_json::json!(b),
        FieldValue::BlobValue(b) => serde_json::json!(b),
        FieldValue::IsNull(_) => serde_json::Value::Null,
        FieldValue::ArrayValue { arrayValue } => {
            if let Some(strings) = &arrayValue.string_values {
                serde_json::json!(strings)
            } else if let Some(longs) = &arrayValue.long_values {
                serde_json::json!(longs)
            } else if let Some(doubles) = &arrayValue.double_values {
                serde_json::json!(doubles)
            } else if let Some(bools) = &arrayValue.boolean_values {
                serde_json::json!(bools)
            } else {
                serde_json::Value::Array(vec![])
            }
        }
    }
}

#[async_trait]
impl SqlDatabaseProvider for RdsClient {
    async fn execute(&self, query: &str, params: &QueryParams) -> Result<QueryResult> {
        let parameters: Vec<SqlParameter> = params.params.iter().enumerate()
            .map(|(i, v)| {
                let name = format!("param{}", i + 1);
                match v {
                    serde_json::Value::String(s) => SqlParameter::string(&name, s),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            SqlParameter::long(&name, i)
                        } else if let Some(f) = n.as_f64() {
                            SqlParameter::double(&name, f)
                        } else {
                            SqlParameter::null(&name)
                        }
                    }
                    serde_json::Value::Bool(b) => SqlParameter::boolean(&name, *b),
                    serde_json::Value::Null => SqlParameter::null(&name),
                    _ => SqlParameter::string(&name, &v.to_string()),
                }
            })
            .collect();

        let response = self.execute_statement(query, if parameters.is_empty() { None } else { Some(parameters) }).await?;

        Ok(QueryResult {
            rows: vec![],
            affected_rows: response.number_of_records_updated.map(|n| n as u64),
            columns: vec![],
        })
    }

    async fn query(&self, query: &str, params: &QueryParams) -> Result<QueryResult> {
        let parameters: Vec<SqlParameter> = params.params.iter().enumerate()
            .map(|(i, v)| {
                let name = format!("param{}", i + 1);
                match v {
                    serde_json::Value::String(s) => SqlParameter::string(&name, s),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            SqlParameter::long(&name, i)
                        } else if let Some(f) = n.as_f64() {
                            SqlParameter::double(&name, f)
                        } else {
                            SqlParameter::null(&name)
                        }
                    }
                    serde_json::Value::Bool(b) => SqlParameter::boolean(&name, *b),
                    serde_json::Value::Null => SqlParameter::null(&name),
                    _ => SqlParameter::string(&name, &v.to_string()),
                }
            })
            .collect();

        let response = self.execute_statement(query, if parameters.is_empty() { None } else { Some(parameters) }).await?;

        let columns: Vec<ColumnInfo> = response.column_metadata.as_ref()
            .map(|cols| cols.iter().map(|c| ColumnInfo {
                name: c.name.clone().unwrap_or_default(),
                data_type: c.type_name.clone().unwrap_or_default(),
                nullable: c.nullable.map(|n| n != 0).unwrap_or(true),
            }).collect())
            .unwrap_or_default();

        let rows: Vec<HashMap<String, serde_json::Value>> = response.records.as_ref()
            .map(|records| {
                records.iter().map(|row| {
                    let mut map = HashMap::new();
                    for (i, field) in row.iter().enumerate() {
                        let col_name = columns.get(i).map(|c| c.name.clone()).unwrap_or_else(|| format!("col{}", i));
                        map.insert(col_name, field_value_to_json(field));
                    }
                    map
                }).collect()
            })
            .unwrap_or_default();

        Ok(QueryResult {
            rows,
            affected_rows: response.number_of_records_updated.map(|n| n as u64),
            columns,
        })
    }

    async fn list_tables(&self, schema: Option<&str>) -> Result<Vec<TableInfo>> {
        let sql = match schema {
            Some(s) => format!("SELECT table_name FROM information_schema.tables WHERE table_schema = '{}'", s),
            None => "SELECT table_name FROM information_schema.tables WHERE table_schema = 'public'".to_string(),
        };

        let response = self.execute_statement(&sql, None).await?;

        let tables = response.records.unwrap_or_default().iter()
            .filter_map(|row| {
                row.first().and_then(|f| {
                    if let FieldValue::StringValue(name) = f {
                        Some(TableInfo {
                            name: name.clone(),
                            schema: schema.map(|s| s.to_string()),
                            columns: vec![],
                            primary_key: None,
                            row_count: None,
                        })
                    } else {
                        None
                    }
                })
            })
            .collect();

        Ok(tables)
    }

    async fn describe_table(&self, table: &str, schema: Option<&str>) -> Result<TableInfo> {
        let sql = format!(
            "SELECT column_name, data_type, is_nullable FROM information_schema.columns WHERE table_name = '{}' {}",
            table,
            schema.map(|s| format!("AND table_schema = '{}'", s)).unwrap_or_default()
        );

        let response = self.execute_statement(&sql, None).await?;

        let columns: Vec<ColumnInfo> = response.records.unwrap_or_default().iter()
            .filter_map(|row| {
                let name = match row.get(0) {
                    Some(FieldValue::StringValue(s)) => s.clone(),
                    _ => return None,
                };
                let data_type = match row.get(1) {
                    Some(FieldValue::StringValue(s)) => s.clone(),
                    _ => "unknown".to_string(),
                };
                let nullable = match row.get(2) {
                    Some(FieldValue::StringValue(s)) => s == "YES",
                    _ => true,
                };
                Some(ColumnInfo { name, data_type, nullable })
            })
            .collect();

        Ok(TableInfo {
            name: table.to_string(),
            schema: schema.map(|s| s.to_string()),
            columns,
            primary_key: None,
            row_count: None,
        })
    }

    async fn list_indexes(&self, _table: &str) -> Result<Vec<crate::IndexInfo>> {
        Ok(vec![])
    }
}
