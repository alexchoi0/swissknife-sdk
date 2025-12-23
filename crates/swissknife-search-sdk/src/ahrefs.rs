use crate::{Error, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const BASE_URL: &str = "https://api.ahrefs.com/v3";

pub struct AhrefsClient {
    api_key: String,
    client: Client,
}

impl AhrefsClient {
    pub fn new(api_key: &str) -> Self {
        Self {
            api_key: api_key.to_string(),
            client: Client::new(),
        }
    }

    pub async fn get_backlinks(&self, target: &str, options: BacklinksOptions) -> Result<BacklinksResponse> {
        let mut params = vec![
            ("target", target.to_string()),
            ("mode", options.mode.unwrap_or_else(|| "domain".to_string())),
            ("limit", options.limit.unwrap_or(100).to_string()),
        ];

        if let Some(offset) = options.offset {
            params.push(("offset", offset.to_string()));
        }

        let response = self.client
            .get(format!("{}/site-explorer/backlinks", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let ahrefs_response: AhrefsBacklinksResponse = response.json().await?;

        Ok(BacklinksResponse {
            backlinks: ahrefs_response.backlinks.into_iter().map(|b| Backlink {
                url_from: b.url_from,
                url_to: b.url_to,
                anchor: b.anchor,
                domain_rating: b.domain_rating,
                url_rating: b.url_rating,
                traffic: b.traffic,
                first_seen: b.first_seen,
                last_seen: b.last_seen,
                nofollow: b.nofollow,
                redirect: b.redirect,
            }).collect(),
            total: ahrefs_response.total,
        })
    }

    pub async fn get_domain_rating(&self, target: &str) -> Result<DomainRating> {
        let response = self.client
            .get(format!("{}/site-explorer/domain-rating", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .query(&[("target", target)])
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

        let dr: AhrefsDomainRating = response.json().await?;
        Ok(DomainRating {
            domain: dr.domain,
            domain_rating: dr.domain_rating,
            ahrefs_rank: dr.ahrefs_rank,
        })
    }

    pub async fn get_organic_keywords(&self, target: &str, options: KeywordsOptions) -> Result<OrganicKeywordsResponse> {
        let mut params = vec![
            ("target", target.to_string()),
            ("mode", options.mode.unwrap_or_else(|| "domain".to_string())),
            ("limit", options.limit.unwrap_or(100).to_string()),
        ];

        if let Some(country) = options.country {
            params.push(("country", country));
        }
        if let Some(offset) = options.offset {
            params.push(("offset", offset.to_string()));
        }

        let response = self.client
            .get(format!("{}/site-explorer/organic-keywords", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let ahrefs_response: AhrefsKeywordsResponse = response.json().await?;

        Ok(OrganicKeywordsResponse {
            keywords: ahrefs_response.keywords.into_iter().map(|k| OrganicKeyword {
                keyword: k.keyword,
                country: k.country,
                volume: k.volume,
                keyword_difficulty: k.keyword_difficulty,
                cpc: k.cpc,
                traffic: k.traffic,
                traffic_percentage: k.traffic_percentage,
                position: k.position,
                url: k.url,
            }).collect(),
            total: ahrefs_response.total,
        })
    }

    pub async fn get_referring_domains(&self, target: &str, options: RefDomainsOptions) -> Result<ReferringDomainsResponse> {
        let mut params = vec![
            ("target", target.to_string()),
            ("mode", options.mode.unwrap_or_else(|| "domain".to_string())),
            ("limit", options.limit.unwrap_or(100).to_string()),
        ];

        if let Some(offset) = options.offset {
            params.push(("offset", offset.to_string()));
        }

        let response = self.client
            .get(format!("{}/site-explorer/refdomains", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let ahrefs_response: AhrefsRefDomainsResponse = response.json().await?;

        Ok(ReferringDomainsResponse {
            domains: ahrefs_response.refdomains.into_iter().map(|d| ReferringDomain {
                domain: d.domain,
                domain_rating: d.domain_rating,
                backlinks: d.backlinks,
                dofollow: d.dofollow,
                first_seen: d.first_seen,
                last_seen: d.last_seen,
            }).collect(),
            total: ahrefs_response.total,
        })
    }

    pub async fn keyword_explorer(&self, keyword: &str, country: Option<&str>) -> Result<KeywordData> {
        let mut params = vec![("keyword", keyword.to_string())];
        if let Some(c) = country {
            params.push(("country", c.to_string()));
        }

        let response = self.client
            .get(format!("{}/keywords-explorer/overview", BASE_URL))
            .header("Authorization", format!("Bearer {}", self.api_key))
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

        let kw: AhrefsKeywordData = response.json().await?;
        Ok(KeywordData {
            keyword: kw.keyword,
            country: kw.country,
            volume: kw.volume,
            keyword_difficulty: kw.keyword_difficulty,
            cpc: kw.cpc,
            clicks: kw.clicks,
            global_volume: kw.global_volume,
            parent_topic: kw.parent_topic,
        })
    }
}

#[derive(Default)]
pub struct BacklinksOptions {
    pub mode: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Default)]
pub struct KeywordsOptions {
    pub mode: Option<String>,
    pub country: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Default)]
pub struct RefDomainsOptions {
    pub mode: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BacklinksResponse {
    pub backlinks: Vec<Backlink>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct Backlink {
    pub url_from: String,
    pub url_to: String,
    pub anchor: Option<String>,
    pub domain_rating: Option<f64>,
    pub url_rating: Option<f64>,
    pub traffic: Option<u64>,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
    pub nofollow: Option<bool>,
    pub redirect: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct DomainRating {
    pub domain: String,
    pub domain_rating: f64,
    pub ahrefs_rank: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct OrganicKeywordsResponse {
    pub keywords: Vec<OrganicKeyword>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct OrganicKeyword {
    pub keyword: String,
    pub country: Option<String>,
    pub volume: Option<u64>,
    pub keyword_difficulty: Option<u32>,
    pub cpc: Option<f64>,
    pub traffic: Option<u64>,
    pub traffic_percentage: Option<f64>,
    pub position: Option<u32>,
    pub url: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ReferringDomainsResponse {
    pub domains: Vec<ReferringDomain>,
    pub total: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct ReferringDomain {
    pub domain: String,
    pub domain_rating: Option<f64>,
    pub backlinks: Option<u64>,
    pub dofollow: Option<u64>,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
}

#[derive(Debug, Clone)]
pub struct KeywordData {
    pub keyword: String,
    pub country: Option<String>,
    pub volume: Option<u64>,
    pub keyword_difficulty: Option<u32>,
    pub cpc: Option<f64>,
    pub clicks: Option<u64>,
    pub global_volume: Option<u64>,
    pub parent_topic: Option<String>,
}

#[derive(Deserialize)]
struct AhrefsBacklinksResponse {
    backlinks: Vec<AhrefsBacklink>,
    total: Option<u64>,
}

#[derive(Deserialize)]
struct AhrefsBacklink {
    url_from: String,
    url_to: String,
    anchor: Option<String>,
    domain_rating: Option<f64>,
    url_rating: Option<f64>,
    traffic: Option<u64>,
    first_seen: Option<String>,
    last_seen: Option<String>,
    nofollow: Option<bool>,
    redirect: Option<bool>,
}

#[derive(Deserialize)]
struct AhrefsDomainRating {
    domain: String,
    domain_rating: f64,
    ahrefs_rank: Option<u64>,
}

#[derive(Deserialize)]
struct AhrefsKeywordsResponse {
    keywords: Vec<AhrefsKeyword>,
    total: Option<u64>,
}

#[derive(Deserialize)]
struct AhrefsKeyword {
    keyword: String,
    country: Option<String>,
    volume: Option<u64>,
    keyword_difficulty: Option<u32>,
    cpc: Option<f64>,
    traffic: Option<u64>,
    traffic_percentage: Option<f64>,
    position: Option<u32>,
    url: Option<String>,
}

#[derive(Deserialize)]
struct AhrefsRefDomainsResponse {
    refdomains: Vec<AhrefsRefDomain>,
    total: Option<u64>,
}

#[derive(Deserialize)]
struct AhrefsRefDomain {
    domain: String,
    domain_rating: Option<f64>,
    backlinks: Option<u64>,
    dofollow: Option<u64>,
    first_seen: Option<String>,
    last_seen: Option<String>,
}

#[derive(Deserialize)]
struct AhrefsKeywordData {
    keyword: String,
    country: Option<String>,
    volume: Option<u64>,
    keyword_difficulty: Option<u32>,
    cpc: Option<f64>,
    clicks: Option<u64>,
    global_volume: Option<u64>,
    parent_topic: Option<String>,
}
