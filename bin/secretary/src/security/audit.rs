use std::sync::atomic::{AtomicU64, Ordering};
use chrono::Utc;
use serde::Serialize;

static BLOCKED_EVENTS: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone, Serialize)]
pub enum SecurityEvent {
    PathTraversalBlocked { path: String, reason: String },
    SymlinkEscapeBlocked { path: String, target: String },
    HardlinkEscapeBlocked { path: String, inode: u64 },
    SsrfBlocked { url: String, reason: String },
    IpNormalized { original: String, normalized: String },
    RateLimitExceeded { operation: String, count: u64 },
}

pub fn log_security_event(event: SecurityEvent) {
    BLOCKED_EVENTS.fetch_add(1, Ordering::Relaxed);

    let timestamp = Utc::now().to_rfc3339();
    let event_json = serde_json::to_string(&event).unwrap_or_default();

    eprintln!("[SECURITY] {} {}", timestamp, event_json);
}

pub fn get_blocked_count() -> u64 {
    BLOCKED_EVENTS.load(Ordering::Relaxed)
}
