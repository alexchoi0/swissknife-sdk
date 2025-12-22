mod error;

#[cfg(feature = "twilio")]
pub mod twilio;

#[cfg(feature = "sendgrid")]
pub mod sendgrid;

#[cfg(feature = "fcm")]
pub mod fcm;

#[cfg(feature = "apns")]
pub mod apns;

#[cfg(feature = "slack")]
pub mod slack;

#[cfg(feature = "discord")]
pub mod discord;

#[cfg(feature = "telegram")]
pub mod telegram;

pub use error::{Error, Result};

#[cfg(feature = "sms")]
pub mod sms {
    use async_trait::async_trait;

    #[async_trait]
    pub trait SmsSender: Send + Sync {
        async fn send_sms(
            &self,
            from: &str,
            to: &str,
            body: &str,
        ) -> crate::Result<SmsResponse>;
    }

    #[derive(Debug, Clone)]
    pub struct SmsResponse {
        pub message_id: String,
        pub status: String,
    }
}

#[cfg(feature = "voice")]
pub mod voice {
    use async_trait::async_trait;

    #[async_trait]
    pub trait VoiceCaller: Send + Sync {
        async fn make_call(
            &self,
            from: &str,
            to: &str,
            twiml_url: &str,
        ) -> crate::Result<CallResponse>;
    }

    #[derive(Debug, Clone)]
    pub struct CallResponse {
        pub call_id: String,
        pub status: String,
    }
}

#[cfg(feature = "whatsapp")]
pub mod whatsapp {
    use async_trait::async_trait;

    #[async_trait]
    pub trait WhatsAppSender: Send + Sync {
        async fn send_whatsapp(
            &self,
            from: &str,
            to: &str,
            body: &str,
        ) -> crate::Result<WhatsAppResponse>;
    }

    #[derive(Debug, Clone)]
    pub struct WhatsAppResponse {
        pub message_id: String,
        pub status: String,
    }
}

#[cfg(feature = "email")]
pub mod email {
    use async_trait::async_trait;

    #[derive(Debug, Clone)]
    pub struct EmailAddress {
        pub email: String,
        pub name: Option<String>,
    }

    impl EmailAddress {
        pub fn new(email: impl Into<String>) -> Self {
            Self {
                email: email.into(),
                name: None,
            }
        }

        pub fn with_name(email: impl Into<String>, name: impl Into<String>) -> Self {
            Self {
                email: email.into(),
                name: Some(name.into()),
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Email {
        pub from: EmailAddress,
        pub to: Vec<EmailAddress>,
        pub subject: String,
        pub text: Option<String>,
        pub html: Option<String>,
    }

    impl Email {
        pub fn new(from: EmailAddress, to: EmailAddress, subject: impl Into<String>) -> Self {
            Self {
                from,
                to: vec![to],
                subject: subject.into(),
                text: None,
                html: None,
            }
        }

        pub fn text(mut self, text: impl Into<String>) -> Self {
            self.text = Some(text.into());
            self
        }

        pub fn html(mut self, html: impl Into<String>) -> Self {
            self.html = Some(html.into());
            self
        }

        pub fn add_to(mut self, to: EmailAddress) -> Self {
            self.to.push(to);
            self
        }
    }

    #[derive(Debug, Clone)]
    pub struct EmailResponse {
        pub message_id: Option<String>,
        pub status: String,
    }

    #[async_trait]
    pub trait EmailSender: Send + Sync {
        async fn send_email(&self, email: &Email) -> crate::Result<EmailResponse>;
    }
}

#[cfg(feature = "push")]
pub mod push {
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct PushNotification {
        pub title: String,
        pub body: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub image: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub data: Option<HashMap<String, String>>,
    }

    impl PushNotification {
        pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
            Self {
                title: title.into(),
                body: body.into(),
                image: None,
                data: None,
            }
        }

        pub fn image(mut self, url: impl Into<String>) -> Self {
            self.image = Some(url.into());
            self
        }

        pub fn data(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
            self.data
                .get_or_insert_with(HashMap::new)
                .insert(key.into(), value.into());
            self
        }
    }

    #[derive(Debug, Clone)]
    pub struct PushResponse {
        pub message_id: Option<String>,
        pub success_count: u32,
        pub failure_count: u32,
    }

    #[async_trait]
    pub trait PushSender: Send + Sync {
        async fn send_to_token(
            &self,
            token: &str,
            notification: &PushNotification,
        ) -> crate::Result<PushResponse>;

        async fn send_to_topic(
            &self,
            topic: &str,
            notification: &PushNotification,
        ) -> crate::Result<PushResponse>;
    }
}

#[cfg(feature = "chat")]
pub mod chat {
    use async_trait::async_trait;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ChatMessage {
        pub text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub username: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub icon_url: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub icon_emoji: Option<String>,
    }

    impl ChatMessage {
        pub fn new(text: impl Into<String>) -> Self {
            Self {
                text: text.into(),
                username: None,
                icon_url: None,
                icon_emoji: None,
            }
        }

        pub fn username(mut self, username: impl Into<String>) -> Self {
            self.username = Some(username.into());
            self
        }

        pub fn icon_url(mut self, url: impl Into<String>) -> Self {
            self.icon_url = Some(url.into());
            self
        }

        pub fn icon_emoji(mut self, emoji: impl Into<String>) -> Self {
            self.icon_emoji = Some(emoji.into());
            self
        }
    }

    #[derive(Debug, Clone)]
    pub struct ChatResponse {
        pub message_id: Option<String>,
        pub channel: Option<String>,
        pub timestamp: Option<String>,
    }

    #[async_trait]
    pub trait ChatSender: Send + Sync {
        async fn send_message(
            &self,
            channel: &str,
            message: &ChatMessage,
        ) -> crate::Result<ChatResponse>;
    }
}
