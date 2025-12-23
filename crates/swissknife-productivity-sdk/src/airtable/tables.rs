use crate::{Error, Result, Database, DatabaseEntry, DatabaseProvider, QueryFilter, PropertySchema, PropertyType};
use crate::airtable::AirtableClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl AirtableClient {
    pub async fn list_records(&self, base_id: &str, table_id: &str, params: Option<ListRecordsParams>) -> Result<RecordsResponse> {
        let mut request = self.client()
            .get(format!("{}/{}/{}", self.base_url(), base_id, table_id))
            .header("Authorization", format!("Bearer {}", self.api_key()));

        if let Some(p) = params {
            let mut query_params = Vec::new();
            if let Some(view) = p.view {
                query_params.push(("view", view));
            }
            if let Some(page_size) = p.page_size {
                query_params.push(("pageSize", page_size.to_string()));
            }
            if let Some(offset) = p.offset {
                query_params.push(("offset", offset));
            }
            if !query_params.is_empty() {
                request = request.query(&query_params);
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let records: RecordsResponse = response.json().await?;
        Ok(records)
    }

    pub async fn get_record(&self, base_id: &str, table_id: &str, record_id: &str) -> Result<AirtableRecord> {
        let response = self.client()
            .get(format!("{}/{}/{}/{}", self.base_url(), base_id, table_id, record_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let record: AirtableRecord = response.json().await?;
        Ok(record)
    }

    pub async fn create_records(&self, base_id: &str, table_id: &str, records: Vec<CreateRecord>) -> Result<RecordsResponse> {
        let body = serde_json::json!({
            "records": records.into_iter().map(|r| serde_json::json!({ "fields": r.fields })).collect::<Vec<_>>()
        });

        let response = self.client()
            .post(format!("{}/{}/{}", self.base_url(), base_id, table_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let records: RecordsResponse = response.json().await?;
        Ok(records)
    }

    pub async fn update_records(&self, base_id: &str, table_id: &str, records: Vec<UpdateRecord>) -> Result<RecordsResponse> {
        let body = serde_json::json!({
            "records": records.into_iter().map(|r| serde_json::json!({
                "id": r.id,
                "fields": r.fields
            })).collect::<Vec<_>>()
        });

        let response = self.client()
            .patch(format!("{}/{}/{}", self.base_url(), base_id, table_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let records: RecordsResponse = response.json().await?;
        Ok(records)
    }

    pub async fn delete_records(&self, base_id: &str, table_id: &str, record_ids: &[&str]) -> Result<DeleteRecordsResponse> {
        let params: Vec<(&str, &str)> = record_ids.iter().map(|id| ("records[]", *id)).collect();

        let response = self.client()
            .delete(format!("{}/{}/{}", self.base_url(), base_id, table_id))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .query(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: DeleteRecordsResponse = response.json().await?;
        Ok(result)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListRecordsParams {
    pub view: Option<String>,
    pub page_size: Option<u32>,
    pub offset: Option<String>,
    pub filter_by_formula: Option<String>,
    pub sort: Option<Vec<SortParam>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SortParam {
    pub field: String,
    pub direction: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RecordsResponse {
    pub records: Vec<AirtableRecord>,
    pub offset: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AirtableRecord {
    pub id: String,
    pub fields: HashMap<String, serde_json::Value>,
    #[serde(rename = "createdTime")]
    pub created_time: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateRecord {
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdateRecord {
    pub id: String,
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeleteRecordsResponse {
    pub records: Vec<DeletedRecord>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeletedRecord {
    pub id: String,
    pub deleted: bool,
}

fn map_airtable_field_type(field_type: &str) -> PropertyType {
    match field_type {
        "singleLineText" | "multilineText" | "richText" => PropertyType::RichText,
        "number" | "currency" | "percent" | "duration" => PropertyType::Number,
        "singleSelect" => PropertyType::Select,
        "multipleSelects" => PropertyType::MultiSelect,
        "date" | "dateTime" => PropertyType::Date,
        "checkbox" => PropertyType::Checkbox,
        "url" => PropertyType::Url,
        "email" => PropertyType::Email,
        "phoneNumber" => PropertyType::Phone,
        "formula" => PropertyType::Formula,
        "rollup" => PropertyType::Rollup,
        "multipleRecordLinks" => PropertyType::Relation,
        "createdTime" => PropertyType::CreatedTime,
        "createdBy" => PropertyType::CreatedBy,
        "lastModifiedTime" => PropertyType::LastEditedTime,
        "lastModifiedBy" => PropertyType::LastEditedBy,
        "multipleAttachments" => PropertyType::Files,
        "singleCollaborator" | "multipleCollaborators" => PropertyType::People,
        _ => PropertyType::RichText,
    }
}

pub struct AirtableTableProvider {
    client: AirtableClient,
    base_id: String,
    table_id: String,
}

impl AirtableTableProvider {
    pub fn new(api_key: &str, base_id: &str, table_id: &str) -> Self {
        Self {
            client: AirtableClient::new(api_key),
            base_id: base_id.to_string(),
            table_id: table_id.to_string(),
        }
    }
}

#[async_trait]
impl DatabaseProvider for AirtableTableProvider {
    async fn get_database(&self, _id: &str) -> Result<Database> {
        let schema = self.client.get_base_schema(&self.base_id).await?;
        let table = schema.tables.into_iter()
            .find(|t| t.id == self.table_id || t.name == self.table_id)
            .ok_or_else(|| Error::NotFound(format!("Table {} not found", self.table_id)))?;

        let properties: HashMap<String, PropertySchema> = table.fields.into_iter()
            .map(|f| {
                (f.name.clone(), PropertySchema {
                    name: f.name,
                    property_type: map_airtable_field_type(&f.field_type),
                    options: None,
                })
            })
            .collect();

        Ok(Database {
            id: table.id,
            title: table.name,
            description: None,
            url: Some(format!("https://airtable.com/{}/{}", self.base_id, self.table_id)),
            properties,
            created_at: None,
            updated_at: None,
        })
    }

    async fn query_database(&self, _id: &str, _filter: &QueryFilter) -> Result<Vec<DatabaseEntry>> {
        let response = self.client.list_records(&self.base_id, &self.table_id, None).await?;

        let entries = response.records.into_iter().map(|record| {
            DatabaseEntry {
                id: record.id,
                properties: record.fields,
                created_at: record.created_time
                    .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
                    .map(|dt| dt.with_timezone(&chrono::Utc)),
                updated_at: None,
            }
        }).collect();

        Ok(entries)
    }

    async fn create_entry(&self, _database_id: &str, properties: HashMap<String, serde_json::Value>) -> Result<DatabaseEntry> {
        let records = vec![CreateRecord { fields: properties }];
        let response = self.client.create_records(&self.base_id, &self.table_id, records).await?;

        let record = response.records.into_iter().next()
            .ok_or_else(|| Error::Api { message: "No record created".to_string(), code: None })?;

        Ok(DatabaseEntry {
            id: record.id,
            properties: record.fields,
            created_at: record.created_time
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: None,
        })
    }

    async fn update_entry(&self, entry_id: &str, properties: HashMap<String, serde_json::Value>) -> Result<DatabaseEntry> {
        let records = vec![UpdateRecord {
            id: entry_id.to_string(),
            fields: properties,
        }];
        let response = self.client.update_records(&self.base_id, &self.table_id, records).await?;

        let record = response.records.into_iter().next()
            .ok_or_else(|| Error::Api { message: "No record updated".to_string(), code: None })?;

        Ok(DatabaseEntry {
            id: record.id,
            properties: record.fields,
            created_at: record.created_time
                .and_then(|t| chrono::DateTime::parse_from_rfc3339(&t).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: None,
        })
    }

    async fn delete_entry(&self, entry_id: &str) -> Result<()> {
        self.client.delete_records(&self.base_id, &self.table_id, &[entry_id]).await?;
        Ok(())
    }
}
