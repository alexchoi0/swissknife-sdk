#[cfg(feature = "slack")]
mod slack {
    use swissknife_communication_sdk::slack::{SlackClient, SlackWebhookClient};

    #[test]
    fn test_slack_client_new() {
        let client = SlackClient::new("xoxb-test-token");
        assert!(client.is_ok());
    }

    #[test]
    fn test_slack_client_empty_token() {
        let client = SlackClient::new("");
        assert!(client.is_err());
    }

    #[test]
    fn test_slack_webhook_client_new() {
        let client = SlackWebhookClient::new("https://hooks.slack.com/services/xxx");
        assert!(client.is_ok());
    }

    #[test]
    fn test_slack_webhook_client_empty_url() {
        let client = SlackWebhookClient::new("");
        assert!(client.is_err());
    }

    #[test]
    fn test_slack_client_from_webhook() {
        let client = SlackClient::from_webhook("https://hooks.slack.com/services/test");
        assert!(client.is_ok());
    }
}

#[cfg(feature = "discord")]
mod discord {
    use swissknife_communication_sdk::discord::{DiscordBotClient, DiscordWebhookClient};

    #[test]
    fn test_discord_bot_client_new() {
        let client = DiscordBotClient::new("discord-bot-token");
        assert!(client.is_ok());
    }

    #[test]
    fn test_discord_bot_client_empty_token() {
        let client = DiscordBotClient::new("");
        assert!(client.is_err());
    }

    #[test]
    fn test_discord_webhook_client_new() {
        let client = DiscordWebhookClient::new("https://discord.com/api/webhooks/xxx");
        assert!(client.is_ok());
    }

    #[test]
    fn test_discord_webhook_client_empty_url() {
        let client = DiscordWebhookClient::new("");
        assert!(client.is_err());
    }
}

#[cfg(feature = "twilio")]
mod twilio {
    use swissknife_communication_sdk::twilio::TwilioClient;

    #[test]
    fn test_twilio_client_new() {
        let client = TwilioClient::new("account-sid", "auth-token");
        assert!(client.is_ok());
    }

    #[test]
    fn test_twilio_client_empty_sid() {
        let client = TwilioClient::new("", "auth-token");
        assert!(client.is_err());
    }

    #[test]
    fn test_twilio_client_empty_token() {
        let client = TwilioClient::new("account-sid", "");
        assert!(client.is_err());
    }

    #[test]
    fn test_twilio_client_both_empty() {
        let client = TwilioClient::new("", "");
        assert!(client.is_err());
    }
}

#[cfg(feature = "sendgrid")]
mod sendgrid {
    use swissknife_communication_sdk::sendgrid::SendGridClient;

    #[test]
    fn test_sendgrid_client_new() {
        let client = SendGridClient::new("sg-api-key");
        assert!(client.is_ok());
    }

    #[test]
    fn test_sendgrid_client_empty_key() {
        let client = SendGridClient::new("");
        assert!(client.is_err());
    }
}

#[cfg(feature = "telegram")]
mod telegram {
    use swissknife_communication_sdk::telegram::TelegramClient;

    #[test]
    fn test_telegram_client_new() {
        let client = TelegramClient::new("bot-token");
        assert!(client.is_ok());
    }

    #[test]
    fn test_telegram_client_empty_token() {
        let client = TelegramClient::new("");
        assert!(client.is_err());
    }
}

#[cfg(feature = "resend")]
mod resend {
    use swissknife_communication_sdk::resend::ResendClient;

    #[test]
    fn test_resend_client_new() {
        let client = ResendClient::new("re_api_key");
        assert!(client.is_ok());
    }

    #[test]
    fn test_resend_client_empty_key() {
        let client = ResendClient::new("");
        assert!(client.is_err());
    }
}

#[cfg(feature = "teams")]
mod teams {
    use swissknife_communication_sdk::teams::TeamsClient;

    #[test]
    fn test_teams_client_new() {
        let client = TeamsClient::new("teams-access-token");
        assert!(client.is_ok());
    }

    #[test]
    fn test_teams_client_empty_token() {
        let client = TeamsClient::new("");
        assert!(client.is_err());
    }
}
