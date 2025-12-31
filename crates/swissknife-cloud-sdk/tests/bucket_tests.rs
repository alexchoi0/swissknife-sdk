use swissknife_cloud_sdk::Bucket;
use chrono::{Utc, TimeZone};

#[test]
fn test_bucket_creation() {
    let bucket = Bucket {
        name: "my-bucket".to_string(),
        region: Some("us-east-1".to_string()),
        created_at: Some(Utc.with_ymd_and_hms(2024, 1, 15, 10, 30, 0).unwrap()),
    };

    assert_eq!(bucket.name, "my-bucket");
    assert_eq!(bucket.region, Some("us-east-1".to_string()));
    assert!(bucket.created_at.is_some());
}

#[test]
fn test_bucket_minimal() {
    let bucket = Bucket {
        name: "minimal-bucket".to_string(),
        region: None,
        created_at: None,
    };

    assert_eq!(bucket.name, "minimal-bucket");
    assert!(bucket.region.is_none());
    assert!(bucket.created_at.is_none());
}

#[test]
fn test_bucket_various_regions() {
    let regions = vec![
        "us-east-1",
        "us-west-2",
        "eu-west-1",
        "ap-southeast-1",
        "sa-east-1",
    ];

    for region in regions {
        let bucket = Bucket {
            name: format!("bucket-{}", region),
            region: Some(region.to_string()),
            created_at: None,
        };
        assert_eq!(bucket.region, Some(region.to_string()));
    }
}

#[test]
fn test_bucket_clone() {
    let bucket = Bucket {
        name: "cloneable".to_string(),
        region: Some("eu-central-1".to_string()),
        created_at: None,
    };

    let cloned = bucket.clone();
    assert_eq!(bucket.name, cloned.name);
    assert_eq!(bucket.region, cloned.region);
}

#[test]
fn test_bucket_debug() {
    let bucket = Bucket {
        name: "debug-bucket".to_string(),
        region: None,
        created_at: None,
    };

    let debug_str = format!("{:?}", bucket);
    assert!(debug_str.contains("debug-bucket"));
}

#[test]
fn test_bucket_serialize() {
    let bucket = Bucket {
        name: "test-bucket".to_string(),
        region: Some("us-east-1".to_string()),
        created_at: None,
    };

    let json = serde_json::to_string(&bucket).unwrap();
    assert!(json.contains("test-bucket"));
    assert!(json.contains("us-east-1"));
}
