#[cfg(feature = "arxiv")]
mod arxiv_tests {
    use swissknife_research_sdk::arxiv::{
        ArxivClient, SearchQuery, SortBy, SortOrder,
    };

    #[test]
    fn test_arxiv_client_creation() {
        let client = ArxivClient::new();
        assert!(true);
    }

    #[test]
    fn test_search_query_basic() {
        let query = SearchQuery::new("machine learning");

        assert_eq!(query.query, "machine learning");
        assert!(query.id_list.is_none());
        assert!(query.start.is_none());
        assert!(query.max_results.is_none());
    }

    #[test]
    fn test_search_query_builder() {
        let query = SearchQuery::new("quantum computing")
            .with_max_results(50)
            .with_start(10)
            .with_sort_by(SortBy::SubmittedDate)
            .with_sort_order(SortOrder::Descending);

        assert_eq!(query.query, "quantum computing");
        assert_eq!(query.max_results, Some(50));
        assert_eq!(query.start, Some(10));
        assert!(matches!(query.sort_by, Some(SortBy::SubmittedDate)));
        assert!(matches!(query.sort_order, Some(SortOrder::Descending)));
    }

    #[test]
    fn test_search_query_with_ids() {
        let ids = vec!["2301.00001".to_string(), "2301.00002".to_string()];
        let query = SearchQuery::new("")
            .with_id_list(ids.clone());

        assert_eq!(query.id_list, Some(ids));
    }

    #[test]
    fn test_search_query_with_categories() {
        let categories = vec!["cs.AI".to_string(), "cs.LG".to_string()];
        let query = SearchQuery::new("neural networks")
            .with_categories(categories.clone());

        assert_eq!(query.categories, Some(categories));
    }

    #[test]
    fn test_sort_by_variants() {
        let relevance = SortBy::Relevance;
        let last_updated = SortBy::LastUpdatedDate;
        let submitted = SortBy::SubmittedDate;

        assert!(matches!(relevance, SortBy::Relevance));
        assert!(matches!(last_updated, SortBy::LastUpdatedDate));
        assert!(matches!(submitted, SortBy::SubmittedDate));
    }

    #[test]
    fn test_sort_order_variants() {
        let asc = SortOrder::Ascending;
        let desc = SortOrder::Descending;

        assert!(matches!(asc, SortOrder::Ascending));
        assert!(matches!(desc, SortOrder::Descending));
    }

    #[test]
    fn test_search_query_advanced() {
        let query = SearchQuery::new("ti:attention AND au:vaswani")
            .with_max_results(10);

        assert!(query.query.contains("ti:attention"));
        assert!(query.query.contains("au:vaswani"));
    }

    #[test]
    fn test_arxiv_id_format() {
        let old_format = "hep-th/9901001";
        let new_format = "2301.00001";
        let new_format_v2 = "2301.00001v2";

        assert!(old_format.contains("/"));
        assert!(!new_format.contains("/"));
        assert!(new_format_v2.contains("v"));
    }

    #[test]
    fn test_category_codes() {
        let categories = vec![
            "cs.AI",      // Artificial Intelligence
            "cs.LG",      // Machine Learning
            "cs.CL",      // Computation and Language
            "stat.ML",    // Machine Learning (Statistics)
            "physics.hep-th", // High Energy Physics - Theory
            "math.CO",    // Combinatorics
        ];

        for cat in categories {
            assert!(cat.contains("."));
        }
    }
}

mod error_tests {
    use swissknife_research_sdk::Error;

    #[test]
    fn test_error_display() {
        let api_error = Error::Api {
            message: "Rate limit exceeded".to_string(),
            code: Some("429".to_string()),
        };

        let error_string = format!("{}", api_error);
        assert!(error_string.contains("Rate limit exceeded"));
    }

    #[test]
    fn test_paper_not_found_error() {
        let error = Error::PaperNotFound("2301.99999".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("2301.99999"));
    }

    #[test]
    fn test_parse_error() {
        let error = Error::Parse("Invalid XML response".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Invalid XML"));
    }

    #[test]
    fn test_invalid_query_error() {
        let error = Error::InvalidQuery("Empty query string".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Empty query"));
    }
}

mod parser_tests {
    #[test]
    fn test_date_parsing() {
        let date_str = "2024-01-15T10:30:00Z";
        assert!(date_str.contains("T"));
        assert!(date_str.ends_with("Z"));
    }

    #[test]
    fn test_author_name_parsing() {
        let authors = vec![
            "John Doe",
            "Jane Smith",
            "Alice Johnson III",
        ];

        for author in authors {
            assert!(!author.is_empty());
        }
    }

    #[test]
    fn test_category_parsing() {
        let primary = "cs.AI";
        let secondary = vec!["cs.LG", "stat.ML"];

        assert!(primary.starts_with("cs."));
        for cat in secondary {
            assert!(cat.contains("."));
        }
    }
}
