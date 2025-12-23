mod error;

pub use error::{Error, Result};

#[cfg(feature = "datadog")]
pub mod datadog;

#[cfg(feature = "posthog")]
pub mod posthog;

#[cfg(feature = "sentry")]
pub mod sentry;

#[cfg(feature = "grafana")]
pub mod grafana;

#[cfg(feature = "mixpanel")]
pub mod mixpanel;

#[cfg(feature = "amplitude")]
pub mod amplitude;

#[cfg(feature = "incidentio")]
pub mod incidentio;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub name: String,
    pub timestamp: Option<DateTime<Utc>>,
    pub distinct_id: Option<String>,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub distinct_id: String,
    pub properties: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub name: String,
    pub value: f64,
    pub timestamp: Option<DateTime<Utc>>,
    pub tags: HashMap<String, String>,
    pub metric_type: MetricType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricType {
    Gauge,
    Counter,
    Histogram,
    Distribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub message: String,
    pub level: LogLevel,
    pub timestamp: Option<DateTime<Utc>>,
    pub attributes: HashMap<String, serde_json::Value>,
    pub service: Option<String>,
    pub source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub trace_id: String,
    pub spans: Vec<Span>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub span_id: String,
    pub parent_id: Option<String>,
    pub operation_name: String,
    pub service: String,
    pub start_time: DateTime<Utc>,
    pub duration_ms: f64,
    pub status: SpanStatus,
    pub tags: HashMap<String, String>,
    pub logs: Vec<SpanLog>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpanStatus {
    Ok,
    Error,
    Unset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLog {
    pub timestamp: DateTime<Utc>,
    pub fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub name: String,
    pub status: AlertStatus,
    pub severity: AlertSeverity,
    pub message: Option<String>,
    pub triggered_at: Option<DateTime<Utc>>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertStatus {
    Ok,
    Warn,
    Critical,
    Unknown,
    NoData,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub widgets: Vec<Widget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub title: String,
    pub widget_type: String,
    pub query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub id: Option<String>,
    pub message: String,
    pub level: ErrorLevel,
    pub platform: Option<String>,
    pub timestamp: Option<DateTime<Utc>>,
    pub exception: Option<ExceptionInfo>,
    pub tags: HashMap<String, String>,
    pub extra: HashMap<String, serde_json::Value>,
    pub user: Option<ErrorUser>,
    pub contexts: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorLevel {
    Debug,
    Info,
    Warning,
    Error,
    Fatal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExceptionInfo {
    pub exception_type: String,
    pub value: String,
    pub stacktrace: Option<Vec<StackFrame>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrame {
    pub filename: Option<String>,
    pub function: Option<String>,
    pub lineno: Option<u32>,
    pub colno: Option<u32>,
    pub context_line: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorUser {
    pub id: Option<String>,
    pub email: Option<String>,
    pub username: Option<String>,
    pub ip_address: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct TimeRange {
    pub start: Option<DateTime<Utc>>,
    pub end: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Default)]
pub struct MetricQuery {
    pub query: String,
    pub time_range: TimeRange,
    pub step: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSeries {
    pub metric: String,
    pub tags: HashMap<String, String>,
    pub points: Vec<(DateTime<Utc>, f64)>,
}

#[async_trait]
pub trait AnalyticsProvider: Send + Sync {
    async fn track(&self, event: &Event) -> Result<()>;
    async fn track_batch(&self, events: &[Event]) -> Result<()>;
    async fn identify(&self, profile: &UserProfile) -> Result<()>;
    async fn alias(&self, distinct_id: &str, alias: &str) -> Result<()>;
}

#[async_trait]
pub trait MetricsProvider: Send + Sync {
    async fn submit_metrics(&self, metrics: &[Metric]) -> Result<()>;
    async fn query_metrics(&self, query: &MetricQuery) -> Result<Vec<MetricSeries>>;
}

#[async_trait]
pub trait LoggingProvider: Send + Sync {
    async fn send_logs(&self, logs: &[LogEntry]) -> Result<()>;
    async fn query_logs(&self, query: &str, time_range: &TimeRange, limit: u32) -> Result<Vec<LogEntry>>;
}

#[async_trait]
pub trait TracingProvider: Send + Sync {
    async fn send_traces(&self, traces: &[Trace]) -> Result<()>;
    async fn get_trace(&self, trace_id: &str) -> Result<Trace>;
    async fn search_traces(&self, query: &str, time_range: &TimeRange, limit: u32) -> Result<Vec<Trace>>;
}

#[async_trait]
pub trait ErrorTrackingProvider: Send + Sync {
    async fn capture_error(&self, error: &ErrorEvent) -> Result<String>;
    async fn capture_message(&self, message: &str, level: ErrorLevel) -> Result<String>;
    async fn list_issues(&self, query: Option<&str>, limit: u32) -> Result<Vec<ErrorEvent>>;
}

#[async_trait]
pub trait AlertingProvider: Send + Sync {
    async fn list_alerts(&self) -> Result<Vec<Alert>>;
    async fn get_alert(&self, id: &str) -> Result<Alert>;
    async fn mute_alert(&self, id: &str, duration_minutes: u32) -> Result<()>;
    async fn unmute_alert(&self, id: &str) -> Result<()>;
}

#[async_trait]
pub trait DashboardProvider: Send + Sync {
    async fn list_dashboards(&self) -> Result<Vec<Dashboard>>;
    async fn get_dashboard(&self, id: &str) -> Result<Dashboard>;
}
