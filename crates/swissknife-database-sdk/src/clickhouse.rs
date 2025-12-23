use crate::{ColumnInfo, IndexInfo, QueryParams, QueryResult, Result, SqlDatabaseProvider, TableInfo};
use async_trait::async_trait;
use std::collections::HashMap;

pub struct ClickHouseClient {
    base_url: String,
    username: Option<String>,
    password: Option<String>,
    database: String,
}

impl ClickHouseClient {
    pub fn new(base_url: &str, database: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
            username: None,
            password: None,
            database: database.to_string(),
        }
    }

    pub fn with_auth(mut self, username: &str, password: &str) -> Self {
        self.username = Some(username.to_string());
        self.password = Some(password.to_string());
        self
    }

    async fn execute_query(&self, query: &str, format: &str) -> Result<serde_json::Value> {
        let client = reqwest::Client::new();

        let url = format!("{}/", self.base_url);

        let full_query = format!("{} FORMAT {}", query, format);

        let mut request = client.post(&url)
            .query(&[("database", &self.database)])
            .body(full_query);

        if let (Some(ref username), Some(ref password)) = (&self.username, &self.password) {
            request = request.basic_auth(username, Some(password));
        }

        let resp = request.send().await
            .map_err(|e| crate::Error::Connection(e.to_string()))?;

        if !resp.status().is_success() {
            let error = resp.text().await.unwrap_or_default();
            return Err(crate::Error::Query(error));
        }

        if format == "JSON" {
            resp.json().await
                .map_err(|e| crate::Error::Query(e.to_string()))
        } else {
            let text = resp.text().await
                .map_err(|e| crate::Error::Query(e.to_string()))?;
            Ok(serde_json::json!({"result": text}))
        }
    }

    pub async fn insert(&self, table: &str, columns: &[&str], values: &[Vec<serde_json::Value>]) -> Result<u64> {
        let cols = columns.join(", ");
        let rows: Vec<String> = values.iter().map(|row| {
            let vals: Vec<String> = row.iter().map(|v| {
                match v {
                    serde_json::Value::String(s) => format!("'{}'", s.replace('\'', "\\'")),
                    serde_json::Value::Number(n) => n.to_string(),
                    serde_json::Value::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
                    serde_json::Value::Null => "NULL".to_string(),
                    _ => format!("'{}'", v.to_string().replace('\'', "\\'")),
                }
            }).collect();
            format!("({})", vals.join(", "))
        }).collect();

        let query = format!("INSERT INTO {} ({}) VALUES {}", table, cols, rows.join(", "));

        self.execute_query(&query, "JSON").await?;

        Ok(values.len() as u64)
    }

    pub async fn create_table(&self, table: &str, columns: &[(String, String)], engine: &str, order_by: &[&str]) -> Result<()> {
        let cols: Vec<String> = columns.iter()
            .map(|(name, typ)| format!("{} {}", name, typ))
            .collect();

        let order = if order_by.is_empty() {
            "tuple()".to_string()
        } else {
            format!("({})", order_by.join(", "))
        };

        let query = format!(
            "CREATE TABLE IF NOT EXISTS {} ({}) ENGINE = {} ORDER BY {}",
            table,
            cols.join(", "),
            engine,
            order
        );

        self.execute_query(&query, "JSON").await?;
        Ok(())
    }

    pub async fn drop_table(&self, table: &str) -> Result<()> {
        let query = format!("DROP TABLE IF EXISTS {}", table);
        self.execute_query(&query, "JSON").await?;
        Ok(())
    }

    pub async fn optimize_table(&self, table: &str, final_flag: bool) -> Result<()> {
        let query = if final_flag {
            format!("OPTIMIZE TABLE {} FINAL", table)
        } else {
            format!("OPTIMIZE TABLE {}", table)
        };
        self.execute_query(&query, "JSON").await?;
        Ok(())
    }

    pub async fn truncate_table(&self, table: &str) -> Result<()> {
        let query = format!("TRUNCATE TABLE {}", table);
        self.execute_query(&query, "JSON").await?;
        Ok(())
    }
}

#[async_trait]
impl SqlDatabaseProvider for ClickHouseClient {
    async fn execute(&self, query: &str, _params: &QueryParams) -> Result<QueryResult> {
        let result = self.execute_query(query, "JSON").await?;

        let affected = result
            .get("statistics")
            .and_then(|s| s.get("rows_read"))
            .and_then(|r| r.as_u64())
            .unwrap_or(0);

        Ok(QueryResult {
            rows: Vec::new(),
            affected_rows: Some(affected),
            columns: Vec::new(),
        })
    }

    async fn query(&self, query: &str, _params: &QueryParams) -> Result<QueryResult> {
        let result = self.execute_query(query, "JSON").await?;

        let columns: Vec<ColumnInfo> = result
            .get("meta")
            .and_then(|m| m.as_array())
            .map(|arr| {
                arr.iter().map(|col| {
                    ColumnInfo {
                        name: col.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                        data_type: col.get("type").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                        nullable: true,
                    }
                }).collect()
            })
            .unwrap_or_default();

        let rows: Vec<HashMap<String, serde_json::Value>> = result
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter().filter_map(|row| {
                    row.as_object().map(|obj| {
                        obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect()
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(QueryResult {
            rows,
            affected_rows: None,
            columns,
        })
    }

    async fn list_tables(&self, _schema: Option<&str>) -> Result<Vec<TableInfo>> {
        let query = "SHOW TABLES";
        let result = self.execute_query(query, "JSON").await?;

        let tables: Vec<TableInfo> = result
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter().filter_map(|row| {
                    row.get("name").and_then(|n| n.as_str()).map(|name| {
                        TableInfo {
                            name: name.to_string(),
                            schema: Some(self.database.clone()),
                            columns: Vec::new(),
                            primary_key: None,
                            row_count: None,
                        }
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(tables)
    }

    async fn describe_table(&self, table: &str, _schema: Option<&str>) -> Result<TableInfo> {
        let query = format!("DESCRIBE TABLE {}", table);
        let result = self.execute_query(&query, "JSON").await?;

        let columns: Vec<ColumnInfo> = result
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter().map(|row| {
                    ColumnInfo {
                        name: row.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string(),
                        data_type: row.get("type").and_then(|t| t.as_str()).unwrap_or("").to_string(),
                        nullable: row.get("default_type").and_then(|d| d.as_str()).map(|s| s.is_empty()).unwrap_or(true),
                    }
                }).collect()
            })
            .unwrap_or_default();

        Ok(TableInfo {
            name: table.to_string(),
            schema: Some(self.database.clone()),
            columns,
            primary_key: None,
            row_count: None,
        })
    }

    async fn list_indexes(&self, table: &str) -> Result<Vec<IndexInfo>> {
        let query = format!(
            "SELECT name, expr, type FROM system.data_skipping_indices WHERE table = '{}'",
            table
        );

        let result = self.execute_query(&query, "JSON").await?;

        let indexes: Vec<IndexInfo> = result
            .get("data")
            .and_then(|d| d.as_array())
            .map(|arr| {
                arr.iter().filter_map(|row| {
                    row.get("name").and_then(|n| n.as_str()).map(|name| {
                        IndexInfo {
                            name: name.to_string(),
                            table: table.to_string(),
                            columns: Vec::new(),
                            unique: false,
                            index_type: row.get("type").and_then(|t| t.as_str()).map(String::from),
                        }
                    })
                }).collect()
            })
            .unwrap_or_default();

        Ok(indexes)
    }
}
