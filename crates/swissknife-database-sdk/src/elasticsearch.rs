use crate::{Result, SearchDatabaseProvider, SearchHit, SearchQuery, SearchResponse};
use async_trait::async_trait;

pub struct ElasticsearchClient {
    base_url: String,
    api_key: Option<String>,
    username: Option<String>,
    password: Option<String>,
}

impl ElasticsearchClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key: None,
            username: None,
            password: None,
        }
    }

    pub fn with_api_key(mut self, api_key: &str) -> Self {
        self.api_key = Some(api_key.to_string());
        self
    }

    pub fn with_basic_auth(mut self, username: &str, password: &str) -> Self {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self
    }

    fn build_client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }

    async fn request(&self, method: reqwest::Method, path: &str, body: Option<serde_json::Value>) -> Result<serde_json::Value> {
        let client = self.build_client();
        let url = format!("{}/{}", self.base_url, path.trim_start_matches('/'));

        let mut request = client.request(method, &url)
            .header("Content-Type", "application/json");

        if let Some(ref api_key) = self.api_key {
            request = request.header("Authorization", format!("ApiKey {}", api_key));
        } else if let (Some(ref username), Some(ref password)) = (&self.username, &self.password) {
            request = request.basic_auth(username, Some(password));
        }

        if let Some(b) = body {
            request = request.json(&b);
        }

        let resp = request.send().await
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
impl SearchDatabaseProvider for ElasticsearchClient {
    async fn list_indices(&self) -> Result<Vec<String>> {
        let result = self.request(reqwest::Method::GET, "_cat/indices?format=json", None).await?;

        let indices: Vec<String> = result
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.get("index").and_then(|i| i.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(indices)
    }

    async fn create_index(&self, name: &str, mappings: &serde_json::Value) -> Result<()> {
        let body = serde_json::json!({
            "mappings": mappings,
        });

        self.request(reqwest::Method::PUT, name, Some(body)).await?;
        Ok(())
    }

    async fn delete_index(&self, name: &str) -> Result<()> {
        self.request(reqwest::Method::DELETE, name, None).await?;
        Ok(())
    }

    async fn index_document(&self, index: &str, id: Option<&str>, document: &serde_json::Value) -> Result<String> {
        let path = match id {
            Some(doc_id) => format!("{}/_doc/{}", index, doc_id),
            None => format!("{}/_doc", index),
        };

        let method = if id.is_some() {
            reqwest::Method::PUT
        } else {
            reqwest::Method::POST
        };

        let result = self.request(method, &path, Some(document.clone())).await?;

        let doc_id = result
            .get("_id")
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_default();

        Ok(doc_id)
    }

    async fn bulk_index(&self, index: &str, documents: &[serde_json::Value]) -> Result<u64> {
        let mut ndjson = String::new();

        for doc in documents {
            let action = serde_json::json!({"index": {"_index": index}});
            ndjson.push_str(&serde_json::to_string(&action).unwrap());
            ndjson.push('\n');
            ndjson.push_str(&serde_json::to_string(doc).unwrap());
            ndjson.push('\n');
        }

        let client = self.build_client();
        let url = format!("{}/_bulk", self.base_url);

        let mut request = client.post(&url)
            .header("Content-Type", "application/x-ndjson")
            .body(ndjson);

        if let Some(ref api_key) = self.api_key {
            request = request.header("Authorization", format!("ApiKey {}", api_key));
        } else if let (Some(ref username), Some(ref password)) = (&self.username, &self.password) {
            request = request.basic_auth(username, Some(password));
        }

        let resp = request.send().await
            .map_err(|e| crate::Error::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(crate::Error::Query(error));
        }

        let result: serde_json::Value = resp.json().await
            .map_err(|e| crate::Error::Query(e.to_string()))?;

        let items = result.get("items").and_then(|v| v.as_array()).map(|arr| arr.len()).unwrap_or(0);

        Ok(items as u64)
    }

    async fn search(&self, index: &str, query: &SearchQuery) -> Result<SearchResponse> {
        let mut body = serde_json::json!({
            "query": query.query,
        });

        if let Some(from) = query.from {
            body["from"] = serde_json::json!(from);
        }
        if let Some(size) = query.size {
            body["size"] = serde_json::json!(size);
        }
        if let Some(ref sort) = query.sort {
            body["sort"] = serde_json::json!(sort);
        }
        if let Some(ref highlight) = query.highlight {
            body["highlight"] = highlight.clone();
        }
        if let Some(ref aggs) = query.aggregations {
            body["aggs"] = aggs.clone();
        }

        let path = format!("{}/_search", index);
        let result = self.request(reqwest::Method::POST, &path, Some(body)).await?;

        let took = result.get("took").and_then(|v| v.as_u64()).unwrap_or(0);

        let total = result
            .get("hits")
            .and_then(|h| h.get("total"))
            .and_then(|t| {
                if t.is_object() {
                    t.get("value").and_then(|v| v.as_u64())
                } else {
                    t.as_u64()
                }
            })
            .unwrap_or(0);

        let hits: Vec<SearchHit> = result
            .get("hits")
            .and_then(|h| h.get("hits"))
            .and_then(|h| h.as_array())
            .map(|arr| {
                arr.iter().map(|hit| {
                    SearchHit {
                        id: hit.get("_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                        score: hit.get("_score").and_then(|v| v.as_f64()).unwrap_or(0.0) as f32,
                        source: hit.get("_source").cloned().unwrap_or(serde_json::Value::Null),
                        highlights: hit.get("highlight").and_then(|h| {
                            h.as_object().map(|obj| {
                                obj.iter().map(|(k, v)| {
                                    let highlights: Vec<String> = v.as_array()
                                        .map(|arr| arr.iter().filter_map(|s| s.as_str().map(String::from)).collect())
                                        .unwrap_or_default();
                                    (k.clone(), highlights)
                                }).collect()
                            })
                        }),
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(SearchResponse {
            hits,
            total,
            took_ms: took,
        })
    }

    async fn get_document(&self, index: &str, id: &str) -> Result<Option<serde_json::Value>> {
        let path = format!("{}/_doc/{}", index, id);

        match self.request(reqwest::Method::GET, &path, None).await {
            Ok(result) => {
                if result.get("found").and_then(|v| v.as_bool()).unwrap_or(false) {
                    Ok(result.get("_source").cloned())
                } else {
                    Ok(None)
                }
            }
            Err(_) => Ok(None),
        }
    }

    async fn delete_document(&self, index: &str, id: &str) -> Result<bool> {
        let path = format!("{}/_doc/{}", index, id);
        let result = self.request(reqwest::Method::DELETE, &path, None).await?;

        Ok(result.get("result").and_then(|v| v.as_str()).map(|r| r == "deleted").unwrap_or(false))
    }
}
