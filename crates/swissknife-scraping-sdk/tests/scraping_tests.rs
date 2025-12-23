#[cfg(feature = "apify")]
mod apify_tests {
    use swissknife_scraping_sdk::apify::{
        ApifyClient, RunActorParams, GetDatasetItemsParams,
    };
    use std::collections::HashMap;

    #[test]
    fn test_apify_client_creation() {
        let client = ApifyClient::new("apify-api-token");
        assert!(true);
    }

    #[test]
    fn test_run_actor_params() {
        let mut input = HashMap::new();
        input.insert("startUrls".to_string(), serde_json::json!([{"url": "https://example.com"}]));

        let params = RunActorParams {
            input: Some(input),
            build: Some("latest".to_string()),
            timeout_secs: Some(300),
            memory_mbytes: Some(1024),
            webhooks: None,
        };

        assert!(params.input.is_some());
        assert_eq!(params.build, Some("latest".to_string()));
        assert_eq!(params.timeout_secs, Some(300));
        assert_eq!(params.memory_mbytes, Some(1024));
    }

    #[test]
    fn test_get_dataset_items_params() {
        let params = GetDatasetItemsParams {
            offset: Some(0),
            limit: Some(100),
            clean: Some(true),
            format: Some("json".to_string()),
            fields: Some(vec!["title".to_string(), "url".to_string()]),
            omit: None,
            flatten: Some(false),
            desc: Some(true),
        };

        assert_eq!(params.offset, Some(0));
        assert_eq!(params.limit, Some(100));
        assert_eq!(params.clean, Some(true));
        assert!(params.fields.is_some());
    }

    #[test]
    fn test_params_defaults() {
        let run_params = RunActorParams::default();
        let dataset_params = GetDatasetItemsParams::default();

        assert!(run_params.input.is_none());
        assert!(run_params.build.is_none());
        assert!(dataset_params.offset.is_none());
        assert!(dataset_params.limit.is_none());
    }

    #[test]
    fn test_actor_ids() {
        let actors = vec![
            "apify/web-scraper",
            "apify/cheerio-scraper",
            "apify/puppeteer-scraper",
            "apify/playwright-scraper",
        ];

        for actor in actors {
            assert!(actor.contains("/"));
        }
    }
}

#[cfg(feature = "firecrawl")]
mod firecrawl_tests {
    use swissknife_scraping_sdk::firecrawl::{
        FirecrawlClient, ScrapeParams, CrawlParams, ExtractParams, OutputFormat,
    };

    #[test]
    fn test_firecrawl_client_creation() {
        let client = FirecrawlClient::new("fc-api-key");
        assert!(true);
    }

    #[test]
    fn test_scrape_params() {
        let params = ScrapeParams {
            formats: Some(vec![OutputFormat::Markdown, OutputFormat::Html]),
            only_main_content: Some(true),
            include_tags: Some(vec!["article".to_string(), "main".to_string()]),
            exclude_tags: Some(vec!["nav".to_string(), "footer".to_string()]),
            wait_for: Some(2000),
            timeout: Some(30000),
        };

        assert!(params.formats.is_some());
        assert_eq!(params.only_main_content, Some(true));
        assert_eq!(params.wait_for, Some(2000));
    }

    #[test]
    fn test_crawl_params() {
        let params = CrawlParams {
            max_depth: Some(3),
            limit: Some(100),
            include_paths: Some(vec!["/blog/*".to_string()]),
            exclude_paths: Some(vec!["/admin/*".to_string()]),
            allow_backward_links: Some(false),
            allow_external_links: Some(false),
            ignore_sitemap: Some(false),
            scrape_options: None,
        };

        assert_eq!(params.max_depth, Some(3));
        assert_eq!(params.limit, Some(100));
        assert!(params.include_paths.is_some());
    }

    #[test]
    fn test_extract_params() {
        let params = ExtractParams {
            schema: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "title": {"type": "string"},
                    "price": {"type": "number"}
                }
            })),
            prompt: Some("Extract product information".to_string()),
            system_prompt: None,
        };

        assert!(params.schema.is_some());
        assert_eq!(params.prompt, Some("Extract product information".to_string()));
    }

    #[test]
    fn test_output_format_variants() {
        let markdown = OutputFormat::Markdown;
        let html = OutputFormat::Html;
        let raw_html = OutputFormat::RawHtml;
        let screenshot = OutputFormat::Screenshot;
        let links = OutputFormat::Links;

        assert!(matches!(markdown, OutputFormat::Markdown));
        assert!(matches!(html, OutputFormat::Html));
        assert!(matches!(raw_html, OutputFormat::RawHtml));
        assert!(matches!(screenshot, OutputFormat::Screenshot));
        assert!(matches!(links, OutputFormat::Links));
    }

    #[test]
    fn test_params_defaults() {
        let scrape_params = ScrapeParams::default();
        let crawl_params = CrawlParams::default();

        assert!(scrape_params.formats.is_none());
        assert!(crawl_params.max_depth.is_none());
    }
}

#[cfg(feature = "browseruse")]
mod browseruse_tests {
    use swissknife_scraping_sdk::browseruse::{
        BrowserUseClient, CreateBrowserParams, TaskParams,
    };

    #[test]
    fn test_browseruse_client_creation() {
        let client = BrowserUseClient::new("bu-api-key");
        assert!(true);
    }

    #[test]
    fn test_create_browser_params() {
        let params = CreateBrowserParams {
            headless: Some(true),
            proxy: Some("http://proxy.example.com:8080".to_string()),
            user_agent: Some("Custom User Agent".to_string()),
            viewport_width: Some(1920),
            viewport_height: Some(1080),
            locale: Some("en-US".to_string()),
            timezone: Some("America/New_York".to_string()),
        };

        assert_eq!(params.headless, Some(true));
        assert!(params.proxy.is_some());
        assert_eq!(params.viewport_width, Some(1920));
        assert_eq!(params.viewport_height, Some(1080));
    }

    #[test]
    fn test_task_params() {
        let params = TaskParams {
            task: "Go to example.com and click the login button".to_string(),
            url: Some("https://example.com".to_string()),
            max_steps: Some(10),
            timeout: Some(60000),
            extract_data: Some(true),
        };

        assert!(!params.task.is_empty());
        assert!(params.url.is_some());
        assert_eq!(params.max_steps, Some(10));
    }

    #[test]
    fn test_params_defaults() {
        let browser_params = CreateBrowserParams::default();

        assert!(browser_params.headless.is_none());
        assert!(browser_params.proxy.is_none());
        assert!(browser_params.viewport_width.is_none());
    }
}

mod error_tests {
    use swissknife_scraping_sdk::Error;

    #[test]
    fn test_error_display() {
        let api_error = Error::Api {
            message: "Actor run failed".to_string(),
            code: Some("ACTOR_FAILED".to_string()),
        };

        let error_string = format!("{}", api_error);
        assert!(error_string.contains("Actor run failed"));
    }

    #[test]
    fn test_actor_not_found_error() {
        let error = Error::ActorNotFound("unknown-actor".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("unknown-actor"));
    }

    #[test]
    fn test_run_failed_error() {
        let error = Error::RunFailed("Timeout exceeded".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Timeout exceeded"));
    }

    #[test]
    fn test_scrape_failed_error() {
        let error = Error::ScrapeFailed("Page not accessible".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Page not accessible"));
    }
}
