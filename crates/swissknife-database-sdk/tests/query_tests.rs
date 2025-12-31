use swissknife_database_sdk::{
    QueryResult, ColumnInfo, TableInfo, IndexInfo, QueryParams,
};
use std::collections::HashMap;

mod query_result_tests {
    use super::*;

    #[test]
    fn test_query_result_creation() {
        let result = QueryResult {
            rows: vec![
                {
                    let mut row = HashMap::new();
                    row.insert("id".to_string(), serde_json::json!(1));
                    row.insert("name".to_string(), serde_json::json!("Alice"));
                    row
                },
                {
                    let mut row = HashMap::new();
                    row.insert("id".to_string(), serde_json::json!(2));
                    row.insert("name".to_string(), serde_json::json!("Bob"));
                    row
                },
            ],
            affected_rows: None,
            columns: vec![
                ColumnInfo {
                    name: "id".to_string(),
                    data_type: "integer".to_string(),
                    nullable: false,
                },
                ColumnInfo {
                    name: "name".to_string(),
                    data_type: "varchar".to_string(),
                    nullable: true,
                },
            ],
        };

        assert_eq!(result.rows.len(), 2);
        assert!(result.affected_rows.is_none());
        assert_eq!(result.columns.len(), 2);
    }

    #[test]
    fn test_query_result_empty() {
        let result = QueryResult {
            rows: vec![],
            affected_rows: Some(0),
            columns: vec![],
        };

        assert!(result.rows.is_empty());
        assert_eq!(result.affected_rows, Some(0));
        assert!(result.columns.is_empty());
    }

    #[test]
    fn test_query_result_with_affected_rows() {
        let result = QueryResult {
            rows: vec![],
            affected_rows: Some(5),
            columns: vec![],
        };

        assert_eq!(result.affected_rows, Some(5));
    }

    #[test]
    fn test_query_result_clone() {
        let result = QueryResult {
            rows: vec![{
                let mut row = HashMap::new();
                row.insert("test".to_string(), serde_json::json!("value"));
                row
            }],
            affected_rows: Some(1),
            columns: vec![],
        };

        let cloned = result.clone();
        assert_eq!(result.rows.len(), cloned.rows.len());
        assert_eq!(result.affected_rows, cloned.affected_rows);
    }

    #[test]
    fn test_query_result_debug() {
        let result = QueryResult {
            rows: vec![],
            affected_rows: None,
            columns: vec![],
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("QueryResult"));
    }

    #[test]
    fn test_query_result_serialize() {
        let result = QueryResult {
            rows: vec![],
            affected_rows: Some(5),
            columns: vec![],
        };

        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("affected_rows"));
        assert!(json.contains("5"));
    }
}

mod column_info_tests {
    use super::*;

    #[test]
    fn test_column_info_creation() {
        let column = ColumnInfo {
            name: "user_id".to_string(),
            data_type: "bigint".to_string(),
            nullable: false,
        };

        assert_eq!(column.name, "user_id");
        assert_eq!(column.data_type, "bigint");
        assert!(!column.nullable);
    }

    #[test]
    fn test_column_info_nullable() {
        let column = ColumnInfo {
            name: "description".to_string(),
            data_type: "text".to_string(),
            nullable: true,
        };

        assert!(column.nullable);
    }

    #[test]
    fn test_column_info_various_types() {
        let types = vec!["integer", "varchar", "boolean", "timestamp", "jsonb", "uuid"];

        for data_type in types {
            let column = ColumnInfo {
                name: "test_column".to_string(),
                data_type: data_type.to_string(),
                nullable: false,
            };
            assert_eq!(column.data_type, data_type);
        }
    }

    #[test]
    fn test_column_info_clone() {
        let column = ColumnInfo {
            name: "cloneable".to_string(),
            data_type: "text".to_string(),
            nullable: true,
        };

        let cloned = column.clone();
        assert_eq!(column.name, cloned.name);
        assert_eq!(column.data_type, cloned.data_type);
        assert_eq!(column.nullable, cloned.nullable);
    }

    #[test]
    fn test_column_info_debug() {
        let column = ColumnInfo {
            name: "debug_col".to_string(),
            data_type: "integer".to_string(),
            nullable: false,
        };

        let debug_str = format!("{:?}", column);
        assert!(debug_str.contains("debug_col"));
        assert!(debug_str.contains("integer"));
    }

    #[test]
    fn test_column_info_serialize() {
        let column = ColumnInfo {
            name: "test_col".to_string(),
            data_type: "varchar".to_string(),
            nullable: true,
        };

        let json = serde_json::to_string(&column).unwrap();
        assert!(json.contains("test_col"));
        assert!(json.contains("varchar"));
    }
}

mod table_info_tests {
    use super::*;

    #[test]
    fn test_table_info_creation() {
        let table = TableInfo {
            name: "users".to_string(),
            schema: Some("public".to_string()),
            columns: vec![
                ColumnInfo {
                    name: "id".to_string(),
                    data_type: "serial".to_string(),
                    nullable: false,
                },
                ColumnInfo {
                    name: "email".to_string(),
                    data_type: "varchar".to_string(),
                    nullable: false,
                },
            ],
            primary_key: Some(vec!["id".to_string()]),
            row_count: Some(1000),
        };

        assert_eq!(table.name, "users");
        assert_eq!(table.schema, Some("public".to_string()));
        assert_eq!(table.columns.len(), 2);
        assert_eq!(table.primary_key, Some(vec!["id".to_string()]));
        assert_eq!(table.row_count, Some(1000));
    }

    #[test]
    fn test_table_info_minimal() {
        let table = TableInfo {
            name: "minimal_table".to_string(),
            schema: None,
            columns: vec![],
            primary_key: None,
            row_count: None,
        };

        assert!(table.schema.is_none());
        assert!(table.columns.is_empty());
        assert!(table.primary_key.is_none());
        assert!(table.row_count.is_none());
    }

    #[test]
    fn test_table_info_composite_primary_key() {
        let table = TableInfo {
            name: "order_items".to_string(),
            schema: Some("sales".to_string()),
            columns: vec![],
            primary_key: Some(vec!["order_id".to_string(), "item_id".to_string()]),
            row_count: None,
        };

        let pk = table.primary_key.unwrap();
        assert_eq!(pk.len(), 2);
        assert!(pk.contains(&"order_id".to_string()));
        assert!(pk.contains(&"item_id".to_string()));
    }

    #[test]
    fn test_table_info_clone() {
        let table = TableInfo {
            name: "cloneable_table".to_string(),
            schema: Some("test".to_string()),
            columns: vec![],
            primary_key: None,
            row_count: Some(50),
        };

        let cloned = table.clone();
        assert_eq!(table.name, cloned.name);
        assert_eq!(table.schema, cloned.schema);
        assert_eq!(table.row_count, cloned.row_count);
    }

    #[test]
    fn test_table_info_debug() {
        let table = TableInfo {
            name: "debug_table".to_string(),
            schema: None,
            columns: vec![],
            primary_key: None,
            row_count: None,
        };

        let debug_str = format!("{:?}", table);
        assert!(debug_str.contains("debug_table"));
    }

    #[test]
    fn test_table_info_serialize() {
        let table = TableInfo {
            name: "users".to_string(),
            schema: Some("public".to_string()),
            columns: vec![],
            primary_key: None,
            row_count: Some(100),
        };

        let json = serde_json::to_string(&table).unwrap();
        assert!(json.contains("users"));
        assert!(json.contains("public"));
    }
}

mod index_info_tests {
    use super::*;

    #[test]
    fn test_index_info_creation() {
        let index = IndexInfo {
            name: "idx_users_email".to_string(),
            table: "users".to_string(),
            columns: vec!["email".to_string()],
            unique: true,
            index_type: Some("btree".to_string()),
        };

        assert_eq!(index.name, "idx_users_email");
        assert_eq!(index.table, "users");
        assert_eq!(index.columns.len(), 1);
        assert!(index.unique);
        assert_eq!(index.index_type, Some("btree".to_string()));
    }

    #[test]
    fn test_index_info_non_unique() {
        let index = IndexInfo {
            name: "idx_orders_date".to_string(),
            table: "orders".to_string(),
            columns: vec!["created_at".to_string()],
            unique: false,
            index_type: None,
        };

        assert!(!index.unique);
        assert!(index.index_type.is_none());
    }

    #[test]
    fn test_index_info_composite() {
        let index = IndexInfo {
            name: "idx_composite".to_string(),
            table: "test".to_string(),
            columns: vec!["col1".to_string(), "col2".to_string(), "col3".to_string()],
            unique: false,
            index_type: Some("hash".to_string()),
        };

        assert_eq!(index.columns.len(), 3);
    }

    #[test]
    fn test_index_info_clone() {
        let index = IndexInfo {
            name: "cloneable_idx".to_string(),
            table: "test".to_string(),
            columns: vec!["col".to_string()],
            unique: true,
            index_type: Some("gin".to_string()),
        };

        let cloned = index.clone();
        assert_eq!(index.name, cloned.name);
        assert_eq!(index.unique, cloned.unique);
        assert_eq!(index.index_type, cloned.index_type);
    }

    #[test]
    fn test_index_info_debug() {
        let index = IndexInfo {
            name: "debug_idx".to_string(),
            table: "debug_table".to_string(),
            columns: vec![],
            unique: false,
            index_type: None,
        };

        let debug_str = format!("{:?}", index);
        assert!(debug_str.contains("debug_idx"));
    }
}

mod query_params_tests {
    use super::*;

    #[test]
    fn test_query_params_new() {
        let params = QueryParams::new();
        assert!(params.params.is_empty());
    }

    #[test]
    fn test_query_params_default() {
        let params = QueryParams::default();
        assert!(params.params.is_empty());
    }

    #[test]
    fn test_query_params_bind_string() {
        let params = QueryParams::new()
            .bind("hello")
            .bind("world");

        assert_eq!(params.params.len(), 2);
        assert_eq!(params.params[0], serde_json::json!("hello"));
        assert_eq!(params.params[1], serde_json::json!("world"));
    }

    #[test]
    fn test_query_params_bind_integer() {
        let params = QueryParams::new()
            .bind(42)
            .bind(-17)
            .bind(0);

        assert_eq!(params.params.len(), 3);
        assert_eq!(params.params[0], serde_json::json!(42));
        assert_eq!(params.params[1], serde_json::json!(-17));
        assert_eq!(params.params[2], serde_json::json!(0));
    }

    #[test]
    fn test_query_params_bind_float() {
        let params = QueryParams::new()
            .bind(3.14)
            .bind(2.718);

        assert_eq!(params.params.len(), 2);
    }

    #[test]
    fn test_query_params_bind_boolean() {
        let params = QueryParams::new()
            .bind(true)
            .bind(false);

        assert_eq!(params.params.len(), 2);
        assert_eq!(params.params[0], serde_json::json!(true));
        assert_eq!(params.params[1], serde_json::json!(false));
    }

    #[test]
    fn test_query_params_bind_mixed_types() {
        let params = QueryParams::new()
            .bind("text")
            .bind(123)
            .bind(true)
            .bind(45.67);

        assert_eq!(params.params.len(), 4);
    }

    #[test]
    fn test_query_params_bind_null() {
        let params = QueryParams::new()
            .bind(serde_json::Value::Null);

        assert_eq!(params.params.len(), 1);
        assert!(params.params[0].is_null());
    }

    #[test]
    fn test_query_params_chaining() {
        let params = QueryParams::new()
            .bind("first")
            .bind("second")
            .bind("third")
            .bind("fourth")
            .bind("fifth");

        assert_eq!(params.params.len(), 5);
    }

    #[test]
    fn test_query_params_debug() {
        let params = QueryParams::new().bind("debug_value");
        let debug_str = format!("{:?}", params);
        assert!(debug_str.contains("params"));
    }
}
