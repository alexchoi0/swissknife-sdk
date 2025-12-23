use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "database")]
use swissknife_database_sdk as db;

#[derive(Clone)]
pub struct DatabaseTools {
    #[cfg(feature = "supabase")]
    pub supabase: Option<db::supabase::SupabaseClient>,
    #[cfg(feature = "planetscale")]
    pub planetscale: Option<db::planetscale::PlanetScaleClient>,
    #[cfg(feature = "turso")]
    pub turso: Option<db::turso::TursoClient>,
    #[cfg(feature = "neon")]
    pub neon: Option<db::neon::NeonClient>,
    #[cfg(feature = "upstash")]
    pub upstash_redis: Option<db::upstash::redis::UpstashRedisClient>,
    #[cfg(feature = "upstash")]
    pub upstash_vector: Option<db::upstash::vector::UpstashVectorClient>,
    #[cfg(feature = "pinecone")]
    pub pinecone: Option<db::pinecone::PineconeClient>,
    #[cfg(feature = "qdrant")]
    pub qdrant: Option<db::qdrant::QdrantClient>,
    #[cfg(feature = "weaviate")]
    pub weaviate: Option<db::weaviate::WeaviateClient>,
    #[cfg(feature = "chromadb")]
    pub chromadb: Option<db::chromadb::ChromaClient>,
}

impl DatabaseTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "supabase")]
            supabase: None,
            #[cfg(feature = "planetscale")]
            planetscale: None,
            #[cfg(feature = "turso")]
            turso: None,
            #[cfg(feature = "neon")]
            neon: None,
            #[cfg(feature = "upstash")]
            upstash_redis: None,
            #[cfg(feature = "upstash")]
            upstash_vector: None,
            #[cfg(feature = "pinecone")]
            pinecone: None,
            #[cfg(feature = "qdrant")]
            qdrant: None,
            #[cfg(feature = "weaviate")]
            weaviate: None,
            #[cfg(feature = "chromadb")]
            chromadb: None,
        }
    }

    #[cfg(feature = "supabase")]
    pub fn with_supabase(mut self, client: db::supabase::SupabaseClient) -> Self {
        self.supabase = Some(client);
        self
    }

    #[cfg(feature = "planetscale")]
    pub fn with_planetscale(mut self, client: db::planetscale::PlanetScaleClient) -> Self {
        self.planetscale = Some(client);
        self
    }

    #[cfg(feature = "turso")]
    pub fn with_turso(mut self, client: db::turso::TursoClient) -> Self {
        self.turso = Some(client);
        self
    }

    #[cfg(feature = "neon")]
    pub fn with_neon(mut self, client: db::neon::NeonClient) -> Self {
        self.neon = Some(client);
        self
    }

    #[cfg(feature = "upstash")]
    pub fn with_upstash_redis(mut self, client: db::upstash::redis::UpstashRedisClient) -> Self {
        self.upstash_redis = Some(client);
        self
    }

    #[cfg(feature = "upstash")]
    pub fn with_upstash_vector(mut self, client: db::upstash::vector::UpstashVectorClient) -> Self {
        self.upstash_vector = Some(client);
        self
    }

    #[cfg(feature = "pinecone")]
    pub fn with_pinecone(mut self, client: db::pinecone::PineconeClient) -> Self {
        self.pinecone = Some(client);
        self
    }

    #[cfg(feature = "qdrant")]
    pub fn with_qdrant(mut self, client: db::qdrant::QdrantClient) -> Self {
        self.qdrant = Some(client);
        self
    }

    #[cfg(feature = "weaviate")]
    pub fn with_weaviate(mut self, client: db::weaviate::WeaviateClient) -> Self {
        self.weaviate = Some(client);
        self
    }

    #[cfg(feature = "chromadb")]
    pub fn with_chromadb(mut self, client: db::chromadb::ChromaClient) -> Self {
        self.chromadb = Some(client);
        self
    }
}

impl Default for DatabaseTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SupabaseQueryRequest {
    pub table: String,
    #[serde(default)]
    pub select: Option<String>,
    #[serde(default)]
    pub filter: Option<serde_json::Value>,
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SupabaseInsertRequest {
    pub table: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PlanetScaleExecuteRequest {
    pub query: String,
    #[serde(default)]
    pub params: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TursoExecuteRequest {
    pub sql: String,
    #[serde(default)]
    pub args: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct NeonQueryRequest {
    pub query: String,
    #[serde(default)]
    pub params: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpstashRedisGetRequest {
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpstashRedisSetRequest {
    pub key: String,
    pub value: String,
    #[serde(default)]
    pub ex: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpstashVectorUpsertRequest {
    pub id: String,
    pub vector: Vec<f64>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpstashVectorQueryRequest {
    pub vector: Vec<f64>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub include_metadata: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PineconeUpsertRequest {
    #[serde(default)]
    pub namespace: Option<String>,
    pub vectors: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PineconeQueryRequest {
    #[serde(default)]
    pub namespace: Option<String>,
    pub vector: Vec<f64>,
    #[serde(default)]
    pub top_k: Option<u32>,
    #[serde(default)]
    pub include_metadata: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QdrantUpsertRequest {
    pub collection: String,
    pub points: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QdrantSearchRequest {
    pub collection: String,
    pub vector: Vec<f64>,
    #[serde(default)]
    pub limit: Option<u64>,
    #[serde(default)]
    pub with_payload: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeaviateCreateObjectRequest {
    pub class: String,
    pub properties: serde_json::Value,
    #[serde(default)]
    pub vector: Option<Vec<f64>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeaviateSearchRequest {
    pub class: String,
    #[serde(default)]
    pub near_vector: Option<Vec<f64>>,
    #[serde(default)]
    pub near_text: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChromaAddRequest {
    pub collection: String,
    pub ids: Vec<String>,
    #[serde(default)]
    pub documents: Option<Vec<String>>,
    #[serde(default)]
    pub embeddings: Option<Vec<Vec<f64>>>,
    #[serde(default)]
    pub metadatas: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChromaQueryRequest {
    pub collection: String,
    #[serde(default)]
    pub query_embeddings: Option<Vec<Vec<f64>>>,
    #[serde(default)]
    pub query_texts: Option<Vec<String>>,
    #[serde(default)]
    pub n_results: Option<u32>,
}

#[tool_box]
impl DatabaseTools {
    #[cfg(feature = "supabase")]
    #[rmcp::tool(description = "Execute a query on Supabase")]
    pub async fn supabase_query(
        &self,
        #[rmcp::tool(aggr)] req: SupabaseQueryRequest,
    ) -> Result<String, String> {
        let client = self.supabase.as_ref()
            .ok_or_else(|| "Supabase client not configured".to_string())?;

        let select = req.select.unwrap_or_else(|| "*".to_string());
        let mut query = client.from(&req.table).select(&select);

        if let Some(limit) = req.limit {
            query = query.limit(limit as usize);
        }

        let response = query.execute().await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&response)
            .map_err(|e| e.to_string())
    }

    #[cfg(feature = "supabase")]
    #[rmcp::tool(description = "Insert data into Supabase")]
    pub async fn supabase_insert(
        &self,
        #[rmcp::tool(aggr)] req: SupabaseInsertRequest,
    ) -> Result<String, String> {
        let client = self.supabase.as_ref()
            .ok_or_else(|| "Supabase client not configured".to_string())?;

        let response = client.from(&req.table)
            .insert(req.data)
            .execute()
            .await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&response)
            .map_err(|e| e.to_string())
    }

    #[cfg(feature = "planetscale")]
    #[rmcp::tool(description = "Execute a SQL query on PlanetScale")]
    pub async fn planetscale_execute(
        &self,
        #[rmcp::tool(aggr)] req: PlanetScaleExecuteRequest,
    ) -> Result<String, String> {
        let client = self.planetscale.as_ref()
            .ok_or_else(|| "PlanetScale client not configured".to_string())?;

        let response = client.execute(&req.query).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "rows": response.rows,
            "rows_affected": response.rows_affected
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "turso")]
    #[rmcp::tool(description = "Execute a SQL query on Turso")]
    pub async fn turso_execute(
        &self,
        #[rmcp::tool(aggr)] req: TursoExecuteRequest,
    ) -> Result<String, String> {
        let client = self.turso.as_ref()
            .ok_or_else(|| "Turso client not configured".to_string())?;

        let stmt = db::turso::Statement {
            sql: req.sql,
            args: None,
            named_args: None,
        };

        let response = client.execute(stmt).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "columns": response.cols,
            "rows": response.rows,
            "affected_row_count": response.affected_row_count
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "neon")]
    #[rmcp::tool(description = "Execute a SQL query on Neon Postgres")]
    pub async fn neon_query(
        &self,
        #[rmcp::tool(aggr)] req: NeonQueryRequest,
    ) -> Result<String, String> {
        let client = self.neon.as_ref()
            .ok_or_else(|| "Neon client not configured".to_string())?;

        let params = req.params.unwrap_or_default();
        let response = client.query(&req.query, &params).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "rows": response.rows,
            "fields": response.fields
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "upstash")]
    #[rmcp::tool(description = "Get a value from Upstash Redis")]
    pub async fn upstash_redis_get(
        &self,
        #[rmcp::tool(aggr)] req: UpstashRedisGetRequest,
    ) -> Result<String, String> {
        let client = self.upstash_redis.as_ref()
            .ok_or_else(|| "Upstash Redis client not configured".to_string())?;

        let value = client.get(&req.key).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "key": req.key,
            "value": value
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "upstash")]
    #[rmcp::tool(description = "Set a value in Upstash Redis")]
    pub async fn upstash_redis_set(
        &self,
        #[rmcp::tool(aggr)] req: UpstashRedisSetRequest,
    ) -> Result<String, String> {
        let client = self.upstash_redis.as_ref()
            .ok_or_else(|| "Upstash Redis client not configured".to_string())?;

        if let Some(seconds) = req.ex {
            client.setex(&req.key, seconds as u64, &req.value).await
                .map_err(|e| e.to_string())?;
        } else {
            client.set(&req.key, &req.value).await
                .map_err(|e| e.to_string())?;
        }

        Ok("OK".to_string())
    }

    #[cfg(feature = "upstash")]
    #[rmcp::tool(description = "Upsert vectors into Upstash Vector")]
    pub async fn upstash_vector_upsert(
        &self,
        #[rmcp::tool(aggr)] req: UpstashVectorUpsertRequest,
    ) -> Result<String, String> {
        let client = self.upstash_vector.as_ref()
            .ok_or_else(|| "Upstash Vector client not configured".to_string())?;

        let vector: Vec<f32> = req.vector.iter().map(|v| *v as f32).collect();

        let upsert = db::upstash::vector::UpsertRequest {
            id: req.id,
            vector,
            metadata: req.metadata,
        };

        client.upsert(vec![upsert]).await
            .map_err(|e| e.to_string())?;

        Ok("Vector upserted successfully".to_string())
    }

    #[cfg(feature = "upstash")]
    #[rmcp::tool(description = "Query similar vectors from Upstash Vector")]
    pub async fn upstash_vector_query(
        &self,
        #[rmcp::tool(aggr)] req: UpstashVectorQueryRequest,
    ) -> Result<String, String> {
        let client = self.upstash_vector.as_ref()
            .ok_or_else(|| "Upstash Vector client not configured".to_string())?;

        let vector: Vec<f32> = req.vector.iter().map(|v| *v as f32).collect();
        let top_k = req.top_k.unwrap_or(10);

        let request = db::upstash::vector::QueryRequest {
            vector,
            top_k,
            include_metadata: req.include_metadata.unwrap_or(true),
            include_vectors: false,
            filter: None,
        };

        let results = client.query(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "score": r.score,
                    "metadata": r.metadata
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "pinecone")]
    #[rmcp::tool(description = "Upsert vectors into Pinecone")]
    pub async fn pinecone_upsert(
        &self,
        #[rmcp::tool(aggr)] req: PineconeUpsertRequest,
    ) -> Result<String, String> {
        let client = self.pinecone.as_ref()
            .ok_or_else(|| "Pinecone client not configured".to_string())?;

        let pinecone_vectors: Vec<db::pinecone::Vector> = req.vectors.iter()
            .filter_map(|v| {
                let id = v.get("id")?.as_str()?.to_string();
                let values: Vec<f32> = v.get("values")?
                    .as_array()?
                    .iter()
                    .filter_map(|f| f.as_f64().map(|n| n as f32))
                    .collect();
                Some(db::pinecone::Vector {
                    id,
                    values,
                    metadata: v.get("metadata").cloned(),
                    sparse_values: None,
                })
            })
            .collect();

        let request = db::pinecone::UpsertRequest {
            vectors: pinecone_vectors,
            namespace: req.namespace,
        };

        let response = client.upsert(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "upserted_count": response.upserted_count
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "pinecone")]
    #[rmcp::tool(description = "Query similar vectors from Pinecone")]
    pub async fn pinecone_query(
        &self,
        #[rmcp::tool(aggr)] req: PineconeQueryRequest,
    ) -> Result<String, String> {
        let client = self.pinecone.as_ref()
            .ok_or_else(|| "Pinecone client not configured".to_string())?;

        let vector: Vec<f32> = req.vector.iter().map(|v| *v as f32).collect();
        let top_k = req.top_k.unwrap_or(10);

        let request = db::pinecone::QueryRequest {
            vector: Some(vector),
            id: None,
            top_k,
            namespace: req.namespace,
            include_values: Some(false),
            include_metadata: req.include_metadata,
            filter: None,
        };

        let response = client.query(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "matches": response.matches.iter().map(|m| {
                serde_json::json!({
                    "id": m.id,
                    "score": m.score,
                    "metadata": m.metadata
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "qdrant")]
    #[rmcp::tool(description = "Upsert points into Qdrant")]
    pub async fn qdrant_upsert(
        &self,
        #[rmcp::tool(aggr)] req: QdrantUpsertRequest,
    ) -> Result<String, String> {
        let client = self.qdrant.as_ref()
            .ok_or_else(|| "Qdrant client not configured".to_string())?;

        let qdrant_points: Vec<db::qdrant::Point> = req.points.iter()
            .filter_map(|p| {
                let id = p.get("id")?.as_str()?.to_string();
                let vector: Vec<f32> = p.get("vector")?
                    .as_array()?
                    .iter()
                    .filter_map(|f| f.as_f64().map(|n| n as f32))
                    .collect();
                Some(db::qdrant::Point {
                    id: db::qdrant::PointId::String(id),
                    vector: db::qdrant::Vector::Dense(vector),
                    payload: p.get("payload").cloned(),
                })
            })
            .collect();

        let request = db::qdrant::UpsertRequest {
            points: qdrant_points,
            wait: Some(true),
            ordering: None,
        };

        let response = client.upsert(&req.collection, &request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "status": response.status
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "qdrant")]
    #[rmcp::tool(description = "Search similar vectors in Qdrant")]
    pub async fn qdrant_search(
        &self,
        #[rmcp::tool(aggr)] req: QdrantSearchRequest,
    ) -> Result<String, String> {
        let client = self.qdrant.as_ref()
            .ok_or_else(|| "Qdrant client not configured".to_string())?;

        let vector: Vec<f32> = req.vector.iter().map(|v| *v as f32).collect();
        let limit = req.limit.unwrap_or(10);

        let request = db::qdrant::SearchRequest {
            vector: db::qdrant::Vector::Dense(vector),
            limit,
            offset: None,
            with_payload: req.with_payload,
            with_vector: Some(false),
            filter: None,
            score_threshold: None,
        };

        let results = client.search(&req.collection, &request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "results": results.iter().map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "score": r.score,
                    "payload": r.payload
                })
            }).collect::<Vec<_>>()
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "weaviate")]
    #[rmcp::tool(description = "Create an object in Weaviate")]
    pub async fn weaviate_create_object(
        &self,
        #[rmcp::tool(aggr)] req: WeaviateCreateObjectRequest,
    ) -> Result<String, String> {
        let client = self.weaviate.as_ref()
            .ok_or_else(|| "Weaviate client not configured".to_string())?;

        let vector: Option<Vec<f32>> = req.vector
            .map(|v| v.iter().map(|f| *f as f32).collect());

        let object = db::weaviate::WeaviateObject {
            class: req.class,
            properties: req.properties,
            vector,
            id: None,
        };

        let response = client.create_object(&object).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": response.id
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "weaviate")]
    #[rmcp::tool(description = "Search objects in Weaviate")]
    pub async fn weaviate_search(
        &self,
        #[rmcp::tool(aggr)] req: WeaviateSearchRequest,
    ) -> Result<String, String> {
        let client = self.weaviate.as_ref()
            .ok_or_else(|| "Weaviate client not configured".to_string())?;

        let limit = req.limit.unwrap_or(10);
        let near_vector: Option<Vec<f32>> = req.near_vector
            .map(|v| v.iter().map(|f| *f as f32).collect());

        let mut query = db::weaviate::GetQuery::new(req.class)
            .with_limit(limit);

        if let Some(vector) = near_vector {
            query = query.with_near_vector(vector);
        }

        if let Some(text) = req.near_text {
            query = query.with_near_text(vec![text]);
        }

        let response = client.query(&query).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&response)
            .map_err(|e| e.to_string())
    }

    #[cfg(feature = "chromadb")]
    #[rmcp::tool(description = "Add documents to ChromaDB collection")]
    pub async fn chromadb_add(
        &self,
        #[rmcp::tool(aggr)] req: ChromaAddRequest,
    ) -> Result<String, String> {
        let client = self.chromadb.as_ref()
            .ok_or_else(|| "ChromaDB client not configured".to_string())?;

        let embeddings: Option<Vec<Vec<f32>>> = req.embeddings.map(|embs| {
            embs.iter()
                .map(|e| e.iter().map(|v| *v as f32).collect())
                .collect()
        });

        let collection = client.get_collection(&req.collection).await
            .map_err(|e| e.to_string())?;

        let request = db::chromadb::AddRequest {
            ids: req.ids,
            documents: req.documents,
            embeddings,
            metadatas: req.metadatas,
        };

        collection.add(&request).await
            .map_err(|e| e.to_string())?;

        Ok("Documents added successfully".to_string())
    }

    #[cfg(feature = "chromadb")]
    #[rmcp::tool(description = "Query ChromaDB collection")]
    pub async fn chromadb_query(
        &self,
        #[rmcp::tool(aggr)] req: ChromaQueryRequest,
    ) -> Result<String, String> {
        let client = self.chromadb.as_ref()
            .ok_or_else(|| "ChromaDB client not configured".to_string())?;

        let query_embeddings: Option<Vec<Vec<f32>>> = req.query_embeddings.map(|embs| {
            embs.iter()
                .map(|e| e.iter().map(|v| *v as f32).collect())
                .collect()
        });

        let n_results = req.n_results.unwrap_or(10);

        let collection = client.get_collection(&req.collection).await
            .map_err(|e| e.to_string())?;

        let request = db::chromadb::QueryRequest {
            query_embeddings,
            query_texts: req.query_texts,
            n_results: Some(n_results),
            where_filter: None,
            where_document: None,
            include: None,
        };

        let response = collection.query(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "ids": response.ids,
            "documents": response.documents,
            "metadatas": response.metadatas,
            "distances": response.distances
        })).map_err(|e| e.to_string())
    }
}
