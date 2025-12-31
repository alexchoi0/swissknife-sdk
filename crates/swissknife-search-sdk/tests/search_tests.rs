#[cfg(feature = "tavily")]
mod tavily_tests {
    use swissknife_search_sdk::tavily::{
        TavilyClient, SearchRequest, SearchDepth, Topic,
    };

    #[test]
    fn test_tavily_client_creation() {
        let client = TavilyClient::new("tvly-api-key");
        assert!(true);
    }

    #[test]
    fn test_search_request_basic() {
        let request = SearchRequest::new("What is Rust programming language?");

        assert_eq!(request.query, "What is Rust programming language?");
        assert!(request.search_depth.is_none());
        assert!(request.topic.is_none());
    }

    #[test]
    fn test_search_request_builder() {
        let request = SearchRequest::new("machine learning trends 2024")
            .with_search_depth(SearchDepth::Advanced)
            .with_topic(Topic::News)
            .with_max_results(10)
            .with_include_answer(true)
            .with_include_raw_content(false);

        assert_eq!(request.query, "machine learning trends 2024");
        assert!(matches!(request.search_depth, Some(SearchDepth::Advanced)));
        assert!(matches!(request.topic, Some(Topic::News)));
        assert_eq!(request.max_results, Some(10));
        assert_eq!(request.include_answer, Some(true));
    }

    #[test]
    fn test_search_request_with_domains() {
        let request = SearchRequest::new("rust async")
            .with_include_domains(vec!["rust-lang.org".to_string(), "docs.rs".to_string()])
            .with_exclude_domains(vec!["spam.com".to_string()]);

        assert!(request.include_domains.is_some());
        assert!(request.exclude_domains.is_some());
    }

    #[test]
    fn test_search_depth_variants() {
        let basic = SearchDepth::Basic;
        let advanced = SearchDepth::Advanced;

        assert!(matches!(basic, SearchDepth::Basic));
        assert!(matches!(advanced, SearchDepth::Advanced));
    }

    #[test]
    fn test_topic_variants() {
        let general = Topic::General;
        let news = Topic::News;

        assert!(matches!(general, Topic::General));
        assert!(matches!(news, Topic::News));
    }
}

#[cfg(feature = "exa")]
mod exa_tests {
    use swissknife_search_sdk::exa::{
        ExaClient, SearchRequest, SearchType, ContentOptions, Category,
    };

    #[test]
    fn test_exa_client_creation() {
        let client = ExaClient::new("exa-api-key");
        assert!(true);
    }

    #[test]
    fn test_search_request_basic() {
        let request = SearchRequest::new("best programming languages 2024");

        assert_eq!(request.query, "best programming languages 2024");
    }

    #[test]
    fn test_search_request_builder() {
        let request = SearchRequest::new("AI research papers")
            .with_search_type(SearchType::Neural)
            .with_num_results(25)
            .with_category(Category::ResearchPaper);

        assert_eq!(request.query, "AI research papers");
        assert!(matches!(request.search_type, Some(SearchType::Neural)));
        assert_eq!(request.num_results, Some(25));
    }

    #[test]
    fn test_content_options() {
        let options = ContentOptions {
            text: Some(true),
            highlights: Some(true),
            summary: Some(true),
        };

        assert_eq!(options.text, Some(true));
        assert_eq!(options.highlights, Some(true));
        assert_eq!(options.summary, Some(true));
    }

    #[test]
    fn test_search_type_variants() {
        let auto = SearchType::Auto;
        let neural = SearchType::Neural;
        let keyword = SearchType::Keyword;

        assert!(matches!(auto, SearchType::Auto));
        assert!(matches!(neural, SearchType::Neural));
        assert!(matches!(keyword, SearchType::Keyword));
    }

    #[test]
    fn test_category_variants() {
        let research = Category::ResearchPaper;
        let news = Category::News;
        let company = Category::Company;

        assert!(matches!(research, Category::ResearchPaper));
        assert!(matches!(news, Category::News));
        assert!(matches!(company, Category::Company));
    }
}

#[cfg(feature = "serper")]
mod serper_tests {
    use swissknife_search_sdk::serper::{
        SerperClient, SearchRequest, SearchType,
    };

    #[test]
    fn test_serper_client_creation() {
        let client = SerperClient::new("serper-api-key");
        assert!(true);
    }

    #[test]
    fn test_search_request_basic() {
        let request = SearchRequest::new("latest tech news");

        assert_eq!(request.q, "latest tech news");
    }

    #[test]
    fn test_search_request_builder() {
        let request = SearchRequest::new("rust programming")
            .with_search_type(SearchType::Search)
            .with_num(20)
            .with_gl("us")
            .with_hl("en");

        assert_eq!(request.q, "rust programming");
        assert_eq!(request.num, Some(20));
        assert_eq!(request.gl, Some("us".to_string()));
        assert_eq!(request.hl, Some("en".to_string()));
    }

    #[test]
    fn test_search_type_variants() {
        let search = SearchType::Search;
        let images = SearchType::Images;
        let news = SearchType::News;
        let places = SearchType::Places;

        assert!(matches!(search, SearchType::Search));
        assert!(matches!(images, SearchType::Images));
        assert!(matches!(news, SearchType::News));
        assert!(matches!(places, SearchType::Places));
    }
}

#[cfg(feature = "perplexity")]
mod perplexity_tests {
    use swissknife_search_sdk::perplexity::{
        PerplexityClient, ChatRequest, Message, Role,
    };

    #[test]
    fn test_perplexity_client_creation() {
        let client = PerplexityClient::new("pplx-api-key");
        assert!(true);
    }

    #[test]
    fn test_chat_request_basic() {
        let messages = vec![
            Message {
                role: Role::User,
                content: "What is quantum computing?".to_string(),
            },
        ];
        let request = ChatRequest::new("llama-3.1-sonar-small-128k-online", messages);

        assert_eq!(request.model, "llama-3.1-sonar-small-128k-online");
        assert_eq!(request.messages.len(), 1);
    }

    #[test]
    fn test_chat_request_with_system() {
        let messages = vec![
            Message {
                role: Role::System,
                content: "You are a helpful assistant.".to_string(),
            },
            Message {
                role: Role::User,
                content: "Explain machine learning.".to_string(),
            },
        ];
        let request = ChatRequest::new("llama-3.1-sonar-large-128k-online", messages)
            .with_temperature(0.7)
            .with_max_tokens(1000);

        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(1000));
    }

    #[test]
    fn test_role_variants() {
        let system = Role::System;
        let user = Role::User;
        let assistant = Role::Assistant;

        assert!(matches!(system, Role::System));
        assert!(matches!(user, Role::User));
        assert!(matches!(assistant, Role::Assistant));
    }
}

#[cfg(feature = "duckduckgo")]
mod duckduckgo_tests {
    use swissknife_search_sdk::duckduckgo::{
        DuckDuckGoClient, SearchParams, Region, SafeSearch,
    };

    #[test]
    fn test_duckduckgo_client_creation() {
        let client = DuckDuckGoClient::new();
        assert!(true);
    }

    #[test]
    fn test_search_params_basic() {
        let params = SearchParams::new("rust programming");

        assert_eq!(params.query, "rust programming");
    }

    #[test]
    fn test_search_params_builder() {
        let params = SearchParams::new("latest tech news")
            .with_region(Region::UsEn)
            .with_safe_search(SafeSearch::Moderate)
            .with_max_results(20);

        assert_eq!(params.query, "latest tech news");
        assert!(matches!(params.region, Some(Region::UsEn)));
        assert!(matches!(params.safe_search, Some(SafeSearch::Moderate)));
        assert_eq!(params.max_results, Some(20));
    }

    #[test]
    fn test_region_variants() {
        let us_en = Region::UsEn;
        let uk_en = Region::UkEn;
        let de_de = Region::DeDe;

        assert!(matches!(us_en, Region::UsEn));
        assert!(matches!(uk_en, Region::UkEn));
        assert!(matches!(de_de, Region::DeDe));
    }

    #[test]
    fn test_safe_search_variants() {
        let strict = SafeSearch::Strict;
        let moderate = SafeSearch::Moderate;
        let off = SafeSearch::Off;

        assert!(matches!(strict, SafeSearch::Strict));
        assert!(matches!(moderate, SafeSearch::Moderate));
        assert!(matches!(off, SafeSearch::Off));
    }
}

#[cfg(feature = "wikipedia")]
mod wikipedia_tests {
    use swissknife_search_sdk::wikipedia::{
        WikipediaClient, SearchParams, Language,
    };

    #[test]
    fn test_wikipedia_client_creation() {
        let client = WikipediaClient::new();
        assert!(true);
    }

    #[test]
    fn test_wikipedia_client_with_language() {
        let client = WikipediaClient::with_language(Language::German);
        assert!(true);
    }

    #[test]
    fn test_search_params_basic() {
        let params = SearchParams::new("machine learning");

        assert_eq!(params.query, "machine learning");
    }

    #[test]
    fn test_search_params_builder() {
        let params = SearchParams::new("quantum physics")
            .with_limit(15)
            .with_namespace(0);

        assert_eq!(params.query, "quantum physics");
        assert_eq!(params.limit, Some(15));
        assert_eq!(params.namespace, Some(0));
    }

    #[test]
    fn test_language_variants() {
        let en = Language::English;
        let de = Language::German;
        let fr = Language::French;
        let es = Language::Spanish;
        let zh = Language::Chinese;
        let ja = Language::Japanese;

        assert!(matches!(en, Language::English));
        assert!(matches!(de, Language::German));
        assert!(matches!(fr, Language::French));
        assert!(matches!(es, Language::Spanish));
        assert!(matches!(zh, Language::Chinese));
        assert!(matches!(ja, Language::Japanese));
    }
}

#[cfg(feature = "google")]
mod google_tests {
    use swissknife_search_sdk::google::{
        GoogleSearchClient, SearchParams, SearchType,
    };

    #[test]
    fn test_google_client_creation() {
        let client = GoogleSearchClient::new("api-key", "cx-id");
        assert!(true);
    }

    #[test]
    fn test_search_params_basic() {
        let params = SearchParams::new("rust programming language");

        assert_eq!(params.query, "rust programming language");
    }

    #[test]
    fn test_search_params_builder() {
        let params = SearchParams::new("climate change")
            .with_num(10)
            .with_start(1)
            .with_search_type(SearchType::Web)
            .with_language("en")
            .with_country("us");

        assert_eq!(params.query, "climate change");
        assert_eq!(params.num, Some(10));
        assert_eq!(params.start, Some(1));
    }

    #[test]
    fn test_search_type_variants() {
        let web = SearchType::Web;
        let image = SearchType::Image;

        assert!(matches!(web, SearchType::Web));
        assert!(matches!(image, SearchType::Image));
    }
}

mod error_tests {
    use swissknife_search_sdk::Error;

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
    fn test_invalid_request_error() {
        let error = Error::InvalidRequest("Query too long".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Query too long"));
    }

    #[test]
    fn test_parse_error() {
        let error = Error::Parse("Empty query string".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Empty query"));
    }
}
