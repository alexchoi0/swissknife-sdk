use crate::{Error, Result, SearchOptions, SearchProvider, SearchResponse, SearchResult, TimeRange};
use async_trait::async_trait;
use reqwest::Client;
use serde::Deserialize;

const BASE_URL: &str = "https://www.googleapis.com/customsearch/v1";

pub struct GoogleSearchClient {
    api_key: String,
    cx: String,
    client: Client,
}

impl GoogleSearchClient {
    pub fn new(api_key: &str, cx: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            cx: cx.to_string(),
            client: Client::new(),
        }
    }

    pub async fn search_images(&self, query: &str, options: ImageSearchOptions) -> Result<Vec<ImageResult>> {
        let mut params = vec![
            ("key", self.api_key.clone()),
            ("cx", self.cx.clone()),
            ("q", query.to_string()),
            ("searchType", "image".to_string()),
        ];

        if let Some(num) = options.num {
            params.push(("num", num.to_string()));
        }
        if let Some(start) = options.start {
            params.push(("start", start.to_string()));
        }
        if let Some(img_size) = options.img_size {
            params.push(("imgSize", img_size));
        }
        if let Some(img_type) = options.img_type {
            params.push(("imgType", img_type));
        }
        if let Some(safe) = options.safe {
            params.push(("safe", safe));
        }

        let response = self.client
            .get(BASE_URL)
            .query(&params)
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

        let google_response: GoogleSearchResponse = response.json().await?;

        Ok(google_response.items.unwrap_or_default().into_iter().filter_map(|item| {
            item.image.map(|img| ImageResult {
                title: item.title,
                link: item.link,
                display_link: item.display_link,
                context_link: img.context_link,
                thumbnail_link: img.thumbnail_link,
                height: img.height,
                width: img.width,
            })
        }).collect())
    }
}

#[derive(Default)]
pub struct ImageSearchOptions {
    pub num: Option<u32>,
    pub start: Option<u32>,
    pub img_size: Option<String>,
    pub img_type: Option<String>,
    pub safe: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ImageResult {
    pub title: String,
    pub link: String,
    pub display_link: Option<String>,
    pub context_link: Option<String>,
    pub thumbnail_link: Option<String>,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

#[derive(Deserialize)]
struct GoogleSearchResponse {
    items: Option<Vec<GoogleSearchItem>>,
    #[serde(rename = "searchInformation")]
    search_information: Option<GoogleSearchInfo>,
}

#[derive(Deserialize)]
struct GoogleSearchInfo {
    #[serde(rename = "totalResults")]
    total_results: Option<String>,
}

#[derive(Deserialize)]
struct GoogleSearchItem {
    title: String,
    link: String,
    snippet: Option<String>,
    #[serde(rename = "displayLink")]
    display_link: Option<String>,
    #[serde(rename = "pagemap")]
    page_map: Option<GooglePageMap>,
    image: Option<GoogleImageInfo>,
}

#[derive(Deserialize)]
struct GooglePageMap {
    metatags: Option<Vec<GoogleMetatag>>,
}

#[derive(Deserialize)]
struct GoogleMetatag {
    #[serde(rename = "article:published_time")]
    published_time: Option<String>,
    #[serde(rename = "og:title")]
    og_title: Option<String>,
    #[serde(rename = "og:description")]
    og_description: Option<String>,
}

#[derive(Deserialize)]
struct GoogleImageInfo {
    #[serde(rename = "contextLink")]
    context_link: Option<String>,
    #[serde(rename = "thumbnailLink")]
    thumbnail_link: Option<String>,
    height: Option<u32>,
    width: Option<u32>,
}

#[async_trait]
impl SearchProvider for GoogleSearchClient {
    async fn search(&self, query: &str, options: &SearchOptions) -> Result<SearchResponse> {
        let mut params = vec![
            ("key", self.api_key.clone()),
            ("cx", self.cx.clone()),
            ("q", query.to_string()),
        ];

        if let Some(num) = options.max_results {
            params.push(("num", num.min(10).to_string()));
        }

        let date_restrict = options.time_range.map(|t| match t {
            TimeRange::Day => "d1".to_string(),
            TimeRange::Week => "w1".to_string(),
            TimeRange::Month => "m1".to_string(),
            TimeRange::Year => "y1".to_string(),
        });

        if let Some(dr) = date_restrict {
            params.push(("dateRestrict", dr));
        }

        if !options.include_domains.is_empty() {
            let site_search = options.include_domains.join(" OR site:");
            params.push(("siteSearch", format!("site:{}", site_search)));
        }

        if !options.exclude_domains.is_empty() {
            let exclude = options.exclude_domains.iter()
                .map(|d| format!("-site:{}", d))
                .collect::<Vec<_>>()
                .join(" ");
            let current_query = params.iter().find(|(k, _)| *k == "q").map(|(_, v)| v.clone()).unwrap();
            params.retain(|(k, _)| *k != "q");
            params.push(("q", format!("{} {}", current_query, exclude)));
        }

        let response = self.client
            .get(BASE_URL)
            .query(&params)
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

        let google_response: GoogleSearchResponse = response.json().await?;

        let results = google_response.items.unwrap_or_default().into_iter().enumerate().map(|(i, item)| {
            let published_date = item.page_map
                .and_then(|pm| pm.metatags)
                .and_then(|mt| mt.first().cloned())
                .and_then(|m| m.published_time);

            SearchResult {
                title: item.title,
                url: item.link,
                snippet: item.snippet.clone(),
                content: item.snippet,
                score: Some(1.0 / (i + 1) as f64),
                published_date,
            }
        }).collect();

        let total_results = google_response.search_information
            .and_then(|si| si.total_results)
            .and_then(|tr| tr.parse().ok());

        Ok(SearchResponse {
            query: query.to_string(),
            results,
            answer: None,
            total_results,
        })
    }
}
