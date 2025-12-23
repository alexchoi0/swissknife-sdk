use crate::{DeleteResult, Document, DocumentDatabaseProvider, FindOptions, InsertResult, Result, UpdateResult};
use async_trait::async_trait;

pub struct MongoClient {
    base_url: String,
    api_key: String,
    data_source: String,
    database: String,
}

impl MongoClient {
    pub fn new(base_url: &str, api_key: &str, data_source: &str, database: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: api_key.to_string(),
            data_source: data_source.to_string(),
            database: database.to_string(),
        }
    }

    async fn request(&self, action: &str, body: serde_json::Value) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();

        let url = format!("{}/action/{}", self.base_url, action);

        let mut request_body = body;
        request_body["dataSource"] = serde_json::json!(self.data_source);
        request_body["database"] = serde_json::json!(self.database);

        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("api-key", &self.api_key)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| crate::Error::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(crate::Error::Query(error));
        }

        resp.json().await
            .map_err(|e| crate::Error::Query(e.to_string()))
    }
}

#[async_trait]
impl DocumentDatabaseProvider for MongoClient {
    async fn list_collections(&self, _database: Option<&str>) -> Result<Vec<String>> {
        Ok(Vec::new())
    }

    async fn find(&self, collection: &str, options: &FindOptions) -> Result<Vec<Document>> {
        let mut body = serde_json::json!({
            "collection": collection,
        });

        if let Some(ref filter) = options.filter {
            body["filter"] = filter.clone();
        }
        if let Some(ref projection) = options.projection {
            let proj: serde_json::Value = projection.iter()
                .map(|f| (f.clone(), serde_json::json!(1)))
                .collect();
            body["projection"] = proj;
        }
        if let Some(ref sort) = options.sort {
            body["sort"] = sort.clone();
        }
        if let Some(limit) = options.limit {
            body["limit"] = serde_json::json!(limit);
        }
        if let Some(skip) = options.skip {
            body["skip"] = serde_json::json!(skip);
        }

        let result = self.request("find", body).await?;

        let documents: Vec<Document> = result
            .get("documents")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().map(|doc| {
                    Document {
                        id: doc.get("_id").and_then(|v| v.get("$oid")).and_then(|v| v.as_str()).map(String::from),
                        data: doc.clone(),
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(documents)
    }

    async fn find_one(&self, collection: &str, filter: &serde_json::Value) -> Result<Option<Document>> {
        let body = serde_json::json!({
            "collection": collection,
            "filter": filter,
        });

        let result = self.request("findOne", body).await?;

        Ok(result.get("document").map(|doc| {
            Document {
                id: doc.get("_id").and_then(|v| v.get("$oid")).and_then(|v| v.as_str()).map(String::from),
                data: doc.clone(),
            }
        }))
    }

    async fn insert_one(&self, collection: &str, document: &serde_json::Value) -> Result<InsertResult> {
        let body = serde_json::json!({
            "collection": collection,
            "document": document,
        });

        let result = self.request("insertOne", body).await?;

        let inserted_id = result
            .get("insertedId")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(InsertResult {
            inserted_id: inserted_id.clone(),
            inserted_ids: inserted_id.into_iter().collect(),
            inserted_count: 1,
        })
    }

    async fn insert_many(&self, collection: &str, documents: &[serde_json::Value]) -> Result<InsertResult> {
        let body = serde_json::json!({
            "collection": collection,
            "documents": documents,
        });

        let result = self.request("insertMany", body).await?;

        let inserted_ids: Vec<String> = result
            .get("insertedIds")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        Ok(InsertResult {
            inserted_id: inserted_ids.first().cloned(),
            inserted_count: inserted_ids.len() as u64,
            inserted_ids,
        })
    }

    async fn update_one(&self, collection: &str, filter: &serde_json::Value, update: &serde_json::Value, upsert: bool) -> Result<UpdateResult> {
        let body = serde_json::json!({
            "collection": collection,
            "filter": filter,
            "update": update,
            "upsert": upsert,
        });

        let result = self.request("updateOne", body).await?;

        Ok(UpdateResult {
            matched_count: result.get("matchedCount").and_then(|v| v.as_u64()).unwrap_or(0),
            modified_count: result.get("modifiedCount").and_then(|v| v.as_u64()).unwrap_or(0),
            upserted_id: result.get("upsertedId").and_then(|v| v.as_str()).map(String::from),
        })
    }

    async fn update_many(&self, collection: &str, filter: &serde_json::Value, update: &serde_json::Value) -> Result<UpdateResult> {
        let body = serde_json::json!({
            "collection": collection,
            "filter": filter,
            "update": update,
        });

        let result = self.request("updateMany", body).await?;

        Ok(UpdateResult {
            matched_count: result.get("matchedCount").and_then(|v| v.as_u64()).unwrap_or(0),
            modified_count: result.get("modifiedCount").and_then(|v| v.as_u64()).unwrap_or(0),
            upserted_id: None,
        })
    }

    async fn delete_one(&self, collection: &str, filter: &serde_json::Value) -> Result<DeleteResult> {
        let body = serde_json::json!({
            "collection": collection,
            "filter": filter,
        });

        let result = self.request("deleteOne", body).await?;

        Ok(DeleteResult {
            deleted_count: result.get("deletedCount").and_then(|v| v.as_u64()).unwrap_or(0),
        })
    }

    async fn delete_many(&self, collection: &str, filter: &serde_json::Value) -> Result<DeleteResult> {
        let body = serde_json::json!({
            "collection": collection,
            "filter": filter,
        });

        let result = self.request("deleteMany", body).await?;

        Ok(DeleteResult {
            deleted_count: result.get("deletedCount").and_then(|v| v.as_u64()).unwrap_or(0),
        })
    }

    async fn aggregate(&self, collection: &str, pipeline: &[serde_json::Value]) -> Result<Vec<Document>> {
        let body = serde_json::json!({
            "collection": collection,
            "pipeline": pipeline,
        });

        let result = self.request("aggregate", body).await?;

        let documents: Vec<Document> = result
            .get("documents")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter().map(|doc| {
                    Document {
                        id: doc.get("_id").and_then(|v| v.get("$oid")).and_then(|v| v.as_str()).map(String::from),
                        data: doc.clone(),
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(documents)
    }
}
