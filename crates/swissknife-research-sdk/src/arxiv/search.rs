use crate::{Error, Paper, Result, SearchParams, SearchResult, SortBy, SortOrder};
use crate::arxiv::ArxivClient;
use crate::arxiv::parser;

impl ArxivClient {
    pub async fn search(&self, params: &SearchParams) -> Result<SearchResult> {
        let mut query_parts: Vec<String> = Vec::new();

        query_parts.push(format!("search_query={}", urlencoding::encode(&params.query)));

        if let Some(start) = params.start {
            query_parts.push(format!("start={}", start));
        }

        let max_results = params.max_results.unwrap_or(10);
        query_parts.push(format!("max_results={}", max_results));

        if let Some(sort_by) = &params.sort_by {
            let sort_str = match sort_by {
                SortBy::Relevance => "relevance",
                SortBy::LastUpdated => "lastUpdatedDate",
                SortBy::Submitted => "submittedDate",
            };
            query_parts.push(format!("sortBy={}", sort_str));
        }

        if let Some(sort_order) = &params.sort_order {
            let order_str = match sort_order {
                SortOrder::Ascending => "ascending",
                SortOrder::Descending => "descending",
            };
            query_parts.push(format!("sortOrder={}", order_str));
        }

        let url = format!("{}/query?{}", self.base_url(), query_parts.join("&"));

        let response = self.client()
            .get(&url)
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

        let xml = response.text().await?;
        let papers = parser::parse_atom_feed(&xml)?;
        let total_results = parser::parse_total_results(&xml);
        let start_index = parser::parse_start_index(&xml);
        let items_per_page = parser::parse_items_per_page(&xml);

        Ok(SearchResult {
            papers,
            total_results,
            start_index,
            items_per_page,
        })
    }

    pub async fn search_all(&self, query: &str, max_results: Option<u32>) -> Result<SearchResult> {
        let params = SearchParams {
            query: format!("all:{}", query),
            max_results,
            ..Default::default()
        };
        self.search(&params).await
    }

    pub async fn search_title(&self, title: &str, max_results: Option<u32>) -> Result<SearchResult> {
        let params = SearchParams {
            query: format!("ti:{}", title),
            max_results,
            ..Default::default()
        };
        self.search(&params).await
    }

    pub async fn search_author(&self, author: &str, max_results: Option<u32>) -> Result<SearchResult> {
        let params = SearchParams {
            query: format!("au:{}", author),
            max_results,
            ..Default::default()
        };
        self.search(&params).await
    }

    pub async fn search_abstract(&self, abstract_query: &str, max_results: Option<u32>) -> Result<SearchResult> {
        let params = SearchParams {
            query: format!("abs:{}", abstract_query),
            max_results,
            ..Default::default()
        };
        self.search(&params).await
    }

    pub async fn search_category(&self, category: &str, max_results: Option<u32>) -> Result<SearchResult> {
        let params = SearchParams {
            query: format!("cat:{}", category),
            max_results,
            ..Default::default()
        };
        self.search(&params).await
    }

    pub async fn get_paper(&self, arxiv_id: &str) -> Result<Paper> {
        let url = format!("{}/query?id_list={}", self.base_url(), arxiv_id);

        let response = self.client()
            .get(&url)
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

        let xml = response.text().await?;
        let papers = parser::parse_atom_feed(&xml)?;

        papers.into_iter().next().ok_or_else(|| Error::NotFound(arxiv_id.to_string()))
    }

    pub async fn get_papers(&self, arxiv_ids: &[&str]) -> Result<Vec<Paper>> {
        let id_list = arxiv_ids.join(",");
        let url = format!("{}/query?id_list={}", self.base_url(), id_list);

        let response = self.client()
            .get(&url)
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

        let xml = response.text().await?;
        parser::parse_atom_feed(&xml)
    }

    pub fn get_pdf_url(&self, arxiv_id: &str) -> String {
        format!("https://arxiv.org/pdf/{}.pdf", arxiv_id)
    }

    pub fn get_abs_url(&self, arxiv_id: &str) -> String {
        format!("https://arxiv.org/abs/{}", arxiv_id)
    }

    pub async fn get_recent(&self, category: &str, max_results: Option<u32>) -> Result<SearchResult> {
        let params = SearchParams {
            query: format!("cat:{}", category),
            max_results,
            sort_by: Some(SortBy::Submitted),
            sort_order: Some(SortOrder::Descending),
            ..Default::default()
        };
        self.search(&params).await
    }
}

fn urlencoding_encode(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            c if c.is_alphanumeric() || "-_.~".contains(c) => c.to_string(),
            c => format!("%{:02X}", c as u8),
        })
        .collect()
}
