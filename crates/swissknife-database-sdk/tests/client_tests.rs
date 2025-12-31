#[cfg(feature = "postgres")]
mod postgres {
    use swissknife_database_sdk::postgres::PostgresClient;

    #[test]
    fn test_postgres_client_new() {
        let client = PostgresClient::new("http://localhost:5432");
        assert_eq!(client.base_url(), "http://localhost:5432");
    }

    #[test]
    fn test_postgres_client_trailing_slash() {
        let client = PostgresClient::new("http://localhost:5432/");
        assert_eq!(client.base_url(), "http://localhost:5432");
    }
}

#[cfg(feature = "mongodb")]
mod mongodb {
    use swissknife_database_sdk::mongodb::MongoClient;

    #[test]
    fn test_mongo_client_new() {
        let client = MongoClient::new(
            "https://data.mongodb-api.com",
            "api-key-123",
            "Cluster0",
            "test_db",
        );
        let _ = client;
    }
}

#[cfg(feature = "redis")]
mod redis {
    use swissknife_database_sdk::redis::RedisClient;

    #[test]
    fn test_redis_client_new() {
        let client = RedisClient::new("http://localhost:6379");
        let _ = client;
    }

    #[test]
    fn test_redis_client_with_auth() {
        let client = RedisClient::new("http://localhost:6379")
            .with_auth("secret-token");
        let _ = client;
    }
}

#[cfg(feature = "elasticsearch")]
mod elasticsearch {
    use swissknife_database_sdk::elasticsearch::ElasticsearchClient;

    #[test]
    fn test_elasticsearch_client_new() {
        let client = ElasticsearchClient::new("http://localhost:9200");
        let _ = client;
    }

    #[test]
    fn test_elasticsearch_client_with_api_key() {
        let client = ElasticsearchClient::new("http://localhost:9200")
            .with_api_key("my-api-key");
        let _ = client;
    }

    #[test]
    fn test_elasticsearch_client_with_basic_auth() {
        let client = ElasticsearchClient::new("http://localhost:9200")
            .with_basic_auth("elastic", "password");
        let _ = client;
    }
}
