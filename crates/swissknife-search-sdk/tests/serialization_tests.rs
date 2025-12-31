use swissknife_search_sdk::{CrawlResult, ExtractedContent, SearchResponse, SearchResult};

#[test]
fn test_search_result_serialize() {
    let result = SearchResult {
        title: "Test".to_string(),
        url: "https://test.com".to_string(),
        snippet: Some("Snippet".to_string()),
        content: None,
        score: Some(0.85),
        published_date: None,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("Test"));
    assert!(json.contains("https://test.com"));
    assert!(json.contains("0.85"));
}

#[test]
fn test_search_result_deserialize() {
    let json = r#"{
        "title": "Deserialized",
        "url": "https://deser.com",
        "snippet": null,
        "content": "Some content",
        "score": 0.9,
        "published_date": null
    }"#;

    let result: SearchResult = serde_json::from_str(json).unwrap();
    assert_eq!(result.title, "Deserialized");
    assert_eq!(result.url, "https://deser.com");
    assert!(result.snippet.is_none());
    assert_eq!(result.content, Some("Some content".to_string()));
    assert_eq!(result.score, Some(0.9));
}

#[test]
fn test_search_response_serialize() {
    let response = SearchResponse {
        query: "test".to_string(),
        results: vec![],
        answer: Some("Answer".to_string()),
        total_results: Some(10),
    };

    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("test"));
    assert!(json.contains("Answer"));
}

#[test]
fn test_search_response_deserialize() {
    let json = r#"{
        "query": "deserialized query",
        "results": [],
        "answer": null,
        "total_results": 0
    }"#;

    let response: SearchResponse = serde_json::from_str(json).unwrap();
    assert_eq!(response.query, "deserialized query");
    assert!(response.results.is_empty());
}

#[test]
fn test_extracted_content_serialize() {
    let content = ExtractedContent {
        url: "https://test.com".to_string(),
        title: Some("Title".to_string()),
        content: "Content".to_string(),
        markdown: None,
        links: vec!["https://link.com".to_string()],
        images: vec![],
    };

    let json = serde_json::to_string(&content).unwrap();
    assert!(json.contains("https://test.com"));
    assert!(json.contains("Title"));
}

#[test]
fn test_crawl_result_serialize() {
    let result = CrawlResult {
        base_url: "https://base.com".to_string(),
        pages: vec![],
        total_pages: 0,
    };

    let json = serde_json::to_string(&result).unwrap();
    assert!(json.contains("https://base.com"));
    assert!(json.contains("total_pages"));
}
