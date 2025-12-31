use swissknife_database_sdk::{SearchHit, SearchResponse, SearchQuery};
use std::collections::HashMap;

mod search_hit_tests {
    use super::*;

    #[test]
    fn test_search_hit_creation() {
        let hit = SearchHit {
            id: "doc123".to_string(),
            score: 0.95,
            source: serde_json::json!({
                "title": "Test Document",
                "content": "Document content"
            }),
            highlights: Some({
                let mut map = HashMap::new();
                map.insert(
                    "title".to_string(),
                    vec!["<em>Test</em> Document".to_string()],
                );
                map
            }),
        };

        assert_eq!(hit.id, "doc123");
        assert_eq!(hit.score, 0.95);
        assert_eq!(hit.source["title"], "Test Document");
        assert!(hit.highlights.is_some());
    }

    #[test]
    fn test_search_hit_without_highlights() {
        let hit = SearchHit {
            id: "no_highlight".to_string(),
            score: 0.5,
            source: serde_json::json!({}),
            highlights: None,
        };

        assert!(hit.highlights.is_none());
    }

    #[test]
    fn test_search_hit_clone() {
        let hit = SearchHit {
            id: "clone_test".to_string(),
            score: 0.8,
            source: serde_json::json!({"test": true}),
            highlights: None,
        };

        let cloned = hit.clone();
        assert_eq!(hit.id, cloned.id);
        assert_eq!(hit.score, cloned.score);
    }

    #[test]
    fn test_search_hit_debug() {
        let hit = SearchHit {
            id: "debug".to_string(),
            score: 0.0,
            source: serde_json::json!({}),
            highlights: None,
        };

        let debug_str = format!("{:?}", hit);
        assert!(debug_str.contains("debug") || debug_str.contains("SearchHit"));
    }

    #[test]
    fn test_search_hit_serialize() {
        let hit = SearchHit {
            id: "hit1".to_string(),
            score: 0.85,
            source: serde_json::json!({}),
            highlights: None,
        };

        let json = serde_json::to_string(&hit).unwrap();
        assert!(json.contains("hit1"));
        assert!(json.contains("0.85"));
    }
}

mod search_response_tests {
    use super::*;

    #[test]
    fn test_search_response_creation() {
        let response = SearchResponse {
            hits: vec![
                SearchHit {
                    id: "1".to_string(),
                    score: 0.9,
                    source: serde_json::json!({"name": "First"}),
                    highlights: None,
                },
                SearchHit {
                    id: "2".to_string(),
                    score: 0.8,
                    source: serde_json::json!({"name": "Second"}),
                    highlights: None,
                },
            ],
            total: 100,
            took_ms: 25,
        };

        assert_eq!(response.hits.len(), 2);
        assert_eq!(response.total, 100);
        assert_eq!(response.took_ms, 25);
    }

    #[test]
    fn test_search_response_empty() {
        let response = SearchResponse {
            hits: vec![],
            total: 0,
            took_ms: 5,
        };

        assert!(response.hits.is_empty());
        assert_eq!(response.total, 0);
    }

    #[test]
    fn test_search_response_clone() {
        let response = SearchResponse {
            hits: vec![],
            total: 50,
            took_ms: 10,
        };

        let cloned = response.clone();
        assert_eq!(response.total, cloned.total);
        assert_eq!(response.took_ms, cloned.took_ms);
    }

    #[test]
    fn test_search_response_debug() {
        let response = SearchResponse {
            hits: vec![],
            total: 0,
            took_ms: 0,
        };

        let debug_str = format!("{:?}", response);
        assert!(debug_str.contains("SearchResponse") || debug_str.contains("hits"));
    }

    #[test]
    fn test_search_response_serialize() {
        let response = SearchResponse {
            hits: vec![],
            total: 42,
            took_ms: 15,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("42"));
        assert!(json.contains("15"));
    }
}

mod search_query_tests {
    use super::*;

    #[test]
    fn test_search_query_default() {
        let query = SearchQuery::default();

        assert!(query.from.is_none());
        assert!(query.size.is_none());
        assert!(query.sort.is_none());
        assert!(query.highlight.is_none());
        assert!(query.aggregations.is_none());
    }

    #[test]
    fn test_search_query_with_pagination() {
        let query = SearchQuery {
            query: serde_json::json!({"match_all": {}}),
            from: Some(10),
            size: Some(20),
            sort: None,
            highlight: None,
            aggregations: None,
        };

        assert_eq!(query.from, Some(10));
        assert_eq!(query.size, Some(20));
    }

    #[test]
    fn test_search_query_with_sort() {
        let query = SearchQuery {
            query: serde_json::json!({"match": {"title": "test"}}),
            from: None,
            size: None,
            sort: Some(vec![serde_json::json!({"_score": "desc"})]),
            highlight: None,
            aggregations: None,
        };

        assert!(query.sort.is_some());
        assert_eq!(query.sort.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_search_query_with_highlight() {
        let query = SearchQuery {
            query: serde_json::json!({"match": {"content": "search term"}}),
            from: None,
            size: None,
            sort: None,
            highlight: Some(serde_json::json!({
                "fields": {
                    "content": {}
                }
            })),
            aggregations: None,
        };

        assert!(query.highlight.is_some());
    }

    #[test]
    fn test_search_query_with_aggregations() {
        let query = SearchQuery {
            query: serde_json::json!({"match_all": {}}),
            from: None,
            size: Some(0),
            sort: None,
            highlight: None,
            aggregations: Some(serde_json::json!({
                "categories": {
                    "terms": {"field": "category"}
                }
            })),
        };

        assert!(query.aggregations.is_some());
    }

    #[test]
    fn test_search_query_full_configuration() {
        let query = SearchQuery {
            query: serde_json::json!({"bool": {"must": [{"match": {"title": "test"}}]}}),
            from: Some(0),
            size: Some(10),
            sort: Some(vec![serde_json::json!({"date": "desc"})]),
            highlight: Some(serde_json::json!({"fields": {"title": {}}})),
            aggregations: Some(serde_json::json!({"tags": {"terms": {"field": "tags"}}})),
        };

        assert!(query.from.is_some());
        assert!(query.size.is_some());
        assert!(query.sort.is_some());
        assert!(query.highlight.is_some());
        assert!(query.aggregations.is_some());
    }

    #[test]
    fn test_search_query_clone() {
        let query = SearchQuery {
            query: serde_json::json!({"match_all": {}}),
            from: Some(5),
            size: Some(10),
            sort: None,
            highlight: None,
            aggregations: None,
        };

        let cloned = query.clone();
        assert_eq!(query.from, cloned.from);
        assert_eq!(query.size, cloned.size);
    }

    #[test]
    fn test_search_query_debug() {
        let query = SearchQuery::default();
        let debug_str = format!("{:?}", query);
        assert!(debug_str.contains("SearchQuery"));
    }
}
