use crate::error::Result;
use crate::tool::{get_array_param, get_bool_param, get_i64_param, get_object_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_vectordb_sdk::{DistanceMetric, QueryOptions, UpsertOptions, Vector, VectorDatabaseProvider};

pub struct VectorUpsertTool;

impl Default for VectorUpsertTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for VectorUpsertTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "vector_upsert",
            "Vector Upsert",
            "Insert or update vectors in a vector database (Pinecone, Qdrant, Weaviate, Chroma)",
            "vectordb",
        )
        .with_param("api_key", ParameterSchema::string("Vector database API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Vector DB provider: pinecone, qdrant, weaviate, chroma").required())
        .with_param("collection", ParameterSchema::string("Collection/index name").required())
        .with_param("vectors", ParameterSchema::array("Array of vectors with id, values, and optional metadata", ParameterSchema::json("Vector")).required())
        .with_param("namespace", ParameterSchema::string("Namespace (for providers that support it)"))
        .with_output("upserted_count", OutputSchema::number("Number of vectors upserted"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let collection = get_required_string_param(&params, "collection")?;
        let vectors_json = get_array_param(&params, "vectors").ok_or_else(|| crate::Error::MissingParameter("vectors".into()))?;
        let namespace = get_string_param(&params, "namespace");

        let vectors: Vec<Vector> = vectors_json.iter().filter_map(|v| {
            let id = v.get("id")?.as_str()?.to_string();
            let values: Vec<f32> = v.get("values")?.as_array()?.iter().filter_map(|x| x.as_f64().map(|f| f as f32)).collect();
            let metadata = v.get("metadata").and_then(|m| {
                if let serde_json::Value::Object(obj) = m {
                    Some(obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                } else {
                    None
                }
            });
            Some(Vector { id, values, metadata, sparse_values: None })
        }).collect();

        let options = UpsertOptions { namespace };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "pinecone")]
            "pinecone" => {
                use swissknife_vectordb_sdk::pinecone::PineconeClient;
                let client = PineconeClient::new(&api_key);
                client.upsert(&collection, &vectors, &options).await
            }
            #[cfg(feature = "qdrant")]
            "qdrant" => {
                use swissknife_vectordb_sdk::qdrant::QdrantClient;
                let client = QdrantClient::new(&api_key);
                client.upsert(&collection, &vectors, &options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported vector DB provider: {}", provider)));
            }
        };

        match result {
            Ok(upsert_result) => Ok(ToolResponse::success(serde_json::json!({
                "upserted_count": upsert_result.upserted_count,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Upsert failed: {}", e))),
        }
    }
}

pub struct VectorQueryTool;

impl Default for VectorQueryTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for VectorQueryTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "vector_query",
            "Vector Query",
            "Query vectors from a vector database using similarity search",
            "vectordb",
        )
        .with_param("api_key", ParameterSchema::string("Vector database API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Vector DB provider: pinecone, qdrant, weaviate, chroma").required())
        .with_param("collection", ParameterSchema::string("Collection/index name").required())
        .with_param("vector", ParameterSchema::array("Query vector", ParameterSchema::number("Vector component")).required())
        .with_param("top_k", ParameterSchema::integer("Number of results to return").with_default(serde_json::json!(10)))
        .with_param("include_values", ParameterSchema::boolean("Include vector values in results").with_default(serde_json::json!(false)))
        .with_param("include_metadata", ParameterSchema::boolean("Include metadata in results").with_default(serde_json::json!(true)))
        .with_param("filter", ParameterSchema::json("Metadata filter"))
        .with_param("namespace", ParameterSchema::string("Namespace (for providers that support it)"))
        .with_output("matches", OutputSchema::array("Matching vectors with scores", OutputSchema::json("Match")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let collection = get_required_string_param(&params, "collection")?;
        let vector_json = get_array_param(&params, "vector").ok_or_else(|| crate::Error::MissingParameter("vector".into()))?;
        let top_k = get_i64_param(&params, "top_k").unwrap_or(10) as u32;
        let include_values = get_bool_param(&params, "include_values").unwrap_or(false);
        let include_metadata = get_bool_param(&params, "include_metadata").unwrap_or(true);
        let filter = get_object_param(&params, "filter");
        let namespace = get_string_param(&params, "namespace");

        let vector: Vec<f32> = vector_json.iter().filter_map(|v| v.as_f64().map(|f| f as f32)).collect();

        let options = QueryOptions {
            top_k,
            include_values,
            include_metadata,
            filter: filter.map(|f| serde_json::Value::Object(f.iter().map(|(k, v)| (k.clone(), v.clone())).collect())),
            namespace,
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "pinecone")]
            "pinecone" => {
                use swissknife_vectordb_sdk::pinecone::PineconeClient;
                let client = PineconeClient::new(&api_key);
                client.query(&collection, &vector, &options).await
            }
            #[cfg(feature = "qdrant")]
            "qdrant" => {
                use swissknife_vectordb_sdk::qdrant::QdrantClient;
                let client = QdrantClient::new(&api_key);
                client.query(&collection, &vector, &options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported vector DB provider: {}", provider)));
            }
        };

        match result {
            Ok(matches) => Ok(ToolResponse::success(serde_json::json!({
                "matches": matches.iter().map(|m| serde_json::json!({
                    "id": m.id,
                    "score": m.score,
                    "values": m.values,
                    "metadata": m.metadata,
                })).collect::<Vec<_>>(),
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Query failed: {}", e))),
        }
    }
}

pub struct VectorDeleteTool;

impl Default for VectorDeleteTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for VectorDeleteTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "vector_delete",
            "Vector Delete",
            "Delete vectors from a vector database by ID",
            "vectordb",
        )
        .with_param("api_key", ParameterSchema::string("Vector database API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Vector DB provider: pinecone, qdrant, weaviate, chroma").required())
        .with_param("collection", ParameterSchema::string("Collection/index name").required())
        .with_param("ids", ParameterSchema::array("Vector IDs to delete", ParameterSchema::string("ID")).required())
        .with_param("namespace", ParameterSchema::string("Namespace (for providers that support it)"))
        .with_output("success", OutputSchema::boolean("Whether deletion succeeded"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let collection = get_required_string_param(&params, "collection")?;
        let ids_json = get_array_param(&params, "ids").ok_or_else(|| crate::Error::MissingParameter("ids".into()))?;
        let namespace = get_string_param(&params, "namespace");

        let ids: Vec<&str> = ids_json.iter().filter_map(|v| v.as_str()).collect();

        let options = swissknife_vectordb_sdk::DeleteOptions {
            namespace,
            delete_all: false,
            filter: None,
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "pinecone")]
            "pinecone" => {
                use swissknife_vectordb_sdk::pinecone::PineconeClient;
                let client = PineconeClient::new(&api_key);
                client.delete(&collection, &ids, &options).await
            }
            #[cfg(feature = "qdrant")]
            "qdrant" => {
                use swissknife_vectordb_sdk::qdrant::QdrantClient;
                let client = QdrantClient::new(&api_key);
                client.delete(&collection, &ids, &options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported vector DB provider: {}", provider)));
            }
        };

        match result {
            Ok(()) => Ok(ToolResponse::success(serde_json::json!({
                "success": true,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Delete failed: {}", e))),
        }
    }
}

pub struct VectorCreateCollectionTool;

impl Default for VectorCreateCollectionTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for VectorCreateCollectionTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "vector_create_collection",
            "Vector Create Collection",
            "Create a new collection/index in a vector database",
            "vectordb",
        )
        .with_param("api_key", ParameterSchema::string("Vector database API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Vector DB provider: pinecone, qdrant, weaviate, chroma").required())
        .with_param("name", ParameterSchema::string("Collection name").required())
        .with_param("dimension", ParameterSchema::integer("Vector dimension").required())
        .with_param("metric", ParameterSchema::string("Distance metric: cosine, euclidean, dot_product").with_default(serde_json::json!("cosine")))
        .with_output("collection", OutputSchema::json("Created collection info"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let name = get_required_string_param(&params, "name")?;
        let dimension = get_i64_param(&params, "dimension").ok_or_else(|| crate::Error::MissingParameter("dimension".into()))? as u32;
        let metric_str = get_string_param(&params, "metric").unwrap_or_else(|| "cosine".to_string());

        let metric = match metric_str.to_lowercase().as_str() {
            "cosine" => DistanceMetric::Cosine,
            "euclidean" => DistanceMetric::Euclidean,
            "dot_product" | "dotproduct" => DistanceMetric::DotProduct,
            _ => DistanceMetric::Cosine,
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "pinecone")]
            "pinecone" => {
                use swissknife_vectordb_sdk::pinecone::PineconeClient;
                let client = PineconeClient::new(&api_key);
                client.create_collection(&name, dimension, metric).await
            }
            #[cfg(feature = "qdrant")]
            "qdrant" => {
                use swissknife_vectordb_sdk::qdrant::QdrantClient;
                let client = QdrantClient::new(&api_key);
                client.create_collection(&name, dimension, metric).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported vector DB provider: {}", provider)));
            }
        };

        match result {
            Ok(collection) => Ok(ToolResponse::success(serde_json::json!({
                "collection": {
                    "name": collection.name,
                    "dimension": collection.dimension,
                    "metric": format!("{:?}", collection.metric),
                }
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Create collection failed: {}", e))),
        }
    }
}
