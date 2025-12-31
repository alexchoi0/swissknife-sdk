use swissknife_database_sdk::{
    Document, FindOptions, InsertResult, UpdateResult, DeleteResult,
};

mod document_tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let doc = Document {
            id: Some("doc123".to_string()),
            data: serde_json::json!({
                "name": "Test Document",
                "count": 42
            }),
        };

        assert_eq!(doc.id, Some("doc123".to_string()));
        assert_eq!(doc.data["name"], "Test Document");
        assert_eq!(doc.data["count"], 42);
    }

    #[test]
    fn test_document_without_id() {
        let doc = Document {
            id: None,
            data: serde_json::json!({"key": "value"}),
        };

        assert!(doc.id.is_none());
    }

    #[test]
    fn test_document_complex_data() {
        let doc = Document {
            id: Some("complex".to_string()),
            data: serde_json::json!({
                "nested": {
                    "level1": {
                        "level2": "deep value"
                    }
                },
                "array": [1, 2, 3],
                "mixed": [{"a": 1}, {"b": 2}]
            }),
        };

        assert_eq!(doc.data["nested"]["level1"]["level2"], "deep value");
        assert!(doc.data["array"].is_array());
    }

    #[test]
    fn test_document_clone() {
        let doc = Document {
            id: Some("original".to_string()),
            data: serde_json::json!({"test": true}),
        };

        let cloned = doc.clone();
        assert_eq!(doc.id, cloned.id);
        assert_eq!(doc.data, cloned.data);
    }

    #[test]
    fn test_document_debug() {
        let doc = Document {
            id: Some("debug_doc".to_string()),
            data: serde_json::json!({}),
        };

        let debug_str = format!("{:?}", doc);
        assert!(debug_str.contains("debug_doc"));
    }

    #[test]
    fn test_document_serialize() {
        let doc = Document {
            id: Some("doc1".to_string()),
            data: serde_json::json!({"key": "value"}),
        };

        let json = serde_json::to_string(&doc).unwrap();
        assert!(json.contains("doc1"));
        assert!(json.contains("key"));
    }
}

mod find_options_tests {
    use super::*;

    #[test]
    fn test_find_options_default() {
        let options = FindOptions::default();

        assert!(options.filter.is_none());
        assert!(options.projection.is_none());
        assert!(options.sort.is_none());
        assert!(options.limit.is_none());
        assert!(options.skip.is_none());
    }

    #[test]
    fn test_find_options_with_filter() {
        let options = FindOptions {
            filter: Some(serde_json::json!({"status": "active"})),
            projection: None,
            sort: None,
            limit: None,
            skip: None,
        };

        assert!(options.filter.is_some());
        assert_eq!(options.filter.as_ref().unwrap()["status"], "active");
    }

    #[test]
    fn test_find_options_with_projection() {
        let options = FindOptions {
            filter: None,
            projection: Some(vec!["name".to_string(), "email".to_string()]),
            sort: None,
            limit: None,
            skip: None,
        };

        let proj = options.projection.unwrap();
        assert_eq!(proj.len(), 2);
        assert!(proj.contains(&"name".to_string()));
    }

    #[test]
    fn test_find_options_with_sort() {
        let options = FindOptions {
            filter: None,
            projection: None,
            sort: Some(serde_json::json!({"created_at": -1})),
            limit: None,
            skip: None,
        };

        assert!(options.sort.is_some());
    }

    #[test]
    fn test_find_options_with_pagination() {
        let options = FindOptions {
            filter: None,
            projection: None,
            sort: None,
            limit: Some(10),
            skip: Some(20),
        };

        assert_eq!(options.limit, Some(10));
        assert_eq!(options.skip, Some(20));
    }

    #[test]
    fn test_find_options_full_configuration() {
        let options = FindOptions {
            filter: Some(serde_json::json!({"active": true})),
            projection: Some(vec!["id".to_string(), "name".to_string()]),
            sort: Some(serde_json::json!({"name": 1})),
            limit: Some(50),
            skip: Some(100),
        };

        assert!(options.filter.is_some());
        assert!(options.projection.is_some());
        assert!(options.sort.is_some());
        assert_eq!(options.limit, Some(50));
        assert_eq!(options.skip, Some(100));
    }

    #[test]
    fn test_find_options_clone() {
        let options = FindOptions {
            filter: Some(serde_json::json!({"test": true})),
            projection: None,
            sort: None,
            limit: Some(5),
            skip: None,
        };

        let cloned = options.clone();
        assert_eq!(options.filter, cloned.filter);
        assert_eq!(options.limit, cloned.limit);
    }

    #[test]
    fn test_find_options_debug() {
        let options = FindOptions::default();
        let debug_str = format!("{:?}", options);
        assert!(debug_str.contains("FindOptions"));
    }
}

mod insert_result_tests {
    use super::*;

    #[test]
    fn test_insert_result_single() {
        let result = InsertResult {
            inserted_id: Some("new_id_123".to_string()),
            inserted_ids: vec!["new_id_123".to_string()],
            inserted_count: 1,
        };

        assert_eq!(result.inserted_id, Some("new_id_123".to_string()));
        assert_eq!(result.inserted_ids.len(), 1);
        assert_eq!(result.inserted_count, 1);
    }

    #[test]
    fn test_insert_result_many() {
        let result = InsertResult {
            inserted_id: Some("id1".to_string()),
            inserted_ids: vec![
                "id1".to_string(),
                "id2".to_string(),
                "id3".to_string(),
            ],
            inserted_count: 3,
        };

        assert_eq!(result.inserted_ids.len(), 3);
        assert_eq!(result.inserted_count, 3);
    }

    #[test]
    fn test_insert_result_clone() {
        let result = InsertResult {
            inserted_id: Some("test".to_string()),
            inserted_ids: vec!["test".to_string()],
            inserted_count: 1,
        };

        let cloned = result.clone();
        assert_eq!(result.inserted_id, cloned.inserted_id);
        assert_eq!(result.inserted_count, cloned.inserted_count);
    }

    #[test]
    fn test_insert_result_debug() {
        let result = InsertResult {
            inserted_id: None,
            inserted_ids: vec![],
            inserted_count: 0,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("InsertResult"));
    }
}

mod update_result_tests {
    use super::*;

    #[test]
    fn test_update_result_matched_modified() {
        let result = UpdateResult {
            matched_count: 5,
            modified_count: 3,
            upserted_id: None,
        };

        assert_eq!(result.matched_count, 5);
        assert_eq!(result.modified_count, 3);
        assert!(result.upserted_id.is_none());
    }

    #[test]
    fn test_update_result_with_upsert() {
        let result = UpdateResult {
            matched_count: 0,
            modified_count: 0,
            upserted_id: Some("new_upserted_id".to_string()),
        };

        assert_eq!(result.matched_count, 0);
        assert_eq!(result.modified_count, 0);
        assert_eq!(result.upserted_id, Some("new_upserted_id".to_string()));
    }

    #[test]
    fn test_update_result_no_changes() {
        let result = UpdateResult {
            matched_count: 10,
            modified_count: 0,
            upserted_id: None,
        };

        assert_eq!(result.matched_count, 10);
        assert_eq!(result.modified_count, 0);
    }

    #[test]
    fn test_update_result_clone() {
        let result = UpdateResult {
            matched_count: 1,
            modified_count: 1,
            upserted_id: None,
        };

        let cloned = result.clone();
        assert_eq!(result.matched_count, cloned.matched_count);
        assert_eq!(result.modified_count, cloned.modified_count);
    }

    #[test]
    fn test_update_result_debug() {
        let result = UpdateResult {
            matched_count: 0,
            modified_count: 0,
            upserted_id: None,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("UpdateResult"));
    }
}

mod delete_result_tests {
    use super::*;

    #[test]
    fn test_delete_result_creation() {
        let result = DeleteResult {
            deleted_count: 5,
        };

        assert_eq!(result.deleted_count, 5);
    }

    #[test]
    fn test_delete_result_zero() {
        let result = DeleteResult {
            deleted_count: 0,
        };

        assert_eq!(result.deleted_count, 0);
    }

    #[test]
    fn test_delete_result_clone() {
        let result = DeleteResult {
            deleted_count: 10,
        };

        let cloned = result.clone();
        assert_eq!(result.deleted_count, cloned.deleted_count);
    }

    #[test]
    fn test_delete_result_debug() {
        let result = DeleteResult {
            deleted_count: 3,
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("DeleteResult") || debug_str.contains("3"));
    }
}
