use swissknife_cloud_sdk::{Object, ListOptions, ListResult};
use std::collections::HashMap;

#[test]
fn test_list_options_default() {
    let options = ListOptions::default();

    assert!(options.prefix.is_none());
    assert!(options.delimiter.is_none());
    assert!(options.max_keys.is_none());
    assert!(options.continuation_token.is_none());
}

#[test]
fn test_list_options_with_prefix() {
    let options = ListOptions {
        prefix: Some("documents/2024/".to_string()),
        delimiter: None,
        max_keys: None,
        continuation_token: None,
    };

    assert_eq!(options.prefix, Some("documents/2024/".to_string()));
}

#[test]
fn test_list_options_with_delimiter() {
    let options = ListOptions {
        prefix: None,
        delimiter: Some("/".to_string()),
        max_keys: None,
        continuation_token: None,
    };

    assert_eq!(options.delimiter, Some("/".to_string()));
}

#[test]
fn test_list_options_with_max_keys() {
    let options = ListOptions {
        prefix: None,
        delimiter: None,
        max_keys: Some(100),
        continuation_token: None,
    };

    assert_eq!(options.max_keys, Some(100));
}

#[test]
fn test_list_options_with_continuation_token() {
    let options = ListOptions {
        prefix: None,
        delimiter: None,
        max_keys: None,
        continuation_token: Some("token123".to_string()),
    };

    assert_eq!(options.continuation_token, Some("token123".to_string()));
}

#[test]
fn test_list_options_full_configuration() {
    let options = ListOptions {
        prefix: Some("images/".to_string()),
        delimiter: Some("/".to_string()),
        max_keys: Some(50),
        continuation_token: Some("next-page".to_string()),
    };

    assert!(options.prefix.is_some());
    assert!(options.delimiter.is_some());
    assert!(options.max_keys.is_some());
    assert!(options.continuation_token.is_some());
}

#[test]
fn test_list_options_clone() {
    let options = ListOptions {
        prefix: Some("test/".to_string()),
        delimiter: None,
        max_keys: Some(10),
        continuation_token: None,
    };

    let cloned = options.clone();
    assert_eq!(options.prefix, cloned.prefix);
    assert_eq!(options.max_keys, cloned.max_keys);
}

#[test]
fn test_list_options_debug() {
    let options = ListOptions::default();
    let debug_str = format!("{:?}", options);
    assert!(debug_str.contains("ListOptions"));
}

#[test]
fn test_list_result_creation() {
    let result = ListResult {
        objects: vec![
            Object {
                key: "file1.txt".to_string(),
                bucket: "bucket".to_string(),
                size: 100,
                content_type: None,
                etag: None,
                last_modified: None,
                storage_class: None,
                metadata: HashMap::new(),
            },
            Object {
                key: "file2.txt".to_string(),
                bucket: "bucket".to_string(),
                size: 200,
                content_type: None,
                etag: None,
                last_modified: None,
                storage_class: None,
                metadata: HashMap::new(),
            },
        ],
        common_prefixes: vec!["folder1/".to_string(), "folder2/".to_string()],
        is_truncated: true,
        next_continuation_token: Some("token123".to_string()),
    };

    assert_eq!(result.objects.len(), 2);
    assert_eq!(result.common_prefixes.len(), 2);
    assert!(result.is_truncated);
    assert!(result.next_continuation_token.is_some());
}

#[test]
fn test_list_result_empty() {
    let result = ListResult {
        objects: vec![],
        common_prefixes: vec![],
        is_truncated: false,
        next_continuation_token: None,
    };

    assert!(result.objects.is_empty());
    assert!(result.common_prefixes.is_empty());
    assert!(!result.is_truncated);
    assert!(result.next_continuation_token.is_none());
}

#[test]
fn test_list_result_not_truncated() {
    let result = ListResult {
        objects: vec![Object {
            key: "single.txt".to_string(),
            bucket: "bucket".to_string(),
            size: 50,
            content_type: None,
            etag: None,
            last_modified: None,
            storage_class: None,
            metadata: HashMap::new(),
        }],
        common_prefixes: vec![],
        is_truncated: false,
        next_continuation_token: None,
    };

    assert!(!result.is_truncated);
    assert!(result.next_continuation_token.is_none());
}

#[test]
fn test_list_result_clone() {
    let result = ListResult {
        objects: vec![],
        common_prefixes: vec!["test/".to_string()],
        is_truncated: false,
        next_continuation_token: None,
    };

    let cloned = result.clone();
    assert_eq!(result.common_prefixes, cloned.common_prefixes);
    assert_eq!(result.is_truncated, cloned.is_truncated);
}

#[test]
fn test_list_result_debug() {
    let result = ListResult {
        objects: vec![],
        common_prefixes: vec![],
        is_truncated: false,
        next_continuation_token: None,
    };

    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("ListResult"));
}
