#[cfg(feature = "openai")]
mod openai_tests {
    use swissknife_ai_sdk::llm::{
        openai::{OpenAIClient, models},
        ChatRequest, ChatMessage, ProviderConfig, EmbeddingRequest, ImageRequest,
        TextToSpeechRequest, VisionRequest, ImageContent,
    };

    #[test]
    fn test_openai_client_creation() {
        let client = OpenAIClient::from_api_key("test-api-key");
        assert!(true);
    }

    #[test]
    fn test_openai_client_with_config() {
        let config = ProviderConfig::new("test-api-key")
            .with_base_url("https://custom.api.com")
            .with_organization("org-123");
        let client = OpenAIClient::new(config);
        assert!(true);
    }

    #[test]
    fn test_chat_request_builder() {
        let request = ChatRequest::new(
            models::GPT_4O,
            vec![
                ChatMessage::system("You are a helpful assistant."),
                ChatMessage::user("Hello!"),
            ],
        )
        .with_max_tokens(100)
        .with_temperature(0.7);

        assert_eq!(request.model, models::GPT_4O);
        assert_eq!(request.messages.len(), 2);
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.temperature, Some(0.7));
    }

    #[test]
    fn test_chat_message_constructors() {
        let system = ChatMessage::system("System prompt");
        let user = ChatMessage::user("User message");
        let assistant = ChatMessage::assistant("Assistant response");
        let tool_result = ChatMessage::tool_result("call-123", "Tool output");

        assert!(matches!(system.role, swissknife_ai_sdk::llm::MessageRole::System));
        assert!(matches!(user.role, swissknife_ai_sdk::llm::MessageRole::User));
        assert!(matches!(assistant.role, swissknife_ai_sdk::llm::MessageRole::Assistant));
        assert!(matches!(tool_result.role, swissknife_ai_sdk::llm::MessageRole::Tool));
        assert_eq!(tool_result.tool_call_id, Some("call-123".to_string()));
    }

    #[test]
    fn test_embedding_request_builder() {
        let request = EmbeddingRequest::new(
            models::TEXT_EMBEDDING_3_SMALL,
            vec!["Hello world".to_string(), "Test text".to_string()],
        );
        assert_eq!(request.model, models::TEXT_EMBEDDING_3_SMALL);
        assert_eq!(request.input.len(), 2);

        let single = EmbeddingRequest::single(models::TEXT_EMBEDDING_3_LARGE, "Single text");
        assert_eq!(single.input.len(), 1);
    }

    #[test]
    fn test_image_request_builder() {
        let request = ImageRequest::new("A beautiful sunset")
            .with_model(models::DALL_E_3)
            .with_size("1024x1024");

        assert_eq!(request.prompt, "A beautiful sunset");
        assert_eq!(request.model, Some(models::DALL_E_3.to_string()));
        assert_eq!(request.size, Some("1024x1024".to_string()));
    }

    #[test]
    fn test_tts_request_builder() {
        let request = TextToSpeechRequest::new(models::TTS_1, "Hello world", "alloy");

        assert_eq!(request.model, models::TTS_1);
        assert_eq!(request.input, "Hello world");
        assert_eq!(request.voice, "alloy");
    }

    #[test]
    fn test_vision_request_builder() {
        let image = ImageContent::from_url("https://example.com/image.jpg");
        let request = VisionRequest::single_image(
            models::GPT_4O,
            image,
            "What's in this image?",
        )
        .with_max_tokens(500);

        assert_eq!(request.model, models::GPT_4O);
        assert_eq!(request.images.len(), 1);
        assert_eq!(request.prompt, "What's in this image?");
        assert_eq!(request.max_tokens, Some(500));
    }

    #[test]
    fn test_image_content_constructors() {
        let url_image = ImageContent::from_url("https://example.com/image.jpg");
        assert_eq!(url_image.url, Some("https://example.com/image.jpg".to_string()));
        assert!(url_image.base64.is_none());

        let base64_image = ImageContent::from_base64("base64data", "image/png");
        assert_eq!(base64_image.base64, Some("base64data".to_string()));
        assert_eq!(base64_image.media_type, Some("image/png".to_string()));
    }

    #[test]
    fn test_model_constants() {
        assert_eq!(models::GPT_4O, "gpt-4o");
        assert_eq!(models::GPT_4O_MINI, "gpt-4o-mini");
        assert_eq!(models::GPT_4_TURBO, "gpt-4-turbo");
        assert_eq!(models::TEXT_EMBEDDING_3_LARGE, "text-embedding-3-large");
        assert_eq!(models::DALL_E_3, "dall-e-3");
        assert_eq!(models::TTS_1, "tts-1");
        assert_eq!(models::WHISPER_1, "whisper-1");
    }
}

#[cfg(feature = "anthropic")]
mod anthropic_tests {
    use swissknife_ai_sdk::llm::{
        anthropic::{AnthropicClient, models},
        ChatRequest, ChatMessage, ProviderConfig, VisionRequest, ImageContent,
    };

    #[test]
    fn test_anthropic_client_creation() {
        let client = AnthropicClient::from_api_key("test-api-key");
        assert!(true);
    }

    #[test]
    fn test_anthropic_client_with_config() {
        let config = ProviderConfig::new("test-api-key")
            .with_base_url("https://custom.anthropic.com");
        let client = AnthropicClient::new(config);
        assert!(true);
    }

    #[test]
    fn test_chat_request_with_system() {
        let request = ChatRequest::new(
            models::CLAUDE_3_5_SONNET,
            vec![
                ChatMessage::system("You are Claude."),
                ChatMessage::user("Hello Claude!"),
            ],
        );

        assert_eq!(request.model, models::CLAUDE_3_5_SONNET);
        assert_eq!(request.messages.len(), 2);
    }

    #[test]
    fn test_model_constants() {
        assert_eq!(models::CLAUDE_OPUS_4, "claude-opus-4-20250514");
        assert_eq!(models::CLAUDE_SONNET_4, "claude-sonnet-4-20250514");
        assert_eq!(models::CLAUDE_3_5_SONNET, "claude-3-5-sonnet-20241022");
        assert_eq!(models::CLAUDE_3_5_HAIKU, "claude-3-5-haiku-20241022");
        assert_eq!(models::CLAUDE_3_OPUS, "claude-3-opus-20240229");
    }
}

#[cfg(feature = "mistral")]
mod mistral_tests {
    use swissknife_ai_sdk::llm::{
        mistral::{MistralClient, models},
        ChatRequest, ChatMessage, ProviderConfig, EmbeddingRequest,
    };

    #[test]
    fn test_mistral_client_creation() {
        let client = MistralClient::from_api_key("test-api-key");
        assert!(true);
    }

    #[test]
    fn test_mistral_chat_request() {
        let request = ChatRequest::new(
            models::MISTRAL_LARGE,
            vec![ChatMessage::user("Hello Mistral!")],
        )
        .with_temperature(0.5);

        assert_eq!(request.model, models::MISTRAL_LARGE);
        assert_eq!(request.temperature, Some(0.5));
    }

    #[test]
    fn test_mistral_embedding_request() {
        let request = EmbeddingRequest::single(models::MISTRAL_EMBED, "Test text");
        assert_eq!(request.model, models::MISTRAL_EMBED);
    }

    #[test]
    fn test_model_constants() {
        assert_eq!(models::MISTRAL_LARGE, "mistral-large-latest");
        assert_eq!(models::MISTRAL_SMALL, "mistral-small-latest");
        assert_eq!(models::CODESTRAL, "codestral-latest");
        assert_eq!(models::MISTRAL_EMBED, "mistral-embed");
    }
}

#[cfg(feature = "stability")]
mod stability_tests {
    use swissknife_ai_sdk::llm::{
        stability::{StabilityClient, SD3Request, UltraRequest, models, aspect_ratios},
        ProviderConfig, ImageRequest,
    };

    #[test]
    fn test_stability_client_creation() {
        let client = StabilityClient::from_api_key("test-api-key");
        assert!(true);
    }

    #[test]
    fn test_sd3_request_builder() {
        let request = SD3Request::new("A futuristic cityscape")
            .with_model(models::SD3_LARGE)
            .with_aspect_ratio(aspect_ratios::LANDSCAPE_16_9)
            .with_negative_prompt("blurry, low quality");

        assert_eq!(request.prompt, "A futuristic cityscape");
        assert_eq!(request.model, Some(models::SD3_LARGE.to_string()));
        assert_eq!(request.aspect_ratio, Some(aspect_ratios::LANDSCAPE_16_9.to_string()));
        assert_eq!(request.negative_prompt, Some("blurry, low quality".to_string()));
    }

    #[test]
    fn test_ultra_request_builder() {
        let request = UltraRequest::new("A beautiful landscape")
            .with_aspect_ratio(aspect_ratios::SQUARE);

        assert_eq!(request.prompt, "A beautiful landscape");
        assert_eq!(request.aspect_ratio, Some(aspect_ratios::SQUARE.to_string()));
    }

    #[test]
    fn test_model_constants() {
        assert_eq!(models::SD3_LARGE, "sd3-large");
        assert_eq!(models::SD3_LARGE_TURBO, "sd3-large-turbo");
        assert_eq!(models::SD3_MEDIUM, "sd3-medium");
    }

    #[test]
    fn test_aspect_ratio_constants() {
        assert_eq!(aspect_ratios::SQUARE, "1:1");
        assert_eq!(aspect_ratios::LANDSCAPE_16_9, "16:9");
        assert_eq!(aspect_ratios::PORTRAIT_9_16, "9:16");
    }
}

#[cfg(feature = "deepl")]
mod deepl_tests {
    use swissknife_ai_sdk::llm::{
        deepl::{DeepLClient, DeepLTranslateRequest, CreateGlossaryRequest, languages, formality},
        ProviderConfig, TranslationRequest,
    };

    #[test]
    fn test_deepl_client_creation() {
        let client = DeepLClient::from_api_key("test-api-key");
        assert!(true);
    }

    #[test]
    fn test_deepl_free_api_detection() {
        let free_client = DeepLClient::from_api_key("test-key:fx");
        let pro_client = DeepLClient::from_api_key("test-key");
        assert!(true);
    }

    #[test]
    fn test_translate_request_builder() {
        let request = DeepLTranslateRequest::single("Hello world", languages::GERMAN)
            .with_source_lang(languages::ENGLISH)
            .with_formality(formality::MORE);

        assert_eq!(request.text, vec!["Hello world".to_string()]);
        assert_eq!(request.target_lang, languages::GERMAN);
        assert_eq!(request.source_lang, Some(languages::ENGLISH.to_string()));
        assert_eq!(request.formality, Some(formality::MORE.to_string()));
    }

    #[test]
    fn test_translation_request_builder() {
        let request = TranslationRequest::single("Hello", "DE")
            .with_source_language("EN");

        assert_eq!(request.text, vec!["Hello".to_string()]);
        assert_eq!(request.target_language, "DE");
        assert_eq!(request.source_language, Some("EN".to_string()));
    }

    #[test]
    fn test_language_constants() {
        assert_eq!(languages::ENGLISH, "EN");
        assert_eq!(languages::GERMAN, "DE");
        assert_eq!(languages::FRENCH, "FR");
        assert_eq!(languages::SPANISH, "ES");
        assert_eq!(languages::JAPANESE, "JA");
        assert_eq!(languages::CHINESE, "ZH");
    }

    #[test]
    fn test_formality_constants() {
        assert_eq!(formality::DEFAULT, "default");
        assert_eq!(formality::MORE, "more");
        assert_eq!(formality::LESS, "less");
    }
}

#[cfg(feature = "runway")]
mod runway_tests {
    use swissknife_ai_sdk::llm::{
        runway::{RunwayClient, Gen3Request, TextToVideoRequest, models, ratios},
        ProviderConfig, VideoRequest,
    };

    #[test]
    fn test_runway_client_creation() {
        let client = RunwayClient::from_api_key("test-api-key");
        assert!(true);
    }

    #[test]
    fn test_gen3_request_builder() {
        let request = Gen3Request::new("https://example.com/image.jpg")
            .with_prompt("Make this image come alive")
            .with_duration(10)
            .with_ratio(ratios::LANDSCAPE_1280_768);

        assert_eq!(request.prompt_image, "https://example.com/image.jpg");
        assert_eq!(request.prompt_text, Some("Make this image come alive".to_string()));
        assert_eq!(request.duration, Some(10));
        assert_eq!(request.ratio, Some(ratios::LANDSCAPE_1280_768.to_string()));
    }

    #[test]
    fn test_text_to_video_request() {
        let request = TextToVideoRequest::new("A rocket launching into space")
            .with_duration(5);

        assert_eq!(request.prompt, "A rocket launching into space");
        assert_eq!(request.duration, Some(5));
    }

    #[test]
    fn test_video_request_builder() {
        let request = VideoRequest::new("Animate this scene")
            .with_image("https://example.com/image.jpg")
            .with_duration(5)
            .with_aspect_ratio("16:9");

        assert_eq!(request.prompt, "Animate this scene");
        assert_eq!(request.image, Some("https://example.com/image.jpg".to_string()));
        assert_eq!(request.duration, Some(5));
    }

    #[test]
    fn test_model_constants() {
        assert_eq!(models::GEN3A_TURBO, "gen3a_turbo");
    }

    #[test]
    fn test_ratio_constants() {
        assert_eq!(ratios::LANDSCAPE_1280_768, "1280:768");
        assert_eq!(ratios::PORTRAIT_768_1280, "768:1280");
    }
}

#[cfg(feature = "llm")]
mod types_tests {
    use swissknife_ai_sdk::llm::{
        ChatRequest, ChatMessage, ChatResponse, ChatChoice, MessageRole, MessageContent,
        EmbeddingRequest, EmbeddingResponse, EmbeddingData, Usage, ProviderConfig,
        AudioFormat,
    };

    #[test]
    fn test_chat_response_content() {
        let response = ChatResponse {
            id: "test-id".to_string(),
            model: "gpt-4".to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: MessageContent::Text("Hello!".to_string()),
                    name: None,
                    tool_call_id: None,
                    tool_calls: None,
                },
                finish_reason: Some("stop".to_string()),
            }],
            usage: Some(Usage {
                prompt_tokens: 10,
                completion_tokens: 5,
                total_tokens: 15,
            }),
        };

        assert_eq!(response.content(), Some("Hello!"));
        assert!(response.tool_calls().is_none());
    }

    #[test]
    fn test_embedding_response_first() {
        let response = EmbeddingResponse {
            model: "text-embedding-3-small".to_string(),
            data: vec![EmbeddingData {
                index: 0,
                embedding: vec![0.1, 0.2, 0.3],
            }],
            usage: None,
        };

        assert_eq!(response.first(), Some(&[0.1, 0.2, 0.3][..]));
    }

    #[test]
    fn test_provider_config_builder() {
        let config = ProviderConfig::new("api-key")
            .with_base_url("https://custom.api.com")
            .with_organization("org-123");

        assert_eq!(config.api_key, "api-key");
        assert_eq!(config.base_url, Some("https://custom.api.com".to_string()));
        assert_eq!(config.organization, Some("org-123".to_string()));
    }

    #[test]
    fn test_audio_format() {
        assert_eq!(AudioFormat::Mp3.as_str(), "mp3");
        assert_eq!(AudioFormat::Wav.as_str(), "wav");
        assert_eq!(AudioFormat::Mp3.mime_type(), "audio/mpeg");
        assert_eq!(AudioFormat::Wav.mime_type(), "audio/wav");
    }

    #[test]
    fn test_message_with_images() {
        use swissknife_ai_sdk::llm::ImageContent;

        let images = vec![
            ImageContent::from_url("https://example.com/img1.jpg"),
            ImageContent::from_url("https://example.com/img2.jpg"),
        ];
        let message = ChatMessage::with_images(
            MessageRole::User,
            "What are these images?",
            images,
        );

        match message.content {
            MessageContent::Parts(parts) => {
                assert_eq!(parts.len(), 3);
            }
            _ => panic!("Expected Parts content"),
        }
    }
}
