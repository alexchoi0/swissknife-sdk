#[cfg(feature = "tavily")]
mod tavily {
    use swissknife_search_sdk::tavily::TavilyClient;

    #[test]
    fn test_tavily_client_new() {
        let client = TavilyClient::new("test-api-key");
        let _ = client;
    }

    #[test]
    fn test_tavily_client_with_empty_key() {
        let client = TavilyClient::new("");
        let _ = client;
    }
}

#[cfg(feature = "exa")]
mod exa {
    use swissknife_search_sdk::exa::ExaClient;

    #[test]
    fn test_exa_client_new() {
        let client = ExaClient::new("test-api-key");
        let _ = client;
    }
}

#[cfg(feature = "duckduckgo")]
mod duckduckgo {
    use swissknife_search_sdk::duckduckgo::DuckDuckGoClient;

    #[test]
    fn test_duckduckgo_client_new() {
        let client = DuckDuckGoClient::new();
        let _ = client;
    }
}

#[cfg(feature = "wikipedia")]
mod wikipedia {
    use swissknife_search_sdk::wikipedia::WikipediaClient;

    #[test]
    fn test_wikipedia_client_new() {
        let client = WikipediaClient::new();
        let _ = client;
    }
}

#[cfg(feature = "google")]
mod google {
    use swissknife_search_sdk::google::GoogleSearchClient;

    #[test]
    fn test_google_client_new() {
        let client = GoogleSearchClient::new("api-key", "cx-id");
        let _ = client;
    }
}

#[cfg(feature = "serper")]
mod serper {
    use swissknife_search_sdk::serper::SerperClient;

    #[test]
    fn test_serper_client_new() {
        let client = SerperClient::new("serper-api-key");
        let _ = client;
    }
}

#[cfg(feature = "perplexity")]
mod perplexity {
    use swissknife_search_sdk::perplexity::PerplexityClient;

    #[test]
    fn test_perplexity_client_new() {
        let client = PerplexityClient::new("pplx-api-key");
        let _ = client;
    }
}
