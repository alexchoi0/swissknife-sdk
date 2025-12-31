use chrono::{DateTime, Utc};
use duckdb::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use super::{Action, ActionType, MemoryConfig, SearchResult, Session};
use crate::{Error, Result};

pub struct DuckDBMemory {
    conn: Arc<Mutex<Connection>>,
    embedding_dim: usize,
}

impl DuckDBMemory {
    pub fn new(config: MemoryConfig) -> Result<Self> {
        let db_path = config.db_path.map(PathBuf::from).unwrap_or_else(|| {
            dirs::data_local_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("swissknife")
                .join("memory.duckdb")
        });

        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).ok();
        }

        let conn = Connection::open(&db_path).map_err(|e| Error::Internal(e.to_string()))?;
        let memory = Self {
            conn: Arc::new(Mutex::new(conn)),
            embedding_dim: config.embedding_dim,
        };
        memory.init_schema()?;
        Ok(memory)
    }

    pub fn in_memory(config: MemoryConfig) -> Result<Self> {
        let conn = Connection::open_in_memory().map_err(|e| Error::Internal(e.to_string()))?;
        let memory = Self {
            conn: Arc::new(Mutex::new(conn)),
            embedding_dim: config.embedding_dim,
        };
        memory.init_schema()?;
        Ok(memory)
    }

    fn init_schema(&self) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;

        if let Err(e) = conn.execute_batch("INSTALL vss; LOAD vss;") {
            eprintln!("Note: VSS extension not available ({})", e);
        }

        let schema = format!(
            r#"
            CREATE TABLE IF NOT EXISTS sessions (
                id VARCHAR PRIMARY KEY,
                session_id VARCHAR NOT NULL UNIQUE,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                title VARCHAR
            );

            CREATE TABLE IF NOT EXISTS actions (
                id VARCHAR PRIMARY KEY,
                session_id VARCHAR NOT NULL,
                sequence BIGINT NOT NULL,
                action_type VARCHAR NOT NULL,
                role VARCHAR,
                content TEXT NOT NULL,
                tool_name VARCHAR,
                tool_input TEXT,
                tool_call_id VARCHAR,
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (session_id) REFERENCES sessions(session_id)
            );

            CREATE TABLE IF NOT EXISTS embeddings (
                id VARCHAR PRIMARY KEY,
                action_id VARCHAR NOT NULL UNIQUE,
                embedding FLOAT[{dim}],
                created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
                FOREIGN KEY (action_id) REFERENCES actions(id)
            );

            CREATE INDEX IF NOT EXISTS idx_sessions_session_id ON sessions(session_id);
            CREATE INDEX IF NOT EXISTS idx_actions_session_id ON actions(session_id, sequence);
            CREATE INDEX IF NOT EXISTS idx_actions_type ON actions(action_type);
            CREATE INDEX IF NOT EXISTS idx_actions_tool_name ON actions(tool_name);
            "#,
            dim = self.embedding_dim
        );

        conn.execute_batch(&schema)
            .map_err(|e| Error::Internal(e.to_string()))?;

        Ok(())
    }

    pub fn create_session(&self, session_id: &str, title: Option<&str>) -> Result<String> {
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        conn.execute(
            "INSERT INTO sessions (id, session_id, created_at, updated_at, title) VALUES (?, ?, ?, ?, ?)",
            params![id, session_id, now.to_rfc3339(), now.to_rfc3339(), title],
        ).map_err(|e| Error::Internal(e.to_string()))?;
        Ok(id)
    }

    pub fn get_session(&self, session_id: &str) -> Result<Option<Session>> {
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, session_id, created_at::VARCHAR, updated_at::VARCHAR, title FROM sessions WHERE session_id = ?")
            .map_err(|e| Error::Internal(e.to_string()))?;
        let mut rows = stmt.query(params![session_id]).map_err(|e| Error::Internal(e.to_string()))?;

        if let Some(row) = rows.next().map_err(|e| Error::Internal(e.to_string()))? {
            Ok(Some(self.parse_session(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn get_or_create_session(&self, session_id: &str) -> Result<Session> {
        if let Some(session) = self.get_session(session_id)? {
            return Ok(session);
        }
        self.create_session(session_id, None)?;
        self.get_session(session_id)?
            .ok_or_else(|| Error::Internal("Failed to create session".to_string()))
    }

    pub fn list_sessions(&self, limit: usize) -> Result<Vec<Session>> {
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT id, session_id, created_at::VARCHAR, updated_at::VARCHAR, title FROM sessions ORDER BY updated_at DESC LIMIT ?")
            .map_err(|e| Error::Internal(e.to_string()))?;
        let mut rows = stmt.query(params![limit as i64]).map_err(|e| Error::Internal(e.to_string()))?;

        let mut sessions = Vec::new();
        while let Some(row) = rows.next().map_err(|e| Error::Internal(e.to_string()))? {
            sessions.push(self.parse_session(row)?);
        }
        Ok(sessions)
    }

    pub fn update_session_title(&self, session_id: &str, title: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
        let now = Utc::now();
        conn.execute(
            "UPDATE sessions SET title = ?, updated_at = ? WHERE session_id = ?",
            params![title, now.to_rfc3339(), session_id],
        ).map_err(|e| Error::Internal(e.to_string()))?;
        Ok(())
    }

    fn get_next_sequence(&self, session_id: &str) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT COALESCE(MAX(sequence), 0) + 1 FROM actions WHERE session_id = ?")
            .map_err(|e| Error::Internal(e.to_string()))?;
        let mut rows = stmt.query(params![session_id]).map_err(|e| Error::Internal(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| Error::Internal(e.to_string()))? {
            row.get(0).map_err(|e| Error::Internal(e.to_string()))
        } else {
            Ok(1)
        }
    }

    fn touch_session(&self, session_id: &str) -> Result<()> {
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
        let now = Utc::now();
        conn.execute(
            "UPDATE sessions SET updated_at = ? WHERE session_id = ?",
            params![now.to_rfc3339(), session_id],
        ).map_err(|e| Error::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn add_message(&self, session_id: &str, role: &str, content: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let sequence = self.get_next_sequence(session_id)?;
        let now = Utc::now();
        {
            let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
            conn.execute(
                "INSERT INTO actions (id, session_id, sequence, action_type, role, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![id, session_id, sequence, "message", role, content, now.to_rfc3339(), now.to_rfc3339()],
            ).map_err(|e| Error::Internal(e.to_string()))?;
        }
        self.touch_session(session_id)?;
        Ok(id)
    }

    pub fn add_tool_call(&self, session_id: &str, tool_name: &str, tool_input: &str, tool_call_id: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let sequence = self.get_next_sequence(session_id)?;
        let now = Utc::now();
        let content = format!("{}({})", tool_name, tool_input);
        {
            let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
            conn.execute(
                "INSERT INTO actions (id, session_id, sequence, action_type, content, tool_name, tool_input, tool_call_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
                params![id, session_id, sequence, "tool_call", content, tool_name, tool_input, tool_call_id, now.to_rfc3339(), now.to_rfc3339()],
            ).map_err(|e| Error::Internal(e.to_string()))?;
        }
        self.touch_session(session_id)?;
        Ok(id)
    }

    pub fn add_tool_result(&self, session_id: &str, tool_call_id: &str, content: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let sequence = self.get_next_sequence(session_id)?;
        let now = Utc::now();
        {
            let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
            conn.execute(
                "INSERT INTO actions (id, session_id, sequence, action_type, content, tool_call_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                params![id, session_id, sequence, "tool_result", content, tool_call_id, now.to_rfc3339(), now.to_rfc3339()],
            ).map_err(|e| Error::Internal(e.to_string()))?;
        }
        self.touch_session(session_id)?;
        Ok(id)
    }

    pub fn add_thinking(&self, session_id: &str, content: &str) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let sequence = self.get_next_sequence(session_id)?;
        let now = Utc::now();
        {
            let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
            conn.execute(
                "INSERT INTO actions (id, session_id, sequence, action_type, content, created_at, updated_at) VALUES (?, ?, ?, ?, ?, ?, ?)",
                params![id, session_id, sequence, "thinking", content, now.to_rfc3339(), now.to_rfc3339()],
            ).map_err(|e| Error::Internal(e.to_string()))?;
        }
        self.touch_session(session_id)?;
        Ok(id)
    }

    pub fn get_actions(&self, session_id: &str) -> Result<Vec<Action>> {
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, sequence, action_type, role, content, tool_name, tool_input, tool_call_id, created_at::VARCHAR, updated_at::VARCHAR
                 FROM actions WHERE session_id = ? ORDER BY sequence ASC"
            )
            .map_err(|e| Error::Internal(e.to_string()))?;
        let mut rows = stmt.query(params![session_id]).map_err(|e| Error::Internal(e.to_string()))?;
        self.parse_actions(&mut rows)
    }

    pub fn get_actions_by_type(&self, session_id: &str, action_type: ActionType) -> Result<Vec<Action>> {
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, session_id, sequence, action_type, role, content, tool_name, tool_input, tool_call_id, created_at::VARCHAR, updated_at::VARCHAR
                 FROM actions WHERE session_id = ? AND action_type = ? ORDER BY sequence ASC"
            )
            .map_err(|e| Error::Internal(e.to_string()))?;
        let mut rows = stmt.query(params![session_id, action_type.as_str()]).map_err(|e| Error::Internal(e.to_string()))?;
        self.parse_actions(&mut rows)
    }

    pub fn get_messages(&self, session_id: &str) -> Result<Vec<Action>> {
        self.get_actions_by_type(session_id, ActionType::Message)
    }

    pub fn get_tool_calls(&self, session_id: &str) -> Result<Vec<Action>> {
        self.get_actions_by_type(session_id, ActionType::ToolCall)
    }

    pub fn add_embedding(&self, action_id: &str, embedding: &[f32]) -> Result<()> {
        if embedding.len() != self.embedding_dim {
            return Err(Error::InvalidParameter(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.embedding_dim,
                embedding.len()
            )));
        }
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        let embedding_str = format!(
            "[{}]",
            embedding.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(",")
        );
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
        conn.execute(
            &format!(
                "INSERT INTO embeddings (id, action_id, embedding, created_at, updated_at) VALUES (?, ?, {}::FLOAT[{}], ?, ?)",
                embedding_str, self.embedding_dim
            ),
            params![id, action_id, now.to_rfc3339(), now.to_rfc3339()],
        ).map_err(|e| Error::Internal(e.to_string()))?;
        Ok(())
    }

    pub fn search_similar(&self, query_embedding: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        if query_embedding.len() != self.embedding_dim {
            return Err(Error::InvalidParameter(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                self.embedding_dim,
                query_embedding.len()
            )));
        }
        let embedding_str = format!(
            "[{}]",
            query_embedding.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(",")
        );
        let query = format!(
            r#"
            SELECT a.id, a.session_id, a.sequence, a.action_type, a.role, a.content,
                   a.tool_name, a.tool_input, a.tool_call_id, a.created_at::VARCHAR, a.updated_at::VARCHAR,
                   array_cosine_similarity(e.embedding, {}::FLOAT[{}]) as similarity
            FROM embeddings e
            JOIN actions a ON e.action_id = a.id
            ORDER BY similarity DESC
            LIMIT ?
            "#,
            embedding_str, self.embedding_dim
        );
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
        let mut stmt = conn.prepare(&query).map_err(|e| Error::Internal(e.to_string()))?;
        let mut rows = stmt.query(params![limit as i64]).map_err(|e| Error::Internal(e.to_string()))?;

        let mut results = Vec::new();
        while let Some(row) = rows.next().map_err(|e| Error::Internal(e.to_string()))? {
            let action = self.parse_action_row(row)?;
            let score: f64 = row.get(11).map_err(|e| Error::Internal(e.to_string()))?;
            results.push(SearchResult { action, score });
        }
        Ok(results)
    }

    pub fn action_count(&self, session_id: &str) -> Result<i64> {
        let conn = self.conn.lock().map_err(|e| Error::Internal(e.to_string()))?;
        let mut stmt = conn
            .prepare("SELECT COUNT(*) FROM actions WHERE session_id = ?")
            .map_err(|e| Error::Internal(e.to_string()))?;
        let mut rows = stmt.query(params![session_id]).map_err(|e| Error::Internal(e.to_string()))?;
        if let Some(row) = rows.next().map_err(|e| Error::Internal(e.to_string()))? {
            row.get(0).map_err(|e| Error::Internal(e.to_string()))
        } else {
            Ok(0)
        }
    }

    fn parse_session(&self, row: &duckdb::Row) -> Result<Session> {
        let created_str: String = row.get(2).map_err(|e| Error::Internal(e.to_string()))?;
        let updated_str: String = row.get(3).map_err(|e| Error::Internal(e.to_string()))?;
        Ok(Session {
            id: row.get(0).map_err(|e| Error::Internal(e.to_string()))?,
            session_id: row.get(1).map_err(|e| Error::Internal(e.to_string()))?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            title: row.get(4).map_err(|e| Error::Internal(e.to_string()))?,
        })
    }

    fn parse_action_row(&self, row: &duckdb::Row) -> Result<Action> {
        let action_type_str: String = row.get(3).map_err(|e| Error::Internal(e.to_string()))?;
        let created_str: String = row.get(9).map_err(|e| Error::Internal(e.to_string()))?;
        let updated_str: String = row.get(10).map_err(|e| Error::Internal(e.to_string()))?;
        Ok(Action {
            id: row.get(0).map_err(|e| Error::Internal(e.to_string()))?,
            session_id: row.get(1).map_err(|e| Error::Internal(e.to_string()))?,
            sequence: row.get(2).map_err(|e| Error::Internal(e.to_string()))?,
            action_type: ActionType::from_str(&action_type_str).unwrap_or(ActionType::Message),
            role: row.get(4).map_err(|e| Error::Internal(e.to_string()))?,
            content: row.get(5).map_err(|e| Error::Internal(e.to_string()))?,
            tool_name: row.get(6).map_err(|e| Error::Internal(e.to_string()))?,
            tool_input: row.get(7).map_err(|e| Error::Internal(e.to_string()))?,
            tool_call_id: row.get(8).map_err(|e| Error::Internal(e.to_string()))?,
            created_at: DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
            updated_at: DateTime::parse_from_rfc3339(&updated_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now()),
        })
    }

    fn parse_actions(&self, rows: &mut duckdb::Rows) -> Result<Vec<Action>> {
        let mut actions = Vec::new();
        while let Some(row) = rows.next().map_err(|e| Error::Internal(e.to_string()))? {
            actions.push(self.parse_action_row(row)?);
        }
        Ok(actions)
    }
}

impl Clone for DuckDBMemory {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
            embedding_dim: self.embedding_dim,
        }
    }
}
