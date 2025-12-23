use crate::error::Result;
use crate::tool::{get_array_param, get_i64_param, get_object_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_database_sdk::{QueryParams, SqlDatabaseProvider, DocumentDatabaseProvider, KeyValueProvider, FindOptions};

pub struct SqlQueryTool;

impl Default for SqlQueryTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for SqlQueryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "sql_query",
            "SQL Query",
            "Execute a SQL query on a database (Postgres, MySQL, ClickHouse)",
            "database",
        )
        .with_param("connection_string", ParameterSchema::string("Database connection string").required().user_only())
        .with_param("provider", ParameterSchema::string("Database provider: postgres, mysql, clickhouse").required())
        .with_param("query", ParameterSchema::string("SQL query to execute").required())
        .with_param("params", ParameterSchema::array("Query parameters", ParameterSchema::json("Parameter value")))
        .with_output("rows", OutputSchema::array("Query result rows", OutputSchema::json("Row data")))
        .with_output("affected_rows", OutputSchema::number("Number of affected rows"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let connection_string = get_required_string_param(&params, "connection_string")?;
        let provider = get_required_string_param(&params, "provider")?;
        let query = get_required_string_param(&params, "query")?;
        let query_params = get_array_param(&params, "params").unwrap_or_default();

        let mut db_params = QueryParams::new();
        for param in query_params {
            db_params = db_params.bind(param);
        }

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "postgres")]
            "postgres" | "postgresql" => {
                use swissknife_database_sdk::postgres::PostgresClient;
                let client = PostgresClient::new(&connection_string);
                client.query(&query, &db_params).await
            }
            #[cfg(feature = "mysql")]
            "mysql" => {
                use swissknife_database_sdk::mysql::MySqlClient;
                let client = MySqlClient::new(&connection_string);
                client.query(&query, &db_params).await
            }
            #[cfg(feature = "clickhouse")]
            "clickhouse" => {
                use swissknife_database_sdk::clickhouse::ClickHouseClient;
                let client = ClickHouseClient::new(&connection_string, "default");
                client.query(&query, &db_params).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported SQL provider: {}", provider)));
            }
        };

        match result {
            Ok(query_result) => Ok(ToolResponse::success(serde_json::json!({
                "rows": query_result.rows,
                "affected_rows": query_result.affected_rows,
                "columns": query_result.columns.iter().map(|c| serde_json::json!({
                    "name": c.name,
                    "data_type": c.data_type,
                })).collect::<Vec<_>>(),
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Query failed: {}", e))),
        }
    }
}

pub struct MongoFindTool;

impl Default for MongoFindTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for MongoFindTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "mongo_find",
            "MongoDB Find",
            "Find documents in a MongoDB collection",
            "database",
        )
        .with_param("connection_string", ParameterSchema::string("MongoDB connection string").required().user_only())
        .with_param("collection", ParameterSchema::string("Collection name").required())
        .with_param("filter", ParameterSchema::json("Query filter"))
        .with_param("projection", ParameterSchema::array("Fields to include", ParameterSchema::string("Field name")))
        .with_param("sort", ParameterSchema::json("Sort specification"))
        .with_param("limit", ParameterSchema::integer("Maximum documents to return"))
        .with_param("skip", ParameterSchema::integer("Number of documents to skip"))
        .with_output("documents", OutputSchema::array("Matching documents", OutputSchema::json("Document")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let connection_string = get_required_string_param(&params, "connection_string")?;
        let collection = get_required_string_param(&params, "collection")?;
        let filter = get_object_param(&params, "filter").map(|m| serde_json::Value::Object(m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()));
        let projection = get_array_param(&params, "projection").map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect());
        let sort = get_object_param(&params, "sort").map(|m| serde_json::Value::Object(m.iter().map(|(k, v)| (k.clone(), v.clone())).collect()));
        let limit = get_i64_param(&params, "limit").map(|v| v as u32);
        let skip = get_i64_param(&params, "skip").map(|v| v as u32);

        let options = FindOptions {
            filter,
            projection,
            sort,
            limit,
            skip,
        };

        #[cfg(feature = "mongodb")]
        {
            use swissknife_database_sdk::mongodb::MongoClient;
            let client = MongoClient::new(&connection_string);
            match client.find(&collection, &options).await {
                Ok(docs) => Ok(ToolResponse::success(serde_json::json!({
                    "documents": docs.iter().map(|d| &d.data).collect::<Vec<_>>(),
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Find failed: {}", e))),
            }
        }
        #[cfg(not(feature = "mongodb"))]
        {
            let _ = (connection_string, collection, options);
            Ok(ToolResponse::error("MongoDB feature not enabled"))
        }
    }
}

pub struct MongoInsertTool;

impl Default for MongoInsertTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for MongoInsertTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "mongo_insert",
            "MongoDB Insert",
            "Insert documents into a MongoDB collection",
            "database",
        )
        .with_param("connection_string", ParameterSchema::string("MongoDB connection string").required().user_only())
        .with_param("collection", ParameterSchema::string("Collection name").required())
        .with_param("documents", ParameterSchema::array("Documents to insert", ParameterSchema::json("Document")).required())
        .with_output("inserted_ids", OutputSchema::array("IDs of inserted documents", OutputSchema::string("Document ID")))
        .with_output("inserted_count", OutputSchema::number("Number of documents inserted"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let connection_string = get_required_string_param(&params, "connection_string")?;
        let collection = get_required_string_param(&params, "collection")?;
        let documents = get_array_param(&params, "documents").ok_or_else(|| crate::Error::MissingParameter("documents".into()))?;

        #[cfg(feature = "mongodb")]
        {
            use swissknife_database_sdk::mongodb::MongoClient;
            let client = MongoClient::new(&connection_string);
            let docs: Vec<serde_json::Value> = documents.to_vec();
            match client.insert_many(&collection, &docs).await {
                Ok(result) => Ok(ToolResponse::success(serde_json::json!({
                    "inserted_ids": result.inserted_ids,
                    "inserted_count": result.inserted_count,
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Insert failed: {}", e))),
            }
        }
        #[cfg(not(feature = "mongodb"))]
        {
            let _ = (connection_string, collection, documents);
            Ok(ToolResponse::error("MongoDB feature not enabled"))
        }
    }
}

pub struct RedisGetTool;

impl Default for RedisGetTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for RedisGetTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "redis_get",
            "Redis Get",
            "Get a value from Redis by key",
            "database",
        )
        .with_param("connection_string", ParameterSchema::string("Redis connection string").required().user_only())
        .with_param("key", ParameterSchema::string("Key to get").required())
        .with_output("value", OutputSchema::string("Value for the key"))
        .with_output("exists", OutputSchema::boolean("Whether the key exists"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let connection_string = get_required_string_param(&params, "connection_string")?;
        let key = get_required_string_param(&params, "key")?;

        #[cfg(feature = "redis")]
        {
            use swissknife_database_sdk::redis::RedisClient;
            let client = RedisClient::new(&connection_string);
            match client.get(&key).await {
                Ok(Some(value)) => Ok(ToolResponse::success(serde_json::json!({
                    "value": value,
                    "exists": true,
                }))),
                Ok(None) => Ok(ToolResponse::success(serde_json::json!({
                    "value": null,
                    "exists": false,
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Get failed: {}", e))),
            }
        }
        #[cfg(not(feature = "redis"))]
        {
            let _ = (connection_string, key);
            Ok(ToolResponse::error("Redis feature not enabled"))
        }
    }
}

pub struct RedisSetTool;

impl Default for RedisSetTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for RedisSetTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "redis_set",
            "Redis Set",
            "Set a value in Redis",
            "database",
        )
        .with_param("connection_string", ParameterSchema::string("Redis connection string").required().user_only())
        .with_param("key", ParameterSchema::string("Key to set").required())
        .with_param("value", ParameterSchema::string("Value to set").required())
        .with_param("ttl", ParameterSchema::integer("Time-to-live in seconds"))
        .with_output("success", OutputSchema::boolean("Whether the operation succeeded"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let connection_string = get_required_string_param(&params, "connection_string")?;
        let key = get_required_string_param(&params, "key")?;
        let value = get_required_string_param(&params, "value")?;
        let ttl = get_i64_param(&params, "ttl").map(|v| v as u64);

        #[cfg(feature = "redis")]
        {
            use swissknife_database_sdk::redis::RedisClient;
            let client = RedisClient::new(&connection_string);
            match client.set(&key, &value, ttl).await {
                Ok(()) => Ok(ToolResponse::success(serde_json::json!({
                    "success": true,
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Set failed: {}", e))),
            }
        }
        #[cfg(not(feature = "redis"))]
        {
            let _ = (connection_string, key, value, ttl);
            Ok(ToolResponse::error("Redis feature not enabled"))
        }
    }
}

pub struct ElasticsearchSearchTool;

impl Default for ElasticsearchSearchTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ElasticsearchSearchTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "elasticsearch_search",
            "Elasticsearch Search",
            "Search documents in Elasticsearch",
            "database",
        )
        .with_param("connection_string", ParameterSchema::string("Elasticsearch URL").required().user_only())
        .with_param("index", ParameterSchema::string("Index name").required())
        .with_param("query", ParameterSchema::json("Elasticsearch query DSL").required())
        .with_param("size", ParameterSchema::integer("Maximum results").with_default(serde_json::json!(10)))
        .with_param("from", ParameterSchema::integer("Offset for pagination"))
        .with_output("hits", OutputSchema::array("Search hits", OutputSchema::json("Hit")))
        .with_output("total", OutputSchema::number("Total matching documents"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let connection_string = get_required_string_param(&params, "connection_string")?;
        let index = get_required_string_param(&params, "index")?;
        let query = get_object_param(&params, "query").ok_or_else(|| crate::Error::MissingParameter("query".into()))?;
        let size = get_i64_param(&params, "size").map(|v| v as u32);
        let from = get_i64_param(&params, "from").map(|v| v as u32);

        #[cfg(feature = "elasticsearch")]
        {
            use swissknife_database_sdk::elasticsearch::ElasticsearchClient;
            use swissknife_database_sdk::SearchQuery;
            let client = ElasticsearchClient::new(&connection_string);
            let search_query = SearchQuery {
                query: serde_json::Value::Object(query.iter().map(|(k, v)| (k.clone(), v.clone())).collect()),
                from,
                size,
                sort: None,
                highlight: None,
                aggregations: None,
            };
            match client.search(&index, &search_query).await {
                Ok(response) => Ok(ToolResponse::success(serde_json::json!({
                    "hits": response.hits.iter().map(|h| serde_json::json!({
                        "id": h.id,
                        "score": h.score,
                        "source": h.source,
                        "highlights": h.highlights,
                    })).collect::<Vec<_>>(),
                    "total": response.total,
                    "took_ms": response.took_ms,
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Search failed: {}", e))),
            }
        }
        #[cfg(not(feature = "elasticsearch"))]
        {
            let _ = (connection_string, index, query, size, from);
            Ok(ToolResponse::error("Elasticsearch feature not enabled"))
        }
    }
}
