use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "vectordb")]
use swissknife_vectordb_sdk as vectordb;

#[derive(Clone)]
pub struct VectorDbTools {
    #[cfg(feature = "pinecone")]
    pub pinecone: Option<vectordb::pinecone::PineconeClient>,
    #[cfg(feature = "qdrant")]
    pub qdrant: Option<vectordb::qdrant::QdrantClient>,
    #[cfg(feature = "weaviate")]
    pub weaviate: Option<vectordb::weaviate::WeaviateClient>,
    #[cfg(feature = "chroma")]
    pub chroma: Option<vectordb::chroma::ChromaClient>,
}

impl VectorDbTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "pinecone")]
            pinecone: None,
            #[cfg(feature = "qdrant")]
            qdrant: None,
            #[cfg(feature = "weaviate")]
            weaviate: None,
            #[cfg(feature = "chroma")]
            chroma: None,
        }
    }

    #[cfg(feature = "pinecone")]
    pub fn with_pinecone(mut self, client: vectordb::pinecone::PineconeClient) -> Self {
        self.pinecone = Some(client);
        self
    }

    #[cfg(feature = "qdrant")]
    pub fn with_qdrant(mut self, client: vectordb::qdrant::QdrantClient) -> Self {
        self.qdrant = Some(client);
        self
    }

    #[cfg(feature = "weaviate")]
    pub fn with_weaviate(mut self, client: vectordb::weaviate::WeaviateClient) -> Self {
        self.weaviate = Some(client);
        self
    }

    #[cfg(feature = "chroma")]
    pub fn with_chroma(mut self, client: vectordb::chroma::ChromaClient) -> Self {
        self.chroma = Some(client);
        self
    }
}

impl Default for VectorDbTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PineconeUpsertRequest {
    pub namespace: String,
    pub vectors: Vec<PineconeVector>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PineconeVector {
    pub id: String,
    pub values: Vec<f32>,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PineconeQueryRequest {
    pub namespace: String,
    pub vector: Vec<f32>,
    pub top_k: u32,
    #[serde(default)]
    pub include_metadata: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PineconeDeleteRequest {
    pub namespace: String,
    pub ids: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QdrantUpsertRequest {
    pub collection: String,
    pub points: Vec<QdrantPoint>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QdrantPoint {
    pub id: String,
    pub vector: Vec<f32>,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QdrantSearchRequest {
    pub collection: String,
    pub vector: Vec<f32>,
    pub limit: u32,
    #[serde(default)]
    pub filter: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QdrantDeleteRequest {
    pub collection: String,
    pub ids: Vec<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeaviateAddObjectRequest {
    pub class_name: String,
    pub properties: serde_json::Value,
    #[serde(default)]
    pub vector: Option<Vec<f32>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeaviateSearchRequest {
    pub class_name: String,
    pub query: String,
    pub limit: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WeaviateVectorSearchRequest {
    pub class_name: String,
    pub vector: Vec<f32>,
    pub limit: u32,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChromaAddRequest {
    pub collection: String,
    pub ids: Vec<String>,
    pub embeddings: Vec<Vec<f32>>,
    #[serde(default)]
    pub documents: Option<Vec<String>>,
    #[serde(default)]
    pub metadatas: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ChromaQueryRequest {
    pub collection: String,
    pub query_embeddings: Vec<Vec<f32>>,
    pub n_results: u32,
}

#[tool_box]
impl VectorDbTools {
    #[cfg(feature = "pinecone")]
    #[rmcp::tool(description = "Upsert vectors to Pinecone")]
    pub async fn pinecone_upsert(
        &self,
        #[rmcp::tool(aggr)] req: PineconeUpsertRequest,
    ) -> Result<String, String> {
        let client = self.pinecone.as_ref()
            .ok_or_else(|| "Pinecone client not configured".to_string())?;

        let vectors: Vec<_> = req.vectors.iter().map(|v| {
            vectordb::pinecone::Vector {
                id: v.id.clone(),
                values: v.values.clone(),
                metadata: v.metadata.clone(),
            }
        }).collect();

        let result = client.upsert(&req.namespace, &vectors).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "pinecone")]
    #[rmcp::tool(description = "Query vectors from Pinecone")]
    pub async fn pinecone_query(
        &self,
        #[rmcp::tool(aggr)] req: PineconeQueryRequest,
    ) -> Result<String, String> {
        let client = self.pinecone.as_ref()
            .ok_or_else(|| "Pinecone client not configured".to_string())?;

        let result = client.query(
            &req.namespace,
            &req.vector,
            req.top_k,
            req.include_metadata.unwrap_or(true),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "pinecone")]
    #[rmcp::tool(description = "Delete vectors from Pinecone")]
    pub async fn pinecone_delete(
        &self,
        #[rmcp::tool(aggr)] req: PineconeDeleteRequest,
    ) -> Result<String, String> {
        let client = self.pinecone.as_ref()
            .ok_or_else(|| "Pinecone client not configured".to_string())?;

        client.delete(&req.namespace, &req.ids).await
            .map_err(|e| e.to_string())?;

        Ok(format!("Deleted {} vectors", req.ids.len()))
    }

    #[cfg(feature = "qdrant")]
    #[rmcp::tool(description = "Upsert points to Qdrant")]
    pub async fn qdrant_upsert(
        &self,
        #[rmcp::tool(aggr)] req: QdrantUpsertRequest,
    ) -> Result<String, String> {
        let client = self.qdrant.as_ref()
            .ok_or_else(|| "Qdrant client not configured".to_string())?;

        let points: Vec<_> = req.points.iter().map(|p| {
            vectordb::qdrant::Point {
                id: p.id.clone(),
                vector: p.vector.clone(),
                payload: p.payload.clone(),
            }
        }).collect();

        let result = client.upsert(&req.collection, &points).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "qdrant")]
    #[rmcp::tool(description = "Search vectors in Qdrant")]
    pub async fn qdrant_search(
        &self,
        #[rmcp::tool(aggr)] req: QdrantSearchRequest,
    ) -> Result<String, String> {
        let client = self.qdrant.as_ref()
            .ok_or_else(|| "Qdrant client not configured".to_string())?;

        let result = client.search(&req.collection, &req.vector, req.limit, req.filter).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "qdrant")]
    #[rmcp::tool(description = "Delete points from Qdrant")]
    pub async fn qdrant_delete(
        &self,
        #[rmcp::tool(aggr)] req: QdrantDeleteRequest,
    ) -> Result<String, String> {
        let client = self.qdrant.as_ref()
            .ok_or_else(|| "Qdrant client not configured".to_string())?;

        client.delete(&req.collection, &req.ids).await
            .map_err(|e| e.to_string())?;

        Ok(format!("Deleted {} points", req.ids.len()))
    }

    #[cfg(feature = "weaviate")]
    #[rmcp::tool(description = "Add an object to Weaviate")]
    pub async fn weaviate_add_object(
        &self,
        #[rmcp::tool(aggr)] req: WeaviateAddObjectRequest,
    ) -> Result<String, String> {
        let client = self.weaviate.as_ref()
            .ok_or_else(|| "Weaviate client not configured".to_string())?;

        let result = client.add_object(&req.class_name, req.properties, req.vector).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "weaviate")]
    #[rmcp::tool(description = "Search objects in Weaviate using text")]
    pub async fn weaviate_search(
        &self,
        #[rmcp::tool(aggr)] req: WeaviateSearchRequest,
    ) -> Result<String, String> {
        let client = self.weaviate.as_ref()
            .ok_or_else(|| "Weaviate client not configured".to_string())?;

        let result = client.search(&req.class_name, &req.query, req.limit).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "weaviate")]
    #[rmcp::tool(description = "Search objects in Weaviate using vector")]
    pub async fn weaviate_vector_search(
        &self,
        #[rmcp::tool(aggr)] req: WeaviateVectorSearchRequest,
    ) -> Result<String, String> {
        let client = self.weaviate.as_ref()
            .ok_or_else(|| "Weaviate client not configured".to_string())?;

        let result = client.vector_search(&req.class_name, &req.vector, req.limit).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "chroma")]
    #[rmcp::tool(description = "Add embeddings to ChromaDB")]
    pub async fn chroma_add(
        &self,
        #[rmcp::tool(aggr)] req: ChromaAddRequest,
    ) -> Result<String, String> {
        let client = self.chroma.as_ref()
            .ok_or_else(|| "ChromaDB client not configured".to_string())?;

        client.add(
            &req.collection,
            &req.ids,
            &req.embeddings,
            req.documents.as_deref(),
            req.metadatas.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        Ok(format!("Added {} embeddings to collection {}", req.ids.len(), req.collection))
    }

    #[cfg(feature = "chroma")]
    #[rmcp::tool(description = "Query embeddings from ChromaDB")]
    pub async fn chroma_query(
        &self,
        #[rmcp::tool(aggr)] req: ChromaQueryRequest,
    ) -> Result<String, String> {
        let client = self.chroma.as_ref()
            .ok_or_else(|| "ChromaDB client not configured".to_string())?;

        let result = client.query(&req.collection, &req.query_embeddings, req.n_results).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }
}
