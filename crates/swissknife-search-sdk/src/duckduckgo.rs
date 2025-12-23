use crate::{Error, Result, SearchOptions, SearchProvider, SearchResponse, SearchResult, TimeRange};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const INSTANT_ANSWER_URL: &str = "https://api.duckduckgo.com";
const HTML_SEARCH_URL: &str = "https://html.duckduckgo.com/html";

pub struct DuckDuckGoClient {
    client: Client,
}

impl DuckDuckGoClient {
    pub fn new() -> Self {
        Self {
            client: Client::builder()
                .user_agent("Mozilla/5.0 (compatible; SwissKnifeBot/1.0)")
                .build()
                .unwrap_or_else(|_| Client::new()),
        }
    }

    pub async fn instant_answer(&self, query: &str) -> Result<InstantAnswer> {
        let response = self.client
            .get(INSTANT_ANSWER_URL)
            .query(&[
                ("q", query),
                ("format", "json"),
                ("no_html", "1"),
                ("skip_disambig", "1"),
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

        let ddg_response: DdgInstantAnswer = response.json().await?;

        Ok(InstantAnswer {
            heading: if ddg_response.heading.is_empty() { None } else { Some(ddg_response.heading) },
            abstract_text: if ddg_response.r#abstract.is_empty() { None } else { Some(ddg_response.r#abstract) },
            abstract_source: ddg_response.abstract_source,
            abstract_url: if ddg_response.abstract_url.is_empty() { None } else { Some(ddg_response.abstract_url) },
            answer: if ddg_response.answer.is_empty() { None } else { Some(ddg_response.answer) },
            answer_type: if ddg_response.answer_type.is_empty() { None } else { Some(ddg_response.answer_type) },
            definition: if ddg_response.definition.is_empty() { None } else { Some(ddg_response.definition) },
            definition_source: ddg_response.definition_source,
            definition_url: if ddg_response.definition_url.is_empty() { None } else { Some(ddg_response.definition_url) },
            image: if ddg_response.image.is_empty() { None } else { Some(ddg_response.image) },
            related_topics: ddg_response.related_topics.into_iter().filter_map(|t| {
                if let Some(text) = t.text {
                    Some(RelatedTopic {
                        text,
                        first_url: t.first_url,
                    })
                } else {
                    None
                }
            }).collect(),
            results: ddg_response.results.into_iter().map(|r| DdgResult {
                text: r.text,
                first_url: r.first_url,
            }).collect(),
        })
    }
}

impl Default for DuckDuckGoClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Deserialize)]
struct DdgInstantAnswer {
    #[serde(rename = "Heading", default)]
    heading: String,
    #[serde(rename = "Abstract", default)]
    r#abstract: String,
    #[serde(rename = "AbstractSource", default)]
    abstract_source: Option<String>,
    #[serde(rename = "AbstractURL", default)]
    abstract_url: String,
    #[serde(rename = "Answer", default)]
    answer: String,
    #[serde(rename = "AnswerType", default)]
    answer_type: String,
    #[serde(rename = "Definition", default)]
    definition: String,
    #[serde(rename = "DefinitionSource", default)]
    definition_source: Option<String>,
    #[serde(rename = "DefinitionURL", default)]
    definition_url: String,
    #[serde(rename = "Image", default)]
    image: String,
    #[serde(rename = "RelatedTopics", default)]
    related_topics: Vec<DdgRelatedTopic>,
    #[serde(rename = "Results", default)]
    results: Vec<DdgResultItem>,
}

#[derive(Deserialize)]
struct DdgRelatedTopic {
    #[serde(rename = "Text")]
    text: Option<String>,
    #[serde(rename = "FirstURL")]
    first_url: Option<String>,
}

#[derive(Deserialize)]
struct DdgResultItem {
    #[serde(rename = "Text", default)]
    text: String,
    #[serde(rename = "FirstURL", default)]
    first_url: String,
}

#[derive(Debug, Clone)]
pub struct InstantAnswer {
    pub heading: Option<String>,
    pub abstract_text: Option<String>,
    pub abstract_source: Option<String>,
    pub abstract_url: Option<String>,
    pub answer: Option<String>,
    pub answer_type: Option<String>,
    pub definition: Option<String>,
    pub definition_source: Option<String>,
    pub definition_url: Option<String>,
    pub image: Option<String>,
    pub related_topics: Vec<RelatedTopic>,
    pub results: Vec<DdgResult>,
}

#[derive(Debug, Clone)]
pub struct RelatedTopic {
    pub text: String,
    pub first_url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DdgResult {
    pub text: String,
    pub first_url: String,
}

#[async_trait]
impl SearchProvider for DuckDuckGoClient {
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResponse> {
        let instant = self.instant_answer(query).await?;

        let mut results = Vec::new();

        if let Some(url) = &instant.abstract_url {
            if !url.is_empty() {
                results.push(SearchResult {
                    title: instant.heading.clone().unwrap_or_else(|| "Abstract".to_string()),
                    url: url.clone(),
                    snippet: instant.abstract_text.clone(),
                    content: instant.abstract_text.clone(),
                    score: Some(1.0),
                    published_date: None,
                });
            }
        }

        if let Some(url) = &instant.definition_url {
            if !url.is_empty() {
                results.push(SearchResult {
                    title: "Definition".to_string(),
                    url: url.clone(),
                    snippet: instant.definition.clone(),
                    content: instant.definition.clone(),
                    score: Some(0.9),
                    published_date: None,
                });
            }
        }

        for (i, result) in instant.results.iter().enumerate() {
            if !result.first_url.is_empty() {
                results.push(SearchResult {
                    title: result.text.chars().take(100).collect(),
                    url: result.first_url.clone(),
                    snippet: Some(result.text.clone()),
                    content: Some(result.text.clone()),
                    score: Some(0.8 - i as f64 * 0.1),
                    published_date: None,
                });
            }
        }

        for (i, topic) in instant.related_topics.iter().enumerate() {
            if let Some(url) = &topic.first_url {
                if !url.is_empty() {
                    results.push(SearchResult {
                        title: topic.text.chars().take(100).collect(),
                        url: url.clone(),
                        snippet: Some(topic.text.clone()),
                        content: Some(topic.text.clone()),
                        score: Some(0.5 - i as f64 * 0.05),
                        published_date: None,
                    });
                }
            }
        }

        if let Some(max) = options.max_results {
            results.truncate(max as usize);
        }

        let answer = instant.answer.or(instant.abstract_text);

        Ok(SearchResponse {
            query: query.to_string(),
            results,
            answer,
            total_results: None,
        })
    }
}
