use rmcp::tool_box;
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "scraping")]
use swissknife_scraping_sdk as scrape;

#[derive(Clone)]
pub struct ScrapingTools {
    #[cfg(feature = "apify")]
    pub apify: Option<scrape::apify::ApifyClient>,
    #[cfg(feature = "firecrawl")]
    pub firecrawl: Option<scrape::firecrawl::FirecrawlClient>,
    #[cfg(feature = "browseruse")]
    pub browseruse: Option<scrape::browseruse::BrowserUseClient>,
}

impl ScrapingTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "apify")]
            apify: None,
            #[cfg(feature = "firecrawl")]
            firecrawl: None,
            #[cfg(feature = "browseruse")]
            browseruse: None,
        }
    }

    #[cfg(feature = "apify")]
    pub fn with_apify(mut self, client: scrape::apify::ApifyClient) -> Self {
        self.apify = Some(client);
        self
    }

    #[cfg(feature = "firecrawl")]
    pub fn with_firecrawl(mut self, client: scrape::firecrawl::FirecrawlClient) -> Self {
        self.firecrawl = Some(client);
        self
    }

    #[cfg(feature = "browseruse")]
    pub fn with_browseruse(mut self, client: scrape::browseruse::BrowserUseClient) -> Self {
        self.browseruse = Some(client);
        self
    }
}

impl Default for ScrapingTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ApifyRunActorRequest {
    pub actor_id: String,
    #[serde(default)]
    pub input: Option<serde_json::Value>,
    #[serde(default)]
    pub wait_for_finish: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ApifyGetDatasetRequest {
    pub dataset_id: String,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub offset: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ApifyWebScraperRequest {
    pub start_urls: Vec<String>,
    #[serde(default)]
    pub page_function: Option<String>,
    #[serde(default)]
    pub max_pages: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FirecrawlScrapeRequest {
    pub url: String,
    #[serde(default)]
    pub formats: Option<Vec<String>>,
    #[serde(default)]
    pub only_main_content: Option<bool>,
    #[serde(default)]
    pub include_tags: Option<Vec<String>>,
    #[serde(default)]
    pub exclude_tags: Option<Vec<String>>,
    #[serde(default)]
    pub wait_for: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FirecrawlCrawlRequest {
    pub url: String,
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub max_depth: Option<u32>,
    #[serde(default)]
    pub include_paths: Option<Vec<String>>,
    #[serde(default)]
    pub exclude_paths: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FirecrawlMapRequest {
    pub url: String,
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FirecrawlExtractRequest {
    pub urls: Vec<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub schema: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BrowserUseTaskRequest {
    pub task: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub max_steps: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BrowserUseNavigateRequest {
    pub url: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BrowserUseClickRequest {
    pub selector: String,
    #[serde(default)]
    pub index: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BrowserUseTypeRequest {
    pub selector: String,
    pub text: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BrowserUseScreenshotRequest {
    #[serde(default)]
    pub full_page: Option<bool>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct BrowserUseExtractRequest {
    #[serde(default)]
    pub selector: Option<String>,
    #[serde(default)]
    pub attribute: Option<String>,
}

#[tool_box]
impl ScrapingTools {
    #[cfg(feature = "apify")]
    #[rmcp::tool(description = "Run an Apify actor")]
    pub async fn apify_run_actor(
        &self,
        #[rmcp::tool(aggr)] req: ApifyRunActorRequest,
    ) -> Result<String, String> {
        let client = self.apify.as_ref()
            .ok_or_else(|| "Apify client not configured".to_string())?;

        let mut request = scrape::apify::RunActorRequest::new();
        if let Some(input) = req.input {
            request = request.with_input(input);
        }
        if let Some(wait) = req.wait_for_finish {
            request = request.with_wait_for_finish(wait);
        }

        let run = client.actors().run(&req.actor_id, &request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": run.id,
            "status": run.status,
            "started_at": run.started_at,
            "finished_at": run.finished_at,
            "default_dataset_id": run.default_dataset_id
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "apify")]
    #[rmcp::tool(description = "Get items from an Apify dataset")]
    pub async fn apify_get_dataset(
        &self,
        #[rmcp::tool(aggr)] req: ApifyGetDatasetRequest,
    ) -> Result<String, String> {
        let client = self.apify.as_ref()
            .ok_or_else(|| "Apify client not configured".to_string())?;

        let mut params = scrape::apify::ListItemsParams::new();
        if let Some(l) = req.limit {
            params = params.with_limit(l);
        }
        if let Some(o) = req.offset {
            params = params.with_offset(o);
        }

        let items = client.datasets().list_items(&req.dataset_id, &params).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "items": items
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "apify")]
    #[rmcp::tool(description = "Scrape websites using Apify Web Scraper")]
    pub async fn apify_web_scraper(
        &self,
        #[rmcp::tool(aggr)] req: ApifyWebScraperRequest,
    ) -> Result<String, String> {
        let client = self.apify.as_ref()
            .ok_or_else(|| "Apify client not configured".to_string())?;

        let urls: Vec<scrape::apify::StartUrl> = req.start_urls.iter()
            .map(|url| scrape::apify::StartUrl { url: url.clone() })
            .collect();

        let mut input = scrape::apify::WebScraperInput::new(urls);
        if let Some(pf) = req.page_function {
            input = input.with_page_function(pf);
        }
        if let Some(mp) = req.max_pages {
            input = input.with_max_pages_per_crawl(mp);
        }

        let request = scrape::apify::RunActorRequest::new()
            .with_input(serde_json::to_value(&input).unwrap_or_default())
            .with_wait_for_finish(300);

        let run = client.actors().run("apify/web-scraper", &request).await
            .map_err(|e| e.to_string())?;

        if let Some(dataset_id) = &run.default_dataset_id {
            let items = client.datasets()
                .list_items(dataset_id, &scrape::apify::ListItemsParams::new())
                .await
                .map_err(|e| e.to_string())?;

            serde_json::to_string_pretty(&serde_json::json!({
                "run_id": run.id,
                "status": run.status,
                "items": items
            })).map_err(|e| e.to_string())
        } else {
            serde_json::to_string_pretty(&serde_json::json!({
                "run_id": run.id,
                "status": run.status
            })).map_err(|e| e.to_string())
        }
    }

    #[cfg(feature = "firecrawl")]
    #[rmcp::tool(description = "Scrape a single URL with Firecrawl")]
    pub async fn firecrawl_scrape(
        &self,
        #[rmcp::tool(aggr)] req: FirecrawlScrapeRequest,
    ) -> Result<String, String> {
        let client = self.firecrawl.as_ref()
            .ok_or_else(|| "Firecrawl client not configured".to_string())?;

        let mut request = scrape::firecrawl::ScrapeRequest::new(req.url);

        if let Some(formats) = req.formats {
            request = request.with_formats(formats);
        }
        if let Some(only_main) = req.only_main_content {
            request = request.with_only_main_content(only_main);
        }
        if let Some(include) = req.include_tags {
            request = request.with_include_tags(include);
        }
        if let Some(exclude) = req.exclude_tags {
            request = request.with_exclude_tags(exclude);
        }
        if let Some(wait) = req.wait_for {
            request = request.with_wait_for(wait);
        }

        let response = client.scrape(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "markdown": response.data.markdown,
            "html": response.data.html,
            "metadata": response.data.metadata
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "firecrawl")]
    #[rmcp::tool(description = "Crawl a website with Firecrawl")]
    pub async fn firecrawl_crawl(
        &self,
        #[rmcp::tool(aggr)] req: FirecrawlCrawlRequest,
    ) -> Result<String, String> {
        let client = self.firecrawl.as_ref()
            .ok_or_else(|| "Firecrawl client not configured".to_string())?;

        let mut request = scrape::firecrawl::CrawlRequest::new(req.url);

        if let Some(limit) = req.limit {
            request = request.with_limit(limit);
        }
        if let Some(depth) = req.max_depth {
            request = request.with_max_depth(depth);
        }
        if let Some(include) = req.include_paths {
            request = request.with_include_paths(include);
        }
        if let Some(exclude) = req.exclude_paths {
            request = request.with_exclude_paths(exclude);
        }

        let response = client.crawl(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": response.id,
            "url": response.url
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "firecrawl")]
    #[rmcp::tool(description = "Map a website's structure with Firecrawl")]
    pub async fn firecrawl_map(
        &self,
        #[rmcp::tool(aggr)] req: FirecrawlMapRequest,
    ) -> Result<String, String> {
        let client = self.firecrawl.as_ref()
            .ok_or_else(|| "Firecrawl client not configured".to_string())?;

        let mut request = scrape::firecrawl::MapRequest::new(req.url);

        if let Some(search) = req.search {
            request = request.with_search(search);
        }
        if let Some(limit) = req.limit {
            request = request.with_limit(limit);
        }

        let response = client.map(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "links": response.links
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "firecrawl")]
    #[rmcp::tool(description = "Extract structured data with Firecrawl")]
    pub async fn firecrawl_extract(
        &self,
        #[rmcp::tool(aggr)] req: FirecrawlExtractRequest,
    ) -> Result<String, String> {
        let client = self.firecrawl.as_ref()
            .ok_or_else(|| "Firecrawl client not configured".to_string())?;

        let mut request = scrape::firecrawl::ExtractRequest::new(req.urls);

        if let Some(prompt) = req.prompt {
            request = request.with_prompt(prompt);
        }
        if let Some(schema) = req.schema {
            request = request.with_schema(schema);
        }

        let response = client.extract(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "data": response.data,
            "status": response.status
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "browseruse")]
    #[rmcp::tool(description = "Run a browser automation task")]
    pub async fn browseruse_run_task(
        &self,
        #[rmcp::tool(aggr)] req: BrowserUseTaskRequest,
    ) -> Result<String, String> {
        let client = self.browseruse.as_ref()
            .ok_or_else(|| "BrowserUse client not configured".to_string())?;

        let mut request = scrape::browseruse::TaskRequest::new(req.task);

        if let Some(url) = req.url {
            request = request.with_url(url);
        }
        if let Some(max_steps) = req.max_steps {
            request = request.with_max_steps(max_steps);
        }

        let response = client.run_task(&request).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "task_id": response.task_id,
            "status": response.status,
            "result": response.result,
            "steps": response.steps
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "browseruse")]
    #[rmcp::tool(description = "Navigate to a URL")]
    pub async fn browseruse_navigate(
        &self,
        #[rmcp::tool(aggr)] req: BrowserUseNavigateRequest,
    ) -> Result<String, String> {
        let client = self.browseruse.as_ref()
            .ok_or_else(|| "BrowserUse client not configured".to_string())?;

        let action = scrape::browseruse::BrowserAction::Navigate { url: req.url };
        client.execute_action(&action).await
            .map_err(|e| e.to_string())?;

        Ok("Navigation completed".to_string())
    }

    #[cfg(feature = "browseruse")]
    #[rmcp::tool(description = "Click an element on the page")]
    pub async fn browseruse_click(
        &self,
        #[rmcp::tool(aggr)] req: BrowserUseClickRequest,
    ) -> Result<String, String> {
        let client = self.browseruse.as_ref()
            .ok_or_else(|| "BrowserUse client not configured".to_string())?;

        let action = scrape::browseruse::BrowserAction::Click {
            selector: req.selector,
            index: req.index,
        };
        client.execute_action(&action).await
            .map_err(|e| e.to_string())?;

        Ok("Click action completed".to_string())
    }

    #[cfg(feature = "browseruse")]
    #[rmcp::tool(description = "Type text into an input field")]
    pub async fn browseruse_type(
        &self,
        #[rmcp::tool(aggr)] req: BrowserUseTypeRequest,
    ) -> Result<String, String> {
        let client = self.browseruse.as_ref()
            .ok_or_else(|| "BrowserUse client not configured".to_string())?;

        let action = scrape::browseruse::BrowserAction::Type {
            selector: req.selector,
            text: req.text,
        };
        client.execute_action(&action).await
            .map_err(|e| e.to_string())?;

        Ok("Type action completed".to_string())
    }

    #[cfg(feature = "browseruse")]
    #[rmcp::tool(description = "Take a screenshot of the page")]
    pub async fn browseruse_screenshot(
        &self,
        #[rmcp::tool(aggr)] req: BrowserUseScreenshotRequest,
    ) -> Result<String, String> {
        let client = self.browseruse.as_ref()
            .ok_or_else(|| "BrowserUse client not configured".to_string())?;

        let action = scrape::browseruse::BrowserAction::Screenshot {
            full_page: req.full_page.unwrap_or(false),
        };
        let result = client.execute_action(&action).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "screenshot": result
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "browseruse")]
    #[rmcp::tool(description = "Extract content from the page")]
    pub async fn browseruse_extract(
        &self,
        #[rmcp::tool(aggr)] req: BrowserUseExtractRequest,
    ) -> Result<String, String> {
        let client = self.browseruse.as_ref()
            .ok_or_else(|| "BrowserUse client not configured".to_string())?;

        let action = scrape::browseruse::BrowserAction::ExtractContent {
            selector: req.selector,
            attribute: req.attribute,
        };
        let result = client.execute_action(&action).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "content": result
        })).map_err(|e| e.to_string())
    }
}
