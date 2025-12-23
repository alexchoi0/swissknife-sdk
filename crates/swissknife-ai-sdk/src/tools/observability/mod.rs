use crate::error::Result;
use crate::tool::{get_object_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_observability_sdk::{AnalyticsProvider, Event, ErrorTrackingProvider, ErrorEvent, ErrorLevel};

pub struct TrackEventTool;

impl Default for TrackEventTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for TrackEventTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "analytics_track_event",
            "Track Analytics Event",
            "Track an analytics event (PostHog, Mixpanel, Amplitude)",
            "observability",
        )
        .with_param("api_key", ParameterSchema::string("Analytics API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: posthog, mixpanel, amplitude").required())
        .with_param("event_name", ParameterSchema::string("Event name").required())
        .with_param("distinct_id", ParameterSchema::string("User/session identifier"))
        .with_param("properties", ParameterSchema::json("Event properties"))
        .with_output("success", OutputSchema::boolean("Whether tracking succeeded"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let event_name = get_required_string_param(&params, "event_name")?;
        let distinct_id = get_string_param(&params, "distinct_id");
        let properties = get_object_param(&params, "properties")
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        let event = Event {
            name: event_name,
            timestamp: None,
            distinct_id,
            properties,
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "posthog")]
            "posthog" => {
                use swissknife_observability_sdk::posthog::PostHogClient;
                let client = PostHogClient::new(&api_key);
                client.track(&event).await
            }
            #[cfg(feature = "mixpanel")]
            "mixpanel" => {
                use swissknife_observability_sdk::mixpanel::MixpanelClient;
                let client = MixpanelClient::new(&api_key);
                client.track(&event).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported analytics provider: {}", provider)));
            }
        };

        match result {
            Ok(()) => Ok(ToolResponse::success(serde_json::json!({
                "success": true,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Tracking failed: {}", e))),
        }
    }
}

pub struct IdentifyUserTool;

impl Default for IdentifyUserTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for IdentifyUserTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "analytics_identify_user",
            "Identify User",
            "Identify a user with properties in analytics",
            "observability",
        )
        .with_param("api_key", ParameterSchema::string("Analytics API key").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: posthog, mixpanel, amplitude").required())
        .with_param("distinct_id", ParameterSchema::string("User identifier").required())
        .with_param("properties", ParameterSchema::json("User properties"))
        .with_output("success", OutputSchema::boolean("Whether identification succeeded"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let distinct_id = get_required_string_param(&params, "distinct_id")?;
        let properties = get_object_param(&params, "properties")
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();

        let profile = swissknife_observability_sdk::UserProfile {
            distinct_id,
            properties,
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "posthog")]
            "posthog" => {
                use swissknife_observability_sdk::posthog::PostHogClient;
                let client = PostHogClient::new(&api_key);
                client.identify(&profile).await
            }
            #[cfg(feature = "mixpanel")]
            "mixpanel" => {
                use swissknife_observability_sdk::mixpanel::MixpanelClient;
                let client = MixpanelClient::new(&api_key);
                client.identify(&profile).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported analytics provider: {}", provider)));
            }
        };

        match result {
            Ok(()) => Ok(ToolResponse::success(serde_json::json!({
                "success": true,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Identification failed: {}", e))),
        }
    }
}

pub struct CaptureErrorTool;

impl Default for CaptureErrorTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for CaptureErrorTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "sentry_capture_error",
            "Capture Error",
            "Capture an error/exception in Sentry",
            "observability",
        )
        .with_param("dsn", ParameterSchema::string("Sentry DSN").required().user_only())
        .with_param("message", ParameterSchema::string("Error message").required())
        .with_param("level", ParameterSchema::string("Error level: debug, info, warning, error, fatal").with_default(serde_json::json!("error")))
        .with_param("tags", ParameterSchema::json("Error tags"))
        .with_param("extra", ParameterSchema::json("Extra context"))
        .with_param("user_id", ParameterSchema::string("User ID"))
        .with_param("user_email", ParameterSchema::string("User email"))
        .with_output("event_id", OutputSchema::string("Sentry event ID"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let dsn = get_required_string_param(&params, "dsn")?;
        let message = get_required_string_param(&params, "message")?;
        let level_str = get_string_param(&params, "level").unwrap_or_else(|| "error".to_string());
        let tags = get_object_param(&params, "tags")
            .map(|m| m.iter().filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string()))).collect())
            .unwrap_or_default();
        let extra = get_object_param(&params, "extra")
            .map(|m| m.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        let user_id = get_string_param(&params, "user_id");
        let user_email = get_string_param(&params, "user_email");

        let level = match level_str.to_lowercase().as_str() {
            "debug" => ErrorLevel::Debug,
            "info" => ErrorLevel::Info,
            "warning" => ErrorLevel::Warning,
            "fatal" => ErrorLevel::Fatal,
            _ => ErrorLevel::Error,
        };

        let user = if user_id.is_some() || user_email.is_some() {
            Some(swissknife_observability_sdk::ErrorUser {
                id: user_id,
                email: user_email,
                username: None,
                ip_address: None,
            })
        } else {
            None
        };

        let error_event = ErrorEvent {
            id: None,
            message: message.clone(),
            level,
            platform: Some("rust".to_string()),
            timestamp: None,
            exception: None,
            tags,
            extra,
            user,
            contexts: HashMap::new(),
        };

        #[cfg(feature = "sentry")]
        {
            use swissknife_observability_sdk::sentry::SentryClient;
            let client = SentryClient::new(&dsn);
            match client.capture_error(&error_event).await {
                Ok(event_id) => Ok(ToolResponse::success(serde_json::json!({
                    "event_id": event_id,
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Capture failed: {}", e))),
            }
        }
        #[cfg(not(feature = "sentry"))]
        {
            let _ = (dsn, error_event);
            Ok(ToolResponse::error("Sentry feature not enabled"))
        }
    }
}

pub struct CaptureMessageTool;

impl Default for CaptureMessageTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for CaptureMessageTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "sentry_capture_message",
            "Capture Message",
            "Capture a message in Sentry",
            "observability",
        )
        .with_param("dsn", ParameterSchema::string("Sentry DSN").required().user_only())
        .with_param("message", ParameterSchema::string("Message to capture").required())
        .with_param("level", ParameterSchema::string("Level: debug, info, warning, error, fatal").with_default(serde_json::json!("info")))
        .with_output("event_id", OutputSchema::string("Sentry event ID"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let dsn = get_required_string_param(&params, "dsn")?;
        let message = get_required_string_param(&params, "message")?;
        let level_str = get_string_param(&params, "level").unwrap_or_else(|| "info".to_string());

        let level = match level_str.to_lowercase().as_str() {
            "debug" => ErrorLevel::Debug,
            "warning" => ErrorLevel::Warning,
            "error" => ErrorLevel::Error,
            "fatal" => ErrorLevel::Fatal,
            _ => ErrorLevel::Info,
        };

        #[cfg(feature = "sentry")]
        {
            use swissknife_observability_sdk::sentry::SentryClient;
            let client = SentryClient::new(&dsn);
            match client.capture_message(&message, level).await {
                Ok(event_id) => Ok(ToolResponse::success(serde_json::json!({
                    "event_id": event_id,
                }))),
                Err(e) => Ok(ToolResponse::error(format!("Capture failed: {}", e))),
            }
        }
        #[cfg(not(feature = "sentry"))]
        {
            let _ = (dsn, message, level);
            Ok(ToolResponse::error("Sentry feature not enabled"))
        }
    }
}
