use rmcp::{tool_router, handler::server::router::tool::ToolRouter};
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "observability")]
use swissknife_observability_sdk as obs;

#[derive(Clone)]
pub struct ObservabilityTools {
    #[cfg(feature = "datadog")]
    pub datadog: Option<obs::datadog::DatadogClient>,
    #[cfg(feature = "posthog")]
    pub posthog: Option<obs::posthog::PostHogClient>,
    #[cfg(feature = "sentry")]
    pub sentry: Option<obs::sentry::SentryClient>,
    #[cfg(feature = "mixpanel")]
    pub mixpanel: Option<obs::mixpanel::MixpanelClient>,
}

impl ObservabilityTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "datadog")]
            datadog: None,
            #[cfg(feature = "posthog")]
            posthog: None,
            #[cfg(feature = "sentry")]
            sentry: None,
            #[cfg(feature = "mixpanel")]
            mixpanel: None,
        }
    }

    #[cfg(feature = "datadog")]
    pub fn with_datadog(mut self, client: obs::datadog::DatadogClient) -> Self {
        self.datadog = Some(client);
        self
    }

    #[cfg(feature = "posthog")]
    pub fn with_posthog(mut self, client: obs::posthog::PostHogClient) -> Self {
        self.posthog = Some(client);
        self
    }

    #[cfg(feature = "sentry")]
    pub fn with_sentry(mut self, client: obs::sentry::SentryClient) -> Self {
        self.sentry = Some(client);
        self
    }

    #[cfg(feature = "mixpanel")]
    pub fn with_mixpanel(mut self, client: obs::mixpanel::MixpanelClient) -> Self {
        self.mixpanel = Some(client);
        self
    }
}

impl Default for ObservabilityTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DatadogSendMetricRequest {
    pub metric: String,
    pub value: f64,
    #[serde(default)]
    pub metric_type: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DatadogSendEventRequest {
    pub title: String,
    pub text: String,
    #[serde(default)]
    pub alert_type: Option<String>,
    #[serde(default)]
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DatadogQueryMetricsRequest {
    pub query: String,
    pub from_ts: i64,
    pub to_ts: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PostHogCaptureRequest {
    pub distinct_id: String,
    pub event: String,
    #[serde(default)]
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PostHogIdentifyRequest {
    pub distinct_id: String,
    pub properties: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PostHogGetFeatureFlagRequest {
    pub distinct_id: String,
    pub key: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SentryCaptureMessageRequest {
    pub message: String,
    #[serde(default)]
    pub level: Option<String>,
    #[serde(default)]
    pub tags: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SentryListIssuesRequest {
    pub project_slug: String,
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MixpanelTrackRequest {
    pub event: String,
    pub distinct_id: String,
    #[serde(default)]
    pub properties: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct MixpanelSetProfileRequest {
    pub distinct_id: String,
    pub properties: serde_json::Value,
}

#[tool_router]
impl ObservabilityTools {
    #[cfg(feature = "datadog")]
    #[rmcp::tool(description = "Send a metric to Datadog")]
    pub async fn datadog_send_metric(
        &self,
        #[rmcp::tool(aggr)] req: DatadogSendMetricRequest,
    ) -> Result<String, String> {
        let client = self.datadog.as_ref()
            .ok_or_else(|| "Datadog client not configured".to_string())?;

        client.send_metric(
            &req.metric,
            req.value,
            req.metric_type.as_deref(),
            req.tags.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        Ok("Metric sent successfully".to_string())
    }

    #[cfg(feature = "datadog")]
    #[rmcp::tool(description = "Send an event to Datadog")]
    pub async fn datadog_send_event(
        &self,
        #[rmcp::tool(aggr)] req: DatadogSendEventRequest,
    ) -> Result<String, String> {
        let client = self.datadog.as_ref()
            .ok_or_else(|| "Datadog client not configured".to_string())?;

        client.send_event(
            &req.title,
            &req.text,
            req.alert_type.as_deref(),
            req.tags.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        Ok("Event sent successfully".to_string())
    }

    #[cfg(feature = "datadog")]
    #[rmcp::tool(description = "Query metrics from Datadog")]
    pub async fn datadog_query_metrics(
        &self,
        #[rmcp::tool(aggr)] req: DatadogQueryMetricsRequest,
    ) -> Result<String, String> {
        let client = self.datadog.as_ref()
            .ok_or_else(|| "Datadog client not configured".to_string())?;

        let result = client.query_metrics(&req.query, req.from_ts, req.to_ts).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "posthog")]
    #[rmcp::tool(description = "Capture an event in PostHog")]
    pub async fn posthog_capture(
        &self,
        #[rmcp::tool(aggr)] req: PostHogCaptureRequest,
    ) -> Result<String, String> {
        let client = self.posthog.as_ref()
            .ok_or_else(|| "PostHog client not configured".to_string())?;

        client.capture(&req.distinct_id, &req.event, req.properties).await
            .map_err(|e| e.to_string())?;

        Ok("Event captured successfully".to_string())
    }

    #[cfg(feature = "posthog")]
    #[rmcp::tool(description = "Identify a user in PostHog")]
    pub async fn posthog_identify(
        &self,
        #[rmcp::tool(aggr)] req: PostHogIdentifyRequest,
    ) -> Result<String, String> {
        let client = self.posthog.as_ref()
            .ok_or_else(|| "PostHog client not configured".to_string())?;

        client.identify(&req.distinct_id, req.properties).await
            .map_err(|e| e.to_string())?;

        Ok("User identified successfully".to_string())
    }

    #[cfg(feature = "posthog")]
    #[rmcp::tool(description = "Get a feature flag value from PostHog")]
    pub async fn posthog_get_feature_flag(
        &self,
        #[rmcp::tool(aggr)] req: PostHogGetFeatureFlagRequest,
    ) -> Result<String, String> {
        let client = self.posthog.as_ref()
            .ok_or_else(|| "PostHog client not configured".to_string())?;

        let value = client.get_feature_flag(&req.distinct_id, &req.key).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "key": req.key,
            "value": value
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "sentry")]
    #[rmcp::tool(description = "Capture a message in Sentry")]
    pub async fn sentry_capture_message(
        &self,
        #[rmcp::tool(aggr)] req: SentryCaptureMessageRequest,
    ) -> Result<String, String> {
        let client = self.sentry.as_ref()
            .ok_or_else(|| "Sentry client not configured".to_string())?;

        let event_id = client.capture_message(
            &req.message,
            req.level.as_deref(),
            req.tags,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "event_id": event_id
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "sentry")]
    #[rmcp::tool(description = "List issues from Sentry")]
    pub async fn sentry_list_issues(
        &self,
        #[rmcp::tool(aggr)] req: SentryListIssuesRequest,
    ) -> Result<String, String> {
        let client = self.sentry.as_ref()
            .ok_or_else(|| "Sentry client not configured".to_string())?;

        let issues = client.list_issues(
            &req.project_slug,
            req.query.as_deref(),
            req.limit,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&issues).map_err(|e| e.to_string())
    }

    #[cfg(feature = "mixpanel")]
    #[rmcp::tool(description = "Track an event in Mixpanel")]
    pub async fn mixpanel_track(
        &self,
        #[rmcp::tool(aggr)] req: MixpanelTrackRequest,
    ) -> Result<String, String> {
        let client = self.mixpanel.as_ref()
            .ok_or_else(|| "Mixpanel client not configured".to_string())?;

        client.track(&req.event, &req.distinct_id, req.properties).await
            .map_err(|e| e.to_string())?;

        Ok("Event tracked successfully".to_string())
    }

    #[cfg(feature = "mixpanel")]
    #[rmcp::tool(description = "Set user profile properties in Mixpanel")]
    pub async fn mixpanel_set_profile(
        &self,
        #[rmcp::tool(aggr)] req: MixpanelSetProfileRequest,
    ) -> Result<String, String> {
        let client = self.mixpanel.as_ref()
            .ok_or_else(|| "Mixpanel client not configured".to_string())?;

        client.set_profile(&req.distinct_id, req.properties).await
            .map_err(|e| e.to_string())?;

        Ok("Profile updated successfully".to_string())
    }
}
