use crate::{Error, Result, SearchOptions, SearchProvider, SearchResponse, SearchResult, ContentExtractor, ExtractedContent};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const BASE_URL: &str = "https://en.wikipedia.org/w/api.php";

pub struct WikipediaClient {
    client: Client,
    language: String,
}

impl WikipediaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            language: "en".to_string(),
        }
    }

    pub fn with_language(mut self, language: &str) -> Self {
        self.language = language.to_string();
        self
    }

    fn base_url(&self) -> String {
        format!("https://{}.wikipedia.org/w/api.php", self.language)
    }

    pub async fn get_page(&self, title: &str) -> Result<WikiPage> {
        let response = self.client
            .get(&self.base_url())
            .query(&[
                ("action", "query"),
                ("titles", title),
                ("prop", "extracts|info|pageimages"),
                ("exintro", "true"),
                ("explaintext", "true"),
                ("inprop", "url"),
                ("pithumbsize", "500"),
                ("format", "json"),
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

        let wiki_response: WikiQueryResponse = response.json().await?;

        let (_, page) = wiki_response.query.pages.into_iter().next()
            .ok_or_else(|| Error::NotFound(title.to_string()))?;

        if page.missing.is_some() {
            return Err(Error::NotFound(title.to_string()));
        }

        Ok(WikiPage {
            page_id: page.pageid.unwrap_or(0),
            title: page.title,
            extract: page.extract,
            url: page.fullurl,
            thumbnail: page.thumbnail.map(|t| t.source),
        })
    }

    pub async fn get_full_content(&self, title: &str) -> Result<WikiFullContent> {
        let response = self.client
            .get(&self.base_url())
            .query(&[
                ("action", "query"),
                ("titles", title),
                ("prop", "extracts|info|categories|links|images"),
                ("explaintext", "true"),
                ("inprop", "url"),
                ("cllimit", "50"),
                ("pllimit", "100"),
                ("imlimit", "50"),
                ("format", "json"),
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

        let wiki_response: WikiFullQueryResponse = response.json().await?;

        let (_, page) = wiki_response.query.pages.into_iter().next()
            .ok_or_else(|| Error::NotFound(title.to_string()))?;

        if page.missing.is_some() {
            return Err(Error::NotFound(title.to_string()));
        }

        Ok(WikiFullContent {
            page_id: page.pageid.unwrap_or(0),
            title: page.title,
            content: page.extract.unwrap_or_default(),
            url: page.fullurl,
            categories: page.categories.unwrap_or_default().into_iter().map(|c| c.title).collect(),
            links: page.links.unwrap_or_default().into_iter().map(|l| l.title).collect(),
            images: page.images.unwrap_or_default().into_iter().map(|i| i.title).collect(),
        })
    }

    pub async fn get_summary(&self, title: &str) -> Result<WikiSummary> {
        let url = format!("https://{}.wikipedia.org/api/rest_v1/page/summary/{}",
            self.language,
            urlencoding::encode(title)
        );

        let response = self.client
            .get(&url)
            .header("Accept", "application/json")
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

        let summary: WikiRestSummary = response.json().await?;

        Ok(WikiSummary {
            title: summary.title,
            display_title: summary.displaytitle,
            extract: summary.extract,
            extract_html: summary.extract_html,
            description: summary.description,
            thumbnail: summary.thumbnail.map(|t| WikiThumbnail {
                source: t.source,
                width: t.width,
                height: t.height,
            }),
            content_urls: summary.content_urls.map(|cu| WikiContentUrls {
                desktop: cu.desktop.page,
                mobile: cu.mobile.page,
            }),
        })
    }

    pub async fn random(&self, count: u32) -> Result<Vec<WikiPage>> {
        let response = self.client
            .get(&self.base_url())
            .query(&[
                ("action", "query"),
                ("list", "random"),
                ("rnnamespace", "0"),
                ("rnlimit", &count.to_string()),
                ("format", "json"),
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

        let random_response: WikiRandomResponse = response.json().await?;

        let mut pages = Vec::new();
        for random_page in random_response.query.random {
            if let Ok(page) = self.get_page(&random_page.title).await {
                pages.push(page);
            }
        }

        Ok(pages)
    }
}

impl Default for WikipediaClient {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct WikiPage {
    pub page_id: u64,
    pub title: String,
    pub extract: Option<String>,
    pub url: Option<String>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WikiFullContent {
    pub page_id: u64,
    pub title: String,
    pub content: String,
    pub url: Option<String>,
    pub categories: Vec<String>,
    pub links: Vec<String>,
    pub images: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WikiSummary {
    pub title: String,
    pub display_title: Option<String>,
    pub extract: String,
    pub extract_html: Option<String>,
    pub description: Option<String>,
    pub thumbnail: Option<WikiThumbnail>,
    pub content_urls: Option<WikiContentUrls>,
}

#[derive(Debug, Clone)]
pub struct WikiThumbnail {
    pub source: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct WikiContentUrls {
    pub desktop: String,
    pub mobile: String,
}

#[derive(Deserialize)]
struct WikiQueryResponse {
    query: WikiQueryData,
}

#[derive(Deserialize)]
struct WikiQueryData {
    pages: std::collections::HashMap<String, WikiPageData>,
}

#[derive(Deserialize)]
struct WikiPageData {
    pageid: Option<u64>,
    title: String,
    extract: Option<String>,
    fullurl: Option<String>,
    missing: Option<String>,
    thumbnail: Option<WikiThumbnailData>,
}

#[derive(Deserialize)]
struct WikiThumbnailData {
    source: String,
}

#[derive(Deserialize)]
struct WikiFullQueryResponse {
    query: WikiFullQueryData,
}

#[derive(Deserialize)]
struct WikiFullQueryData {
    pages: std::collections::HashMap<String, WikiFullPageData>,
}

#[derive(Deserialize)]
struct WikiFullPageData {
    pageid: Option<u64>,
    title: String,
    extract: Option<String>,
    fullurl: Option<String>,
    missing: Option<String>,
    categories: Option<Vec<WikiCategory>>,
    links: Option<Vec<WikiLink>>,
    images: Option<Vec<WikiImage>>,
}

#[derive(Deserialize)]
struct WikiCategory {
    title: String,
}

#[derive(Deserialize)]
struct WikiLink {
    title: String,
}

#[derive(Deserialize)]
struct WikiImage {
    title: String,
}

#[derive(Deserialize)]
struct WikiSearchResponse {
    query: WikiSearchQueryData,
}

#[derive(Deserialize)]
struct WikiSearchQueryData {
    search: Vec<WikiSearchResult>,
    searchinfo: Option<WikiSearchInfo>,
}

#[derive(Deserialize)]
struct WikiSearchInfo {
    totalhits: Option<u64>,
}

#[derive(Deserialize)]
struct WikiSearchResult {
    pageid: u64,
    title: String,
    snippet: Option<String>,
    timestamp: Option<String>,
}

#[derive(Deserialize)]
struct WikiRandomResponse {
    query: WikiRandomQueryData,
}

#[derive(Deserialize)]
struct WikiRandomQueryData {
    random: Vec<WikiRandomPage>,
}

#[derive(Deserialize)]
struct WikiRandomPage {
    title: String,
}

#[derive(Deserialize)]
struct WikiRestSummary {
    title: String,
    displaytitle: Option<String>,
    extract: String,
    extract_html: Option<String>,
    description: Option<String>,
    thumbnail: Option<WikiRestThumbnail>,
    content_urls: Option<WikiRestContentUrls>,
}

#[derive(Deserialize)]
struct WikiRestThumbnail {
    source: String,
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
struct WikiRestContentUrls {
    desktop: WikiRestUrl,
    mobile: WikiRestUrl,
}

#[derive(Deserialize)]
struct WikiRestUrl {
    page: String,
}

#[async_trait]
impl SearchProvider for WikipediaClient {
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResponse> {
        let limit = options.max_results.unwrap_or(10);

        let response = self.client
            .get(&self.base_url())
            .query(&[
                ("action", "query"),
                ("list", "search"),
                ("srsearch", query),
                ("srlimit", &limit.to_string()),
                ("srprop", "snippet|timestamp"),
                ("format", "json"),
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

        let wiki_response: WikiSearchResponse = response.json().await?;

        let results: Vec<SearchResult> = wiki_response.query.search.into_iter()
            .enumerate()
            .map(|(i, r)| {
                let url = format!("https://{}.wikipedia.org/wiki/{}",
                    self.language,
                    urlencoding::encode(&r.title.replace(' ', "_"))
                );
                let snippet = r.snippet.map(|s| {
                    s.replace("<span class=\"searchmatch\">", "")
                        .replace("</span>", "")
                        .replace("&quot;", "\"")
                        .replace("&amp;", "&")
                });
                SearchResult {
                    title: r.title,
                    url,
                    snippet: snippet.clone(),
                    content: snippet,
                    score: Some(1.0 / (i + 1) as f64),
                    published_date: r.timestamp,
                }
            })
            .collect();

        let total_results = wiki_response.query.searchinfo.and_then(|si| si.totalhits);

        Ok(SearchResponse {
            query: query.to_string(),
            results,
            answer: None,
            total_results,
        })
    }
}

#[async_trait]
impl ContentExtractor for WikipediaClient {
    async fn extract(&self, url: &str) -> Result<ExtractedContent> {
        let title = url
            .rsplit('/')
            .next()
            .map(|t| urlencoding::decode(t).unwrap_or_else(|_| t.into()).to_string())
            .ok_or_else(|| Error::InvalidRequest("Invalid Wikipedia URL".to_string()))?
            .replace('_', " ");

        let full_content = self.get_full_content(&title).await?;

        Ok(ExtractedContent {
            url: full_content.url.unwrap_or_else(|| url.to_string()),
            title: Some(full_content.title),
            content: full_content.content.clone(),
            markdown: Some(full_content.content),
            links: full_content.links.into_iter().map(|l| {
                format!("https://{}.wikipedia.org/wiki/{}", self.language, urlencoding::encode(&l.replace(' ', "_")))
            }).collect(),
            images: full_content.images,
        })
    }

    async fn extract_many(&self, urls: &[&str]) -> Result<Vec<ExtractedContent>> {
        let mut results = Vec::new();
        for url in urls {
            match self.extract(url).await {
                Ok(content) => results.push(content),
                Err(_) => continue,
            }
        }
        Ok(results)
    }
}
