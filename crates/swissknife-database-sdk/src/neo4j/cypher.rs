use crate::{Error, Result};
use crate::neo4j::Neo4jClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

impl Neo4jClient {
    pub async fn run_query(&self, cypher: &str, parameters: Option<HashMap<String, serde_json::Value>>) -> Result<CypherResponse> {
        let body = serde_json::json!({
            "statements": [{
                "statement": cypher,
                "parameters": parameters.unwrap_or_default(),
                "resultDataContents": ["row", "graph"]
            }]
        });

        let response = self.client()
            .post(format!("{}/db/{}/tx/commit", self.base_url(), self.database()))
            .header("Authorization", self.auth_header())
            .header("Content-Type", "application/json")
            .header("Accept", "application/json")
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

        let result: CypherResponse = response.json().await?;

        if !result.errors.is_empty() {
            return Err(Error::Api {
                message: result.errors.first().map(|e| e.message.clone()).unwrap_or_default(),
                code: result.errors.first().and_then(|e| e.code.clone()),
            });
        }

        Ok(result)
    }

    pub async fn create_node(&self, labels: &[&str], properties: HashMap<String, serde_json::Value>) -> Result<Neo4jNode> {
        let labels_str = labels.iter().map(|l| format!(":{}", l)).collect::<Vec<_>>().join("");
        let cypher = format!("CREATE (n{} $props) RETURN n", labels_str);

        let mut params = HashMap::new();
        params.insert("props".to_string(), serde_json::to_value(properties)?);

        let response = self.run_query(&cypher, Some(params)).await?;

        let node = response.results.first()
            .and_then(|r| r.data.first())
            .and_then(|d| d.graph.as_ref())
            .and_then(|g| g.nodes.first())
            .cloned()
            .ok_or_else(|| Error::Api { message: "No node created".to_string(), code: None })?;

        Ok(node)
    }

    pub async fn find_nodes(&self, label: &str, properties: Option<HashMap<String, serde_json::Value>>, limit: Option<u32>) -> Result<Vec<Neo4jNode>> {
        let where_clause = if let Some(props) = &properties {
            let conditions: Vec<String> = props.keys()
                .map(|k| format!("n.{} = $props.{}", k, k))
                .collect();
            if conditions.is_empty() {
                String::new()
            } else {
                format!(" WHERE {}", conditions.join(" AND "))
            }
        } else {
            String::new()
        };

        let limit_clause = limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();
        let cypher = format!("MATCH (n:{}){}RETURN n{}", label, where_clause, limit_clause);

        let mut params = HashMap::new();
        if let Some(props) = properties {
            params.insert("props".to_string(), serde_json::to_value(props)?);
        }

        let response = self.run_query(&cypher, Some(params)).await?;

        let nodes = response.results.first()
            .map(|r| r.data.iter()
                .filter_map(|d| d.graph.as_ref())
                .flat_map(|g| g.nodes.clone())
                .collect())
            .unwrap_or_default();

        Ok(nodes)
    }

    pub async fn find_node_by_id(&self, id: i64) -> Result<Option<Neo4jNode>> {
        let cypher = "MATCH (n) WHERE id(n) = $id RETURN n";
        let mut params = HashMap::new();
        params.insert("id".to_string(), serde_json::json!(id));

        let response = self.run_query(cypher, Some(params)).await?;

        let node = response.results.first()
            .and_then(|r| r.data.first())
            .and_then(|d| d.graph.as_ref())
            .and_then(|g| g.nodes.first())
            .cloned();

        Ok(node)
    }

    pub async fn update_node(&self, id: i64, properties: HashMap<String, serde_json::Value>) -> Result<Neo4jNode> {
        let cypher = "MATCH (n) WHERE id(n) = $id SET n += $props RETURN n";
        let mut params = HashMap::new();
        params.insert("id".to_string(), serde_json::json!(id));
        params.insert("props".to_string(), serde_json::to_value(properties)?);

        let response = self.run_query(cypher, Some(params)).await?;

        let node = response.results.first()
            .and_then(|r| r.data.first())
            .and_then(|d| d.graph.as_ref())
            .and_then(|g| g.nodes.first())
            .cloned()
            .ok_or_else(|| Error::Api { message: "Node not found".to_string(), code: None })?;

        Ok(node)
    }

    pub async fn delete_node(&self, id: i64, detach: bool) -> Result<()> {
        let cypher = if detach {
            "MATCH (n) WHERE id(n) = $id DETACH DELETE n"
        } else {
            "MATCH (n) WHERE id(n) = $id DELETE n"
        };

        let mut params = HashMap::new();
        params.insert("id".to_string(), serde_json::json!(id));

        self.run_query(cypher, Some(params)).await?;
        Ok(())
    }

    pub async fn create_relationship(&self, from_id: i64, to_id: i64, rel_type: &str, properties: Option<HashMap<String, serde_json::Value>>) -> Result<Neo4jRelationship> {
        let props_clause = if properties.is_some() { " $props" } else { "" };
        let cypher = format!(
            "MATCH (a), (b) WHERE id(a) = $from AND id(b) = $to CREATE (a)-[r:{}{}]->(b) RETURN r",
            rel_type, props_clause
        );

        let mut params = HashMap::new();
        params.insert("from".to_string(), serde_json::json!(from_id));
        params.insert("to".to_string(), serde_json::json!(to_id));
        if let Some(props) = properties {
            params.insert("props".to_string(), serde_json::to_value(props)?);
        }

        let response = self.run_query(&cypher, Some(params)).await?;

        let rel = response.results.first()
            .and_then(|r| r.data.first())
            .and_then(|d| d.graph.as_ref())
            .and_then(|g| g.relationships.first())
            .cloned()
            .ok_or_else(|| Error::Api { message: "No relationship created".to_string(), code: None })?;

        Ok(rel)
    }

    pub async fn find_relationships(&self, from_label: Option<&str>, to_label: Option<&str>, rel_type: Option<&str>) -> Result<Vec<Neo4jRelationship>> {
        let from = from_label.map(|l| format!(":{}", l)).unwrap_or_default();
        let to = to_label.map(|l| format!(":{}", l)).unwrap_or_default();
        let rel = rel_type.map(|t| format!(":{}", t)).unwrap_or_default();

        let cypher = format!("MATCH (a{})-[r{}]->(b{}) RETURN r", from, rel, to);
        let response = self.run_query(&cypher, None).await?;

        let rels = response.results.first()
            .map(|r| r.data.iter()
                .filter_map(|d| d.graph.as_ref())
                .flat_map(|g| g.relationships.clone())
                .collect())
            .unwrap_or_default();

        Ok(rels)
    }

    pub async fn get_node_count(&self, label: Option<&str>) -> Result<i64> {
        let cypher = match label {
            Some(l) => format!("MATCH (n:{}) RETURN count(n) as count", l),
            None => "MATCH (n) RETURN count(n) as count".to_string(),
        };

        let response = self.run_query(&cypher, None).await?;

        let count = response.results.first()
            .and_then(|r| r.data.first())
            .and_then(|d| d.row.as_ref())
            .and_then(|row| row.first())
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        Ok(count)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CypherResponse {
    pub results: Vec<CypherResult>,
    pub errors: Vec<CypherError>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CypherResult {
    pub columns: Vec<String>,
    pub data: Vec<CypherData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CypherData {
    pub row: Option<Vec<serde_json::Value>>,
    pub graph: Option<GraphData>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<Neo4jNode>,
    pub relationships: Vec<Neo4jRelationship>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Neo4jNode {
    pub id: String,
    pub labels: Vec<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Neo4jRelationship {
    pub id: String,
    #[serde(rename = "type")]
    pub rel_type: String,
    #[serde(rename = "startNode")]
    pub start_node: String,
    #[serde(rename = "endNode")]
    pub end_node: String,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CypherError {
    pub code: Option<String>,
    pub message: String,
}
