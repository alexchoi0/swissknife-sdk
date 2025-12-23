use crate::{Error, Result, SearchOptions, SearchProvider, SearchResponse, SearchResult};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.perplexity.ai";

pub struct PerplexityClient {
    api_key: String,
    client: Client,
    model: String,
}

impl PerplexityClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
            model: "llama-3.1-sonar-small-128k-online".to_string(),
        }
    }

    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    pub async fn chat(&self, messages: Vec<Message>, options: ChatOptions) -> Result<ChatResponse> {
        let request = PerplexityChatRequest {
            model: options.model.unwrap_or_else(|| self.model.clone()),
            messages,
            max_tokens: options.max_tokens,
            temperature: options.temperature,
            search_domain_filter: options.search_domain_filter,
            return_citations: options.return_citations,
            search_recency_filter: options.search_recency_filter,
        };

        let response = self.client
            .post(format!("{}/chat/completions", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
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

        let chat_response: PerplexityChatResponse = response.json().await?;

        Ok(ChatResponse {
            content: chat_response.choices.first()
                .map(|c| c.message.content.clone())
                .unwrap_or_default(),
            citations: chat_response.citations.unwrap_or_default(),
            model: chat_response.model,
            usage: chat_response.usage.map(|u| Usage {
                prompt_tokens: u.prompt_tokens,
                completion_tokens: u.completion_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

impl Message {
    pub fn system(content: &str) -> Self {
        Self { role: "system".to_string(), content: content.to_string() }
    }

    pub fn user(content: &str) -> Self {
        Self { role: "user".to_string(), content: content.to_string() }
    }

    pub fn assistant(content: &str) -> Self {
        Self { role: "assistant".to_string(), content: content.to_string() }
    }
}

#[derive(Default)]
pub struct ChatOptions {
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub search_domain_filter: Option<Vec<String>>,
    pub return_citations: Option<bool>,
    pub search_recency_filter: Option<String>,
}

pub struct ChatResponse {
    pub content: String,
    pub citations: Vec<String>,
    pub model: String,
    pub usage: Option<Usage>,
}

pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

#[derive(Serialize)]
struct PerplexityChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_domain_filter: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    return_citations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    search_recency_filter: Option<String>,
}

#[derive(Deserialize)]
struct PerplexityChatResponse {
    model: String,
    choices: Vec<PerplexityChoice>,
    citations: Option<Vec<String>>,
    usage: Option<PerplexityUsage>,
}

#[derive(Deserialize)]
struct PerplexityChoice {
    message: PerplexityMessage,
}

#[derive(Deserialize)]
struct PerplexityMessage {
    content: String,
}

#[derive(Deserialize)]
struct PerplexityUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

#[async_trait]
impl SearchProvider for PerplexityClient {
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResponse> {
        let search_recency_filter = options.time_range.map(|t| match t {
            crate::TimeRange::Day => "day".to_string(),
            crate::TimeRange::Week => "week".to_string(),
            crate::TimeRange::Month => "month".to_string(),
            crate::TimeRange::Year => "year".to_string(),
        });

        let domain_filter = if !options.include_domains.is_empty() {
            Some(options.include_domains.clone())
        } else {
            None
        };

        let chat_options = ChatOptions {
            model: Some(self.model.clone()),
            max_tokens: Some(1024),
            temperature: Some(0.0),
            search_domain_filter: domain_filter,
            return_citations: Some(true),
            search_recency_filter,
        };

        let messages = vec![
            Message::system("You are a helpful search assistant. Provide concise, accurate answers based on web search results."),
            Message::user(query),
        ];

        let response = self.chat(messages, chat_options).await?;

        let results: Vec<SearchResult> = response.citations.iter().enumerate().map(|(i, url)| {
            SearchResult {
                title: format!("Citation {}", i + 1),
                url: url.clone(),
                snippet: None,
                content: None,
                score: Some(1.0 / (i + 1) as f64),
                published_date: None,
            }
        }).collect();

        Ok(SearchResponse {
            query: query.to_string(),
            results,
            answer: Some(response.content),
            total_results: None,
        })
    }
}
