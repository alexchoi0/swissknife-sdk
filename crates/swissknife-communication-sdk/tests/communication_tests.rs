#[cfg(feature = "twilio")]
mod twilio_tests {
    use swissknife_communication_sdk::twilio::{
        TwilioClient, SendSmsRequest, MakeCallRequest,
    };

    #[test]
    fn test_twilio_client_creation() {
        let client = TwilioClient::new("account-sid", "auth-token");
        assert!(true);
    }

    #[test]
    fn test_send_sms_request() {
        let request = SendSmsRequest {
            to: "+1234567890".to_string(),
            from: "+0987654321".to_string(),
            body: "Hello from Twilio!".to_string(),
            messaging_service_sid: None,
            media_url: None,
            status_callback: None,
        };

        assert_eq!(request.to, "+1234567890");
        assert_eq!(request.from, "+0987654321");
        assert_eq!(request.body, "Hello from Twilio!");
    }

    #[test]
    fn test_send_sms_with_media() {
        let request = SendSmsRequest {
            to: "+1234567890".to_string(),
            from: "+0987654321".to_string(),
            body: "Check out this image!".to_string(),
            messaging_service_sid: None,
            media_url: Some(vec!["https://example.com/image.jpg".to_string()]),
            status_callback: Some("https://webhook.example.com/status".to_string()),
        };

        assert!(request.media_url.is_some());
        assert!(request.status_callback.is_some());
    }

    #[test]
    fn test_make_call_request() {
        let request = MakeCallRequest {
            to: "+1234567890".to_string(),
            from: "+0987654321".to_string(),
            url: Some("https://example.com/twiml".to_string()),
            twiml: None,
            timeout: Some(30),
            record: Some(false),
        };

        assert_eq!(request.to, "+1234567890");
        assert!(request.url.is_some());
        assert_eq!(request.timeout, Some(30));
    }
}

#[cfg(feature = "sendgrid")]
mod sendgrid_tests {
    use swissknife_communication_sdk::sendgrid::{
        SendGridClient, SendEmailRequest, EmailAddress, Content, Attachment,
    };

    #[test]
    fn test_sendgrid_client_creation() {
        let client = SendGridClient::new("sg-api-key");
        assert!(true);
    }

    #[test]
    fn test_email_address() {
        let addr = EmailAddress {
            email: "test@example.com".to_string(),
            name: Some("Test User".to_string()),
        };

        assert_eq!(addr.email, "test@example.com");
        assert_eq!(addr.name, Some("Test User".to_string()));
    }

    #[test]
    fn test_send_email_request() {
        let request = SendEmailRequest {
            from: EmailAddress {
                email: "sender@example.com".to_string(),
                name: Some("Sender".to_string()),
            },
            to: vec![EmailAddress {
                email: "recipient@example.com".to_string(),
                name: None,
            }],
            subject: "Test Email".to_string(),
            content: vec![Content {
                content_type: "text/plain".to_string(),
                value: "Hello, World!".to_string(),
            }],
            cc: None,
            bcc: None,
            reply_to: None,
            attachments: None,
            template_id: None,
            dynamic_template_data: None,
        };

        assert_eq!(request.subject, "Test Email");
        assert_eq!(request.to.len(), 1);
        assert_eq!(request.content.len(), 1);
    }

    #[test]
    fn test_attachment() {
        let attachment = Attachment {
            content: "base64encodedcontent".to_string(),
            filename: "document.pdf".to_string(),
            content_type: Some("application/pdf".to_string()),
            disposition: Some("attachment".to_string()),
            content_id: None,
        };

        assert_eq!(attachment.filename, "document.pdf");
        assert_eq!(attachment.content_type, Some("application/pdf".to_string()));
    }
}

#[cfg(feature = "slack")]
mod slack_tests {
    use swissknife_communication_sdk::slack::{
        SlackClient, PostMessageRequest, Block, BlockType,
    };

    #[test]
    fn test_slack_client_creation() {
        let client = SlackClient::new("xoxb-slack-token");
        assert!(true);
    }

    #[test]
    fn test_post_message_request() {
        let request = PostMessageRequest {
            channel: "#general".to_string(),
            text: Some("Hello Slack!".to_string()),
            blocks: None,
            thread_ts: None,
            reply_broadcast: None,
            unfurl_links: None,
            unfurl_media: None,
        };

        assert_eq!(request.channel, "#general");
        assert_eq!(request.text, Some("Hello Slack!".to_string()));
    }

    #[test]
    fn test_post_message_with_blocks() {
        let blocks = vec![
            Block {
                block_type: BlockType::Section,
                text: Some(serde_json::json!({
                    "type": "mrkdwn",
                    "text": "*Hello!*"
                })),
                block_id: None,
                accessory: None,
                fields: None,
                elements: None,
            },
        ];

        let request = PostMessageRequest {
            channel: "#general".to_string(),
            text: None,
            blocks: Some(blocks),
            thread_ts: Some("1234567890.123456".to_string()),
            reply_broadcast: Some(true),
            unfurl_links: Some(false),
            unfurl_media: Some(false),
        };

        assert!(request.blocks.is_some());
        assert!(request.thread_ts.is_some());
    }
}

#[cfg(feature = "discord")]
mod discord_tests {
    use swissknife_communication_sdk::discord::{
        DiscordClient, SendMessageRequest, Embed, EmbedField,
    };

    #[test]
    fn test_discord_client_creation() {
        let client = DiscordClient::new("discord-bot-token");
        assert!(true);
    }

    #[test]
    fn test_send_message_request() {
        let request = SendMessageRequest {
            content: Some("Hello Discord!".to_string()),
            embeds: None,
            tts: None,
            message_reference: None,
        };

        assert_eq!(request.content, Some("Hello Discord!".to_string()));
    }

    #[test]
    fn test_embed() {
        let embed = Embed {
            title: Some("Test Embed".to_string()),
            description: Some("This is a test embed".to_string()),
            url: Some("https://example.com".to_string()),
            color: Some(0x00FF00),
            fields: Some(vec![
                EmbedField {
                    name: "Field 1".to_string(),
                    value: "Value 1".to_string(),
                    inline: Some(true),
                },
            ]),
            footer: None,
            thumbnail: None,
            image: None,
            timestamp: None,
        };

        assert_eq!(embed.title, Some("Test Embed".to_string()));
        assert_eq!(embed.color, Some(0x00FF00));
        assert!(embed.fields.is_some());
    }
}

#[cfg(feature = "telegram")]
mod telegram_tests {
    use swissknife_communication_sdk::telegram::{
        TelegramClient, SendMessageRequest, ParseMode,
    };

    #[test]
    fn test_telegram_client_creation() {
        let client = TelegramClient::new("bot-token");
        assert!(true);
    }

    #[test]
    fn test_send_message_request() {
        let request = SendMessageRequest {
            chat_id: "123456789".to_string(),
            text: "Hello Telegram!".to_string(),
            parse_mode: Some(ParseMode::Markdown),
            disable_web_page_preview: Some(true),
            disable_notification: Some(false),
            reply_to_message_id: None,
        };

        assert_eq!(request.chat_id, "123456789");
        assert_eq!(request.text, "Hello Telegram!");
        assert!(matches!(request.parse_mode, Some(ParseMode::Markdown)));
    }

    #[test]
    fn test_parse_mode_variants() {
        let markdown = ParseMode::Markdown;
        let markdown_v2 = ParseMode::MarkdownV2;
        let html = ParseMode::Html;

        assert!(matches!(markdown, ParseMode::Markdown));
        assert!(matches!(markdown_v2, ParseMode::MarkdownV2));
        assert!(matches!(html, ParseMode::Html));
    }
}

#[cfg(feature = "resend")]
mod resend_tests {
    use swissknife_communication_sdk::resend::{
        ResendClient, SendEmailRequest,
    };

    #[test]
    fn test_resend_client_creation() {
        let client = ResendClient::new("re_api-key");
        assert!(true);
    }

    #[test]
    fn test_send_email_request() {
        let request = SendEmailRequest {
            from: "sender@example.com".to_string(),
            to: vec!["recipient@example.com".to_string()],
            subject: "Test Email".to_string(),
            html: Some("<h1>Hello!</h1>".to_string()),
            text: None,
            cc: None,
            bcc: None,
            reply_to: None,
            headers: None,
            attachments: None,
            tags: None,
        };

        assert_eq!(request.from, "sender@example.com");
        assert_eq!(request.to.len(), 1);
        assert!(request.html.is_some());
    }
}

#[cfg(feature = "teams")]
mod teams_tests {
    use swissknife_communication_sdk::teams::{
        TeamsClient, SendMessageRequest, AdaptiveCard,
    };

    #[test]
    fn test_teams_client_creation() {
        let client = TeamsClient::new("teams-access-token");
        assert!(true);
    }

    #[test]
    fn test_send_message_request() {
        let request = SendMessageRequest {
            content: "Hello Teams!".to_string(),
            content_type: Some("text".to_string()),
            attachments: None,
        };

        assert_eq!(request.content, "Hello Teams!");
    }
}

mod error_tests {
    use swissknife_communication_sdk::Error;

    #[test]
    fn test_error_display() {
        let api_error = Error::Api {
            message: "Invalid phone number".to_string(),
            code: Some("21211".to_string()),
        };

        let error_string = format!("{}", api_error);
        assert!(error_string.contains("Invalid phone number"));
    }

    #[test]
    fn test_send_failed_error() {
        let error = Error::SendFailed("Message delivery failed".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Message delivery failed"));
    }

    #[test]
    fn test_invalid_recipient_error() {
        let error = Error::InvalidRecipient("not-an-email".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("not-an-email"));
    }

    #[test]
    fn test_rate_limit_error() {
        let error = Error::RateLimit("Too many requests".to_string());
        let error_string = format!("{}", error);
        assert!(error_string.contains("Too many requests"));
    }
}
