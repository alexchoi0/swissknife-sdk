use crate::{Error, Result};
use crate::cursor::CursorClient;
use serde::{Deserialize, Serialize};

impl CursorClient {
    pub async fn get_workspace_info(&self) -> Result<WorkspaceInfo> {
        let response = self.client()
            .get(format!("{}/workspace/info", self.base_url()))
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

        let result: WorkspaceInfo = response.json().await?;
        Ok(result)
    }

    pub async fn get_file_content(&self, file_path: &str) -> Result<FileInfo> {
        let response = self.client()
            .get(format!("{}/files/content", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .query(&[("path", file_path)])
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

        let result: FileInfo = response.json().await?;
        Ok(result)
    }

    pub async fn list_files(&self, directory: Option<&str>, pattern: Option<&str>) -> Result<Vec<FileEntry>> {
        let mut request = self.client()
            .get(format!("{}/files/list", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()));

        let mut query: Vec<(&str, &str)> = Vec::new();
        if let Some(dir) = directory {
            query.push(("directory", dir));
        }
        if let Some(pat) = pattern {
            query.push(("pattern", pat));
        }
        if !query.is_empty() {
            request = request.query(&query);
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

        let result: FilesListResponse = response.json().await?;
        Ok(result.files)
    }

    pub async fn search_symbols(&self, query: &str, file_filter: Option<&str>) -> Result<Vec<Symbol>> {
        let mut request = self.client()
            .get(format!("{}/symbols/search", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .query(&[("query", query)]);

        if let Some(filter) = file_filter {
            request = request.query(&[("file_filter", filter)]);
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

        let result: SymbolsResponse = response.json().await?;
        Ok(result.symbols)
    }

    pub async fn get_symbol_definition(&self, symbol_name: &str, file_path: Option<&str>) -> Result<SymbolDefinition> {
        let mut request = self.client()
            .get(format!("{}/symbols/definition", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .query(&[("name", symbol_name)]);

        if let Some(path) = file_path {
            request = request.query(&[("file_path", path)]);
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

        let result: SymbolDefinition = response.json().await?;
        Ok(result)
    }

    pub async fn get_references(&self, symbol_name: &str, file_path: &str, line: u32) -> Result<Vec<Reference>> {
        let response = self.client()
            .get(format!("{}/symbols/references", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .query(&[
                ("name", symbol_name),
                ("file_path", file_path),
                ("line", &line.to_string()),
            ])
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

        let result: ReferencesResponse = response.json().await?;
        Ok(result.references)
    }

    pub async fn get_diagnostics(&self, file_path: Option<&str>) -> Result<Vec<Diagnostic>> {
        let mut request = self.client()
            .get(format!("{}/diagnostics", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()));

        if let Some(path) = file_path {
            request = request.query(&[("file_path", path)]);
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

        let result: DiagnosticsResponse = response.json().await?;
        Ok(result.diagnostics)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceInfo {
    pub name: String,
    pub root_path: String,
    pub folders: Vec<WorkspaceFolder>,
    pub active_file: Option<String>,
    pub git_info: Option<GitInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WorkspaceFolder {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitInfo {
    pub branch: Option<String>,
    pub remote_url: Option<String>,
    pub has_changes: bool,
    pub ahead: Option<u32>,
    pub behind: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub content: String,
    pub language: Option<String>,
    pub line_count: u32,
    pub size_bytes: u64,
    pub encoding: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FileEntry {
    pub path: String,
    pub name: String,
    pub is_directory: bool,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FilesListResponse {
    pub files: Vec<FileEntry>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: String,
    pub start_line: u32,
    pub end_line: u32,
    pub container: Option<String>,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    File,
    Module,
    Namespace,
    Package,
    Class,
    Method,
    Property,
    Field,
    Constructor,
    Enum,
    Interface,
    Function,
    Variable,
    Constant,
    String,
    Number,
    Boolean,
    Array,
    Object,
    Key,
    Null,
    EnumMember,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SymbolsResponse {
    pub symbols: Vec<Symbol>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SymbolDefinition {
    pub symbol: Symbol,
    pub documentation: Option<String>,
    pub signature: Option<String>,
    pub source_code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Reference {
    pub file_path: String,
    pub line: u32,
    pub column: u32,
    pub preview: Option<String>,
    pub kind: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ReferencesResponse {
    pub references: Vec<Reference>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Diagnostic {
    pub file_path: String,
    pub message: String,
    pub severity: DiagnosticSeverity,
    pub start_line: u32,
    pub start_column: u32,
    pub end_line: u32,
    pub end_column: u32,
    pub source: Option<String>,
    pub code: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiagnosticsResponse {
    pub diagnostics: Vec<Diagnostic>,
}
