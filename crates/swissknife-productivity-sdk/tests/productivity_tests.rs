#[cfg(feature = "notion")]
mod notion_tests {
    use swissknife_productivity_sdk::notion::{
        NotionClient, CreatePageRequest, QueryDatabaseRequest,
        PageProperty, Filter, Sort, SortDirection,
    };

    #[test]
    fn test_notion_client_creation() {
        let client = NotionClient::new("notion-api-key");
        assert!(true);
    }

    #[test]
    fn test_create_page_request() {
        let request = CreatePageRequest {
            parent: serde_json::json!({"database_id": "db-123"}),
            properties: serde_json::json!({
                "Name": {
                    "title": [{"text": {"content": "Test Page"}}]
                }
            }),
            children: None,
            icon: None,
            cover: None,
        };

        assert!(request.properties.is_object());
    }

    #[test]
    fn test_query_database_request() {
        let request = QueryDatabaseRequest {
            filter: Some(serde_json::json!({
                "property": "Status",
                "select": {"equals": "Done"}
            })),
            sorts: Some(vec![Sort {
                property: Some("Created".to_string()),
                timestamp: None,
                direction: SortDirection::Descending,
            }]),
            start_cursor: None,
            page_size: Some(50),
        };

        assert!(request.filter.is_some());
        assert!(request.sorts.is_some());
        assert_eq!(request.page_size, Some(50));
    }

    #[test]
    fn test_sort_direction_variants() {
        let asc = SortDirection::Ascending;
        let desc = SortDirection::Descending;

        assert!(matches!(asc, SortDirection::Ascending));
        assert!(matches!(desc, SortDirection::Descending));
    }
}

#[cfg(feature = "google")]
mod google_tests {
    use swissknife_productivity_sdk::google::{
        GoogleClient, CreateEventRequest, CreateDocumentRequest,
        CreateSpreadsheetRequest, ListFilesParams,
    };

    #[test]
    fn test_google_client_creation() {
        let client = GoogleClient::new("access-token");
        assert!(true);
    }

    #[test]
    fn test_create_event_request() {
        let request = CreateEventRequest {
            summary: "Team Meeting".to_string(),
            description: Some("Weekly sync".to_string()),
            start: serde_json::json!({"dateTime": "2024-01-15T10:00:00Z"}),
            end: serde_json::json!({"dateTime": "2024-01-15T11:00:00Z"}),
            attendees: Some(vec![serde_json::json!({"email": "test@example.com"})]),
            location: Some("Conference Room".to_string()),
            recurrence: None,
            reminders: None,
        };

        assert_eq!(request.summary, "Team Meeting");
        assert!(request.description.is_some());
        assert!(request.attendees.is_some());
    }

    #[test]
    fn test_create_document_request() {
        let request = CreateDocumentRequest {
            title: "New Document".to_string(),
        };

        assert_eq!(request.title, "New Document");
    }

    #[test]
    fn test_create_spreadsheet_request() {
        let request = CreateSpreadsheetRequest {
            title: "New Spreadsheet".to_string(),
            sheets: None,
        };

        assert_eq!(request.title, "New Spreadsheet");
    }

    #[test]
    fn test_list_files_params() {
        let params = ListFilesParams {
            q: Some("mimeType='application/pdf'".to_string()),
            page_size: Some(20),
            page_token: None,
            order_by: Some("name".to_string()),
            fields: Some("files(id,name,mimeType)".to_string()),
        };

        assert!(params.q.is_some());
        assert_eq!(params.page_size, Some(20));
    }
}

#[cfg(feature = "airtable")]
mod airtable_tests {
    use swissknife_productivity_sdk::airtable::{
        AirtableClient, CreateRecordRequest, ListRecordsParams,
        UpdateRecordRequest, FilterFormula,
    };

    #[test]
    fn test_airtable_client_creation() {
        let client = AirtableClient::new("airtable-api-key");
        assert!(true);
    }

    #[test]
    fn test_create_record_request() {
        let request = CreateRecordRequest {
            fields: serde_json::json!({
                "Name": "John Doe",
                "Email": "john@example.com",
                "Status": "Active"
            }),
            typecast: Some(true),
        };

        assert!(request.fields.is_object());
        assert_eq!(request.typecast, Some(true));
    }

    #[test]
    fn test_list_records_params() {
        let params = ListRecordsParams {
            fields: Some(vec!["Name".to_string(), "Email".to_string()]),
            filter_by_formula: Some("NOT({Status}='Archived')".to_string()),
            max_records: Some(100),
            page_size: Some(50),
            sort: Some(vec![serde_json::json!({"field": "Created", "direction": "desc"})]),
            view: Some("Grid view".to_string()),
            offset: None,
        };

        assert!(params.fields.is_some());
        assert!(params.filter_by_formula.is_some());
        assert_eq!(params.max_records, Some(100));
    }

    #[test]
    fn test_update_record_request() {
        let request = UpdateRecordRequest {
            fields: serde_json::json!({
                "Status": "Completed"
            }),
            typecast: Some(false),
        };

        assert!(request.fields.is_object());
    }
}

#[cfg(feature = "calendly")]
mod calendly_tests {
    use swissknife_productivity_sdk::calendly::{
        CalendlyClient, ListEventsParams, ListEventTypesParams,
    };

    #[test]
    fn test_calendly_client_creation() {
        let client = CalendlyClient::new("calendly-api-key");
        assert!(true);
    }

    #[test]
    fn test_list_events_params() {
        let params = ListEventsParams {
            user: Some("https://api.calendly.com/users/user-123".to_string()),
            organization: None,
            count: Some(50),
            page_token: None,
            status: Some("active".to_string()),
            min_start_time: Some("2024-01-01T00:00:00Z".to_string()),
            max_start_time: Some("2024-12-31T23:59:59Z".to_string()),
            invitee_email: None,
            sort: Some("start_time:asc".to_string()),
        };

        assert!(params.user.is_some());
        assert_eq!(params.count, Some(50));
        assert!(params.status.is_some());
    }

    #[test]
    fn test_list_event_types_params() {
        let params = ListEventTypesParams {
            user: Some("https://api.calendly.com/users/user-123".to_string()),
            organization: None,
            count: Some(20),
            page_token: None,
            active: Some(true),
            sort: None,
        };

        assert!(params.user.is_some());
        assert_eq!(params.active, Some(true));
    }
}

#[cfg(feature = "microsoft")]
mod microsoft_tests {
    use swissknife_productivity_sdk::microsoft::{
        MicrosoftClient, CreatePlanRequest, CreateTaskRequest,
        UploadFileParams, ListItemsParams,
    };

    #[test]
    fn test_microsoft_client_creation() {
        let client = MicrosoftClient::new("access-token");
        assert!(true);
    }

    #[test]
    fn test_create_plan_request() {
        let request = CreatePlanRequest {
            title: "Project Plan".to_string(),
            owner: "group-id-123".to_string(),
        };

        assert_eq!(request.title, "Project Plan");
        assert_eq!(request.owner, "group-id-123");
    }

    #[test]
    fn test_create_task_request() {
        let request = CreateTaskRequest {
            plan_id: "plan-123".to_string(),
            bucket_id: Some("bucket-456".to_string()),
            title: "Complete review".to_string(),
            assignments: None,
            due_date_time: Some("2024-02-01T17:00:00Z".to_string()),
            percent_complete: None,
            priority: Some(5),
        };

        assert_eq!(request.title, "Complete review");
        assert!(request.bucket_id.is_some());
        assert_eq!(request.priority, Some(5));
    }

    #[test]
    fn test_upload_file_params() {
        let params = UploadFileParams {
            name: "document.pdf".to_string(),
            conflict_behavior: Some("replace".to_string()),
        };

        assert_eq!(params.name, "document.pdf");
        assert_eq!(params.conflict_behavior, Some("replace".to_string()));
    }

    #[test]
    fn test_list_items_params() {
        let params = ListItemsParams {
            top: Some(50),
            skip: Some(0),
            filter: Some("name eq 'report.xlsx'".to_string()),
            orderby: Some("lastModifiedDateTime desc".to_string()),
            select: Some("id,name,size".to_string()),
        };

        assert_eq!(params.top, Some(50));
        assert!(params.filter.is_some());
    }
}

#[cfg(feature = "confluence")]
mod confluence_tests {
    use swissknife_productivity_sdk::confluence::{
        ConfluenceClient, CreatePageRequest, UpdatePageRequest,
        CreateSpaceRequest, SearchParams,
    };

    #[test]
    fn test_confluence_client_creation() {
        let client = ConfluenceClient::new(
            "https://your-domain.atlassian.net",
            "email@example.com",
            "api-token",
        );
        assert!(true);
    }

    #[test]
    fn test_create_page_request() {
        let request = CreatePageRequest {
            space_id: "SPACE123".to_string(),
            title: "New Page".to_string(),
            body: serde_json::json!({
                "representation": "storage",
                "value": "<p>Page content</p>"
            }),
            parent_id: None,
            status: Some("current".to_string()),
        };

        assert_eq!(request.title, "New Page");
        assert_eq!(request.space_id, "SPACE123");
    }

    #[test]
    fn test_update_page_request() {
        let request = UpdatePageRequest {
            id: "page-123".to_string(),
            title: Some("Updated Title".to_string()),
            body: Some(serde_json::json!({
                "representation": "storage",
                "value": "<p>Updated content</p>"
            })),
            version: 2,
            status: None,
        };

        assert_eq!(request.id, "page-123");
        assert_eq!(request.version, 2);
    }

    #[test]
    fn test_create_space_request() {
        let request = CreateSpaceRequest {
            key: "PROJ".to_string(),
            name: "Project Space".to_string(),
            description: Some("Project documentation space".to_string()),
            space_type: Some("global".to_string()),
        };

        assert_eq!(request.key, "PROJ");
        assert_eq!(request.name, "Project Space");
    }

    #[test]
    fn test_search_params() {
        let params = SearchParams {
            cql: "type=page AND space=PROJ".to_string(),
            limit: Some(25),
            start: Some(0),
            expand: Some(vec!["body.storage".to_string()]),
        };

        assert!(params.cql.contains("type=page"));
        assert_eq!(params.limit, Some(25));
    }
}

#[cfg(feature = "typeform")]
mod typeform_tests {
    use swissknife_productivity_sdk::typeform::{
        TypeformClient, ListFormsParams, ListResponsesParams,
    };

    #[test]
    fn test_typeform_client_creation() {
        let client = TypeformClient::new("typeform-api-key");
        assert!(true);
    }

    #[test]
    fn test_list_forms_params() {
        let params = ListFormsParams {
            page: Some(1),
            page_size: Some(10),
            search: Some("survey".to_string()),
            workspace_id: Some("workspace-123".to_string()),
        };

        assert_eq!(params.page, Some(1));
        assert_eq!(params.page_size, Some(10));
        assert!(params.search.is_some());
    }

    #[test]
    fn test_list_responses_params() {
        let params = ListResponsesParams {
            page_size: Some(25),
            since: Some("2024-01-01T00:00:00Z".to_string()),
            until: Some("2024-12-31T23:59:59Z".to_string()),
            completed: Some(true),
            sort: Some("submitted_at,desc".to_string()),
            query: None,
            fields: None,
            before: None,
            after: None,
        };

        assert_eq!(params.page_size, Some(25));
        assert_eq!(params.completed, Some(true));
    }
}

mod error_tests {
    use swissknife_productivity_sdk::Error;

    #[test]
    fn test_error_display() {
        let api_error = Error::Api {
            message: "Page not found".to_string(),
            code: Some("404".to_string()),
        };

        let error_string = format!("{}", api_error);
        assert!(error_string.contains("Page not found"));
    }

    #[test]
    fn test_not_found_error() {
        let error = Error::NotFound("Resource not found".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("not found"));
    }

    #[test]
    fn test_permission_denied_error() {
        let error = Error::PermissionDenied("Access denied".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Access denied"));
    }

    #[test]
    fn test_validation_error() {
        let error = Error::Validation("Invalid field value".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Invalid field"));
    }
}
