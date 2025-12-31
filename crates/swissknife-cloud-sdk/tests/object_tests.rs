use swissknife_cloud_sdk::{Object, ObjectVersion};
use chrono::Utc;
use std::collections::HashMap;

#[test]
fn test_object_creation() {
    let mut metadata = HashMap::new();
    metadata.insert("author".to_string(), "test-user".to_string());

    let object = Object {
        key: "documents/report.pdf".to_string(),
        bucket: "my-bucket".to_string(),
        size: 1024 * 1024,
        content_type: Some("application/pdf".to_string()),
        etag: Some("abc123".to_string()),
        last_modified: Some(Utc::now()),
        storage_class: Some("STANDARD".to_string()),
        metadata,
    };

    assert_eq!(object.key, "documents/report.pdf");
    assert_eq!(object.bucket, "my-bucket");
    assert_eq!(object.size, 1024 * 1024);
    assert_eq!(object.content_type, Some("application/pdf".to_string()));
    assert!(object.metadata.contains_key("author"));
}

#[test]
fn test_object_minimal() {
    let object = Object {
        key: "simple.txt".to_string(),
        bucket: "bucket".to_string(),
        size: 100,
        content_type: None,
        etag: None,
        last_modified: None,
        storage_class: None,
        metadata: HashMap::new(),
    };

    assert_eq!(object.key, "simple.txt");
    assert!(object.content_type.is_none());
    assert!(object.metadata.is_empty());
}

#[test]
fn test_object_various_content_types() {
    let content_types = vec![
        "text/plain",
        "application/json",
        "image/png",
        "image/jpeg",
        "video/mp4",
        "application/octet-stream",
    ];

    for ct in content_types {
        let object = Object {
            key: "file".to_string(),
            bucket: "bucket".to_string(),
            size: 0,
            content_type: Some(ct.to_string()),
            etag: None,
            last_modified: None,
            storage_class: None,
            metadata: HashMap::new(),
        };
        assert_eq!(object.content_type, Some(ct.to_string()));
    }
}

#[test]
fn test_object_storage_classes() {
    let storage_classes = vec![
        "STANDARD",
        "STANDARD_IA",
        "ONEZONE_IA",
        "INTELLIGENT_TIERING",
        "GLACIER",
        "DEEP_ARCHIVE",
    ];

    for sc in storage_classes {
        let object = Object {
            key: "file".to_string(),
            bucket: "bucket".to_string(),
            size: 0,
            content_type: None,
            etag: None,
            last_modified: None,
            storage_class: Some(sc.to_string()),
            metadata: HashMap::new(),
        };
        assert_eq!(object.storage_class, Some(sc.to_string()));
    }
}

#[test]
fn test_object_with_metadata() {
    let mut metadata = HashMap::new();
    metadata.insert("x-custom-header".to_string(), "custom-value".to_string());
    metadata.insert("version".to_string(), "1.0".to_string());
    metadata.insert("checksum".to_string(), "sha256:abc".to_string());

    let object = Object {
        key: "with-metadata.txt".to_string(),
        bucket: "bucket".to_string(),
        size: 500,
        content_type: None,
        etag: None,
        last_modified: None,
        storage_class: None,
        metadata,
    };

    assert_eq!(object.metadata.len(), 3);
    assert_eq!(
        object.metadata.get("version"),
        Some(&"1.0".to_string())
    );
}

#[test]
fn test_object_clone() {
    let object = Object {
        key: "clone-test.txt".to_string(),
        bucket: "bucket".to_string(),
        size: 200,
        content_type: Some("text/plain".to_string()),
        etag: None,
        last_modified: None,
        storage_class: None,
        metadata: HashMap::new(),
    };

    let cloned = object.clone();
    assert_eq!(object.key, cloned.key);
    assert_eq!(object.size, cloned.size);
    assert_eq!(object.content_type, cloned.content_type);
}

#[test]
fn test_object_debug() {
    let object = Object {
        key: "debug.txt".to_string(),
        bucket: "bucket".to_string(),
        size: 0,
        content_type: None,
        etag: None,
        last_modified: None,
        storage_class: None,
        metadata: HashMap::new(),
    };

    let debug_str = format!("{:?}", object);
    assert!(debug_str.contains("debug.txt"));
}

#[test]
fn test_object_version_creation() {
    let version = ObjectVersion {
        key: "versioned-file.txt".to_string(),
        version_id: "v123456".to_string(),
        is_latest: true,
        last_modified: Utc::now(),
        size: 1000,
    };

    assert_eq!(version.key, "versioned-file.txt");
    assert_eq!(version.version_id, "v123456");
    assert!(version.is_latest);
    assert_eq!(version.size, 1000);
}

#[test]
fn test_object_version_not_latest() {
    let version = ObjectVersion {
        key: "old-file.txt".to_string(),
        version_id: "v000001".to_string(),
        is_latest: false,
        last_modified: Utc::now(),
        size: 500,
    };

    assert!(!version.is_latest);
}

#[test]
fn test_object_version_clone() {
    let version = ObjectVersion {
        key: "clone.txt".to_string(),
        version_id: "v999".to_string(),
        is_latest: true,
        last_modified: Utc::now(),
        size: 100,
    };

    let cloned = version.clone();
    assert_eq!(version.key, cloned.key);
    assert_eq!(version.version_id, cloned.version_id);
    assert_eq!(version.is_latest, cloned.is_latest);
}

#[test]
fn test_object_version_debug() {
    let version = ObjectVersion {
        key: "debug-version.txt".to_string(),
        version_id: "debug-v1".to_string(),
        is_latest: false,
        last_modified: Utc::now(),
        size: 0,
    };

    let debug_str = format!("{:?}", version);
    assert!(debug_str.contains("debug-version.txt") || debug_str.contains("debug-v1"));
}

#[test]
fn test_object_serialize() {
    let object = Object {
        key: "file.txt".to_string(),
        bucket: "bucket".to_string(),
        size: 1024,
        content_type: Some("text/plain".to_string()),
        etag: None,
        last_modified: None,
        storage_class: None,
        metadata: HashMap::new(),
    };

    let json = serde_json::to_string(&object).unwrap();
    assert!(json.contains("file.txt"));
    assert!(json.contains("1024"));
}

#[test]
fn test_object_version_serialize() {
    let version = ObjectVersion {
        key: "versioned.txt".to_string(),
        version_id: "v1".to_string(),
        is_latest: true,
        last_modified: Utc::now(),
        size: 500,
    };

    let json = serde_json::to_string(&version).unwrap();
    assert!(json.contains("versioned.txt"));
    assert!(json.contains("v1"));
}
