use crate::{Error, Result, Document, DocumentProvider};
use crate::google::GoogleClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

const DOCS_URL: &str = "https://docs.googleapis.com/v1";

impl GoogleClient {
    pub async fn get_google_doc(&self, document_id: &str) -> Result<GoogleDocument> {
        let response = self.client()
            .get(format!("{}/documents/{}", DOCS_URL, document_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let doc: GoogleDocument = response.json().await?;
        Ok(doc)
    }

    pub async fn create_google_doc(&self, title: &str) -> Result<GoogleDocument> {
        let body = serde_json::json!({
            "title": title
        });

        let response = self.client()
            .post(format!("{}/documents", DOCS_URL))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let doc: GoogleDocument = response.json().await?;
        Ok(doc)
    }

    pub async fn batch_update_doc(&self, document_id: &str, requests: Vec<DocUpdateRequest>) -> Result<BatchUpdateResponse> {
        let body = serde_json::json!({
            "requests": requests
        });

        let response = self.client()
            .post(format!("{}/documents/{}:batchUpdate", DOCS_URL, document_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let result: BatchUpdateResponse = response.json().await?;
        Ok(result)
    }

    pub async fn insert_text(&self, document_id: &str, text: &str, index: i32) -> Result<BatchUpdateResponse> {
        let requests = vec![DocUpdateRequest::InsertText {
            insert_text: InsertTextRequest {
                text: text.to_string(),
                location: Location { index },
            },
        }];
        self.batch_update_doc(document_id, requests).await
    }

    pub async fn delete_content(&self, document_id: &str, start_index: i32, end_index: i32) -> Result<BatchUpdateResponse> {
        let requests = vec![DocUpdateRequest::DeleteContentRange {
            delete_content_range: DeleteContentRangeRequest {
                range: Range {
                    start_index,
                    end_index,
                },
            },
        }];
        self.batch_update_doc(document_id, requests).await
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GoogleDocument {
    #[serde(rename = "documentId")]
    pub document_id: String,
    pub title: String,
    pub body: Option<Body>,
    #[serde(rename = "revisionId")]
    pub revision_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Body {
    pub content: Option<Vec<StructuralElement>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StructuralElement {
    #[serde(rename = "startIndex")]
    pub start_index: Option<i32>,
    #[serde(rename = "endIndex")]
    pub end_index: Option<i32>,
    pub paragraph: Option<Paragraph>,
    pub table: Option<Table>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Paragraph {
    pub elements: Option<Vec<ParagraphElement>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ParagraphElement {
    #[serde(rename = "startIndex")]
    pub start_index: Option<i32>,
    #[serde(rename = "endIndex")]
    pub end_index: Option<i32>,
    #[serde(rename = "textRun")]
    pub text_run: Option<TextRun>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TextRun {
    pub content: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Table {
    pub rows: Option<i32>,
    pub columns: Option<i32>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum DocUpdateRequest {
    InsertText {
        #[serde(rename = "insertText")]
        insert_text: InsertTextRequest,
    },
    DeleteContentRange {
        #[serde(rename = "deleteContentRange")]
        delete_content_range: DeleteContentRangeRequest,
    },
    ReplaceAllText {
        #[serde(rename = "replaceAllText")]
        replace_all_text: ReplaceAllTextRequest,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct InsertTextRequest {
    pub text: String,
    pub location: Location,
}

#[derive(Debug, Clone, Serialize)]
pub struct Location {
    pub index: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct DeleteContentRangeRequest {
    pub range: Range,
}

#[derive(Debug, Clone, Serialize)]
pub struct Range {
    #[serde(rename = "startIndex")]
    pub start_index: i32,
    #[serde(rename = "endIndex")]
    pub end_index: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ReplaceAllTextRequest {
    #[serde(rename = "containsText")]
    pub contains_text: SubstringMatchCriteria,
    #[serde(rename = "replaceText")]
    pub replace_text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SubstringMatchCriteria {
    pub text: String,
    #[serde(rename = "matchCase")]
    pub match_case: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BatchUpdateResponse {
    #[serde(rename = "documentId")]
    pub document_id: String,
    #[serde(rename = "writeControl")]
    pub write_control: Option<WriteControl>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WriteControl {
    #[serde(rename = "requiredRevisionId")]
    pub required_revision_id: Option<String>,
}

fn extract_text_content(doc: &GoogleDocument) -> String {
    let mut text = String::new();
    if let Some(body) = &doc.body {
        if let Some(content) = &body.content {
            for element in content {
                if let Some(paragraph) = &element.paragraph {
                    if let Some(elements) = &paragraph.elements {
                        for pe in elements {
                            if let Some(text_run) = &pe.text_run {
                                if let Some(content) = &text_run.content {
                                    text.push_str(content);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    text
}

pub struct GoogleDocsProvider {
    client: GoogleClient,
}

impl GoogleDocsProvider {
    pub fn new(access_token: &str) -> Self {
        Self {
            client: GoogleClient::new(access_token),
        }
    }
}

#[async_trait]
impl DocumentProvider for GoogleDocsProvider {
    async fn get_document(&self, id: &str) -> Result<Document> {
        let doc = self.client.get_google_doc(id).await?;
        let content = extract_text_content(&doc);

        Ok(Document {
            id: doc.document_id,
            title: doc.title,
            content: Some(content),
            url: Some(format!("https://docs.google.com/document/d/{}/edit", id)),
            parent_id: None,
            created_at: None,
            updated_at: None,
        })
    }

    async fn create_document(&self, _parent_id: Option<&str>, title: &str, content: Option<&str>) -> Result<Document> {
        let doc = self.client.create_google_doc(title).await?;

        if let Some(text) = content {
            if !text.is_empty() {
                self.client.insert_text(&doc.document_id, text, 1).await?;
            }
        }

        Ok(Document {
            id: doc.document_id.clone(),
            title: doc.title,
            content: content.map(|s| s.to_string()),
            url: Some(format!("https://docs.google.com/document/d/{}/edit", doc.document_id)),
            parent_id: None,
            created_at: None,
            updated_at: None,
        })
    }

    async fn update_document(&self, id: &str, title: Option<&str>, content: Option<&str>) -> Result<Document> {
        if let Some(text) = content {
            let doc = self.client.get_google_doc(id).await?;
            let current_content = extract_text_content(&doc);

            if !current_content.is_empty() {
                let end_index = current_content.len() as i32;
                if end_index > 1 {
                    self.client.delete_content(id, 1, end_index).await?;
                }
            }

            if !text.is_empty() {
                self.client.insert_text(id, text, 1).await?;
            }
        }

        self.get_document(id).await
    }

    async fn delete_document(&self, _id: &str) -> Result<()> {
        Err(Error::InvalidRequest("Google Docs cannot be deleted via API, use Drive API instead".to_string()))
    }

    async fn search(&self, _query: &str) -> Result<Vec<Document>> {
        Err(Error::InvalidRequest("Search not supported for Google Docs, use Drive API instead".to_string()))
    }
}
