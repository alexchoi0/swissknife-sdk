use swissknife_search_sdk::{
    CrawlResult, ExtractedContent, SearchDepth, SearchOptions, SearchResponse, SearchResult,
    TimeRange,
};

mod search_result_tests {
    use super::*;

    #[test]
    fn test_search_result_creation() {
        let result = SearchResult {
            title: "Test Title".to_string(),
            url: "https://example.com".to_string(),
            snippet: Some("Test snippet content".to_string()),
            content: Some("Full content here".to_string()),
            score: Some(0.95),
            published_date: Some("2024-01-15".to_string()),
        };

        assert_eq!(result.title, "Test Title");
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.snippet, Some("Test snippet content".to_string()));
        assert_eq!(result.content, Some("Full content here".to_string()));
        assert_eq!(result.score, Some(0.95));
        assert_eq!(result.published_date, Some("2024-01-15".to_string()));
    }

    #[test]
    fn test_search_result_minimal() {
        let result = SearchResult {
            title: "Minimal Result".to_string(),
            url: "https://minimal.example.com".to_string(),
            snippet: None,
            content: None,
            score: None,
            published_date: None,
        };

        assert_eq!(result.title, "Minimal Result");
        assert!(result.snippet.is_none());
        assert!(result.content.is_none());
        assert!(result.score.is_none());
        assert!(result.published_date.is_none());
    }

    #[test]
    fn test_search_result_clone() {
        let original = SearchResult {
            title: "Clone Test".to_string(),
            url: "https://clone.example.com".to_string(),
            snippet: Some("Cloneable snippet".to_string()),
            content: None,
            score: Some(0.75),
            published_date: None,
        };

        let cloned = original.clone();
        assert_eq!(original.title, cloned.title);
        assert_eq!(original.url, cloned.url);
        assert_eq!(original.snippet, cloned.snippet);
        assert_eq!(original.score, cloned.score);
    }

    #[test]
    fn test_search_result_debug() {
        let result = SearchResult {
            title: "Debug Test".to_string(),
            url: "https://debug.example.com".to_string(),
            snippet: None,
            content: None,
            score: None,
            published_date: None,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("Debug Test"));
        assert!(debug_str.contains("https://debug.example.com"));
    }
}

mod search_response_tests {
    use super::*;

    #[test]
    fn test_search_response_creation() {
        let response = SearchResponse {
            query: "test query".to_string(),
            results: vec![
                SearchResult {
                    title: "Result 1".to_string(),
                    url: "https://result1.com".to_string(),
                    snippet: Some("First result".to_string()),
                    content: None,
                    score: Some(0.9),
                    published_date: None,
                },
                SearchResult {
                    title: "Result 2".to_string(),
                    url: "https://result2.com".to_string(),
                    snippet: Some("Second result".to_string()),
                    content: None,
                    score: Some(0.8),
                    published_date: None,
                },
            ],
            answer: Some("This is an AI-generated answer".to_string()),
            total_results: Some(100),
        };

        assert_eq!(response.query, "test query");
        assert_eq!(response.results.len(), 2);
        assert!(response.answer.is_some());
        assert_eq!(response.total_results, Some(100));
    }

    #[test]
    fn test_search_response_empty_results() {
        let response = SearchResponse {
            query: "no results query".to_string(),
            results: vec![],
            answer: None,
            total_results: Some(0),
        };

        assert!(response.results.is_empty());
        assert!(response.answer.is_none());
        assert_eq!(response.total_results, Some(0));
    }

    #[test]
    fn test_search_response_clone() {
        let response = SearchResponse {
            query: "clone test".to_string(),
            results: vec![SearchResult {
                title: "Cloned".to_string(),
                url: "https://cloned.com".to_string(),
                snippet: None,
                content: None,
                score: None,
                published_date: None,
            }],
            answer: None,
            total_results: None,
        };

        let cloned = response.clone();
        assert_eq!(response.query, cloned.query);
        assert_eq!(response.results.len(), cloned.results.len());
    }
}

mod search_options_tests {
    use super::*;

    #[test]
    fn test_search_options_default() {
        let options = SearchOptions::default();

        assert!(options.max_results.is_none());
        assert!(options.search_depth.is_none());
        assert!(!options.include_answer);
        assert!(options.include_domains.is_empty());
        assert!(options.exclude_domains.is_empty());
        assert!(options.time_range.is_none());
    }

    #[test]
    fn test_search_options_with_max_results() {
        let mut options = SearchOptions::default();
        options.max_results = Some(25);

        assert_eq!(options.max_results, Some(25));
    }

    #[test]
    fn test_search_options_with_search_depth() {
        let mut options = SearchOptions::default();
        options.search_depth = Some(SearchDepth::Advanced);

        assert!(matches!(options.search_depth, Some(SearchDepth::Advanced)));
    }

    #[test]
    fn test_search_options_with_include_answer() {
        let mut options = SearchOptions::default();
        options.include_answer = true;

        assert!(options.include_answer);
    }

    #[test]
    fn test_search_options_with_domains() {
        let mut options = SearchOptions::default();
        options.include_domains = vec!["example.com".to_string(), "test.org".to_string()];
        options.exclude_domains = vec!["spam.com".to_string()];

        assert_eq!(options.include_domains.len(), 2);
        assert_eq!(options.exclude_domains.len(), 1);
        assert!(options.include_domains.contains(&"example.com".to_string()));
        assert!(options.exclude_domains.contains(&"spam.com".to_string()));
    }

    #[test]
    fn test_search_options_with_time_range() {
        let mut options = SearchOptions::default();
        options.time_range = Some(TimeRange::Week);

        assert!(matches!(options.time_range, Some(TimeRange::Week)));
    }

    #[test]
    fn test_search_options_full_configuration() {
        let options = SearchOptions {
            max_results: Some(50),
            search_depth: Some(SearchDepth::Advanced),
            include_answer: true,
            include_domains: vec!["trusted-source.com".to_string()],
            exclude_domains: vec!["untrusted.com".to_string()],
            time_range: Some(TimeRange::Month),
        };

        assert_eq!(options.max_results, Some(50));
        assert!(matches!(options.search_depth, Some(SearchDepth::Advanced)));
        assert!(options.include_answer);
        assert_eq!(options.include_domains.len(), 1);
        assert_eq!(options.exclude_domains.len(), 1);
        assert!(matches!(options.time_range, Some(TimeRange::Month)));
    }
}

mod search_depth_tests {
    use super::*;

    #[test]
    fn test_search_depth_basic() {
        let depth = SearchDepth::Basic;
        assert!(matches!(depth, SearchDepth::Basic));
    }

    #[test]
    fn test_search_depth_advanced() {
        let depth = SearchDepth::Advanced;
        assert!(matches!(depth, SearchDepth::Advanced));
    }

    #[test]
    fn test_search_depth_equality() {
        assert_eq!(SearchDepth::Basic, SearchDepth::Basic);
        assert_eq!(SearchDepth::Advanced, SearchDepth::Advanced);
        assert_ne!(SearchDepth::Basic, SearchDepth::Advanced);
    }

    #[test]
    fn test_search_depth_copy() {
        let depth = SearchDepth::Basic;
        let copied = depth;
        assert_eq!(depth, copied);
    }

    #[test]
    fn test_search_depth_debug() {
        let depth = SearchDepth::Advanced;
        let debug_str = format!("{:?}", depth);
        assert!(debug_str.contains("Advanced"));
    }
}

mod time_range_tests {
    use super::*;

    #[test]
    fn test_time_range_day() {
        let range = TimeRange::Day;
        assert!(matches!(range, TimeRange::Day));
    }

    #[test]
    fn test_time_range_week() {
        let range = TimeRange::Week;
        assert!(matches!(range, TimeRange::Week));
    }

    #[test]
    fn test_time_range_month() {
        let range = TimeRange::Month;
        assert!(matches!(range, TimeRange::Month));
    }

    #[test]
    fn test_time_range_year() {
        let range = TimeRange::Year;
        assert!(matches!(range, TimeRange::Year));
    }

    #[test]
    fn test_time_range_equality() {
        assert_eq!(TimeRange::Day, TimeRange::Day);
        assert_eq!(TimeRange::Week, TimeRange::Week);
        assert_ne!(TimeRange::Day, TimeRange::Month);
    }

    #[test]
    fn test_time_range_copy() {
        let range = TimeRange::Year;
        let copied = range;
        assert_eq!(range, copied);
    }

    #[test]
    fn test_time_range_debug() {
        let range = TimeRange::Month;
        let debug_str = format!("{:?}", range);
        assert!(debug_str.contains("Month"));
    }
}

mod extracted_content_tests {
    use super::*;

    #[test]
    fn test_extracted_content_creation() {
        let content = ExtractedContent {
            url: "https://example.com/article".to_string(),
            title: Some("Test Article".to_string()),
            content: "This is the main content of the article.".to_string(),
            markdown: Some("# Test Article\n\nThis is the content.".to_string()),
            links: vec![
                "https://example.com/link1".to_string(),
                "https://example.com/link2".to_string(),
            ],
            images: vec!["https://example.com/image.jpg".to_string()],
        };

        assert_eq!(content.url, "https://example.com/article");
        assert_eq!(content.title, Some("Test Article".to_string()));
        assert!(!content.content.is_empty());
        assert!(content.markdown.is_some());
        assert_eq!(content.links.len(), 2);
        assert_eq!(content.images.len(), 1);
    }

    #[test]
    fn test_extracted_content_minimal() {
        let content = ExtractedContent {
            url: "https://minimal.com".to_string(),
            title: None,
            content: "Minimal content".to_string(),
            markdown: None,
            links: vec![],
            images: vec![],
        };

        assert!(content.title.is_none());
        assert!(content.markdown.is_none());
        assert!(content.links.is_empty());
        assert!(content.images.is_empty());
    }

    #[test]
    fn test_extracted_content_clone() {
        let content = ExtractedContent {
            url: "https://clone.com".to_string(),
            title: Some("Clone Test".to_string()),
            content: "Content to clone".to_string(),
            markdown: None,
            links: vec!["https://link.com".to_string()],
            images: vec![],
        };

        let cloned = content.clone();
        assert_eq!(content.url, cloned.url);
        assert_eq!(content.title, cloned.title);
        assert_eq!(content.content, cloned.content);
        assert_eq!(content.links, cloned.links);
    }

    #[test]
    fn test_extracted_content_debug() {
        let content = ExtractedContent {
            url: "https://debug.com".to_string(),
            title: Some("Debug".to_string()),
            content: "Debug content".to_string(),
            markdown: None,
            links: vec![],
            images: vec![],
        };

        let debug_str = format!("{:?}", content);
        assert!(debug_str.contains("https://debug.com"));
    }
}

mod crawl_result_tests {
    use super::*;

    #[test]
    fn test_crawl_result_creation() {
        let result = CrawlResult {
            base_url: "https://example.com".to_string(),
            pages: vec![
                ExtractedContent {
                    url: "https://example.com/page1".to_string(),
                    title: Some("Page 1".to_string()),
                    content: "Page 1 content".to_string(),
                    markdown: None,
                    links: vec![],
                    images: vec![],
                },
                ExtractedContent {
                    url: "https://example.com/page2".to_string(),
                    title: Some("Page 2".to_string()),
                    content: "Page 2 content".to_string(),
                    markdown: None,
                    links: vec![],
                    images: vec![],
                },
            ],
            total_pages: 2,
        };

        assert_eq!(result.base_url, "https://example.com");
        assert_eq!(result.pages.len(), 2);
        assert_eq!(result.total_pages, 2);
    }

    #[test]
    fn test_crawl_result_empty() {
        let result = CrawlResult {
            base_url: "https://empty.com".to_string(),
            pages: vec![],
            total_pages: 0,
        };

        assert!(result.pages.is_empty());
        assert_eq!(result.total_pages, 0);
    }

    #[test]
    fn test_crawl_result_clone() {
        let result = CrawlResult {
            base_url: "https://clone.com".to_string(),
            pages: vec![ExtractedContent {
                url: "https://clone.com/page".to_string(),
                title: None,
                content: "Content".to_string(),
                markdown: None,
                links: vec![],
                images: vec![],
            }],
            total_pages: 1,
        };

        let cloned = result.clone();
        assert_eq!(result.base_url, cloned.base_url);
        assert_eq!(result.pages.len(), cloned.pages.len());
        assert_eq!(result.total_pages, cloned.total_pages);
    }

    #[test]
    fn test_crawl_result_debug() {
        let result = CrawlResult {
            base_url: "https://debug.com".to_string(),
            pages: vec![],
            total_pages: 0,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("https://debug.com"));
    }
}
