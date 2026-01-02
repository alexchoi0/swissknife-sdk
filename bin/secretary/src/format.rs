use swissknife_ai_sdk::memory::{Action, ActionType, Session};

pub const SESSION_ID_LEN: usize = 8;
pub const PREVIEW_SHORT: usize = 60;
pub const PREVIEW_LONG: usize = 80;

pub fn truncate(text: &str, max: usize) -> String {
    text.chars().take(max).collect()
}

pub fn format_session(session: &Session, current_id: Option<&str>) -> String {
    let marker = current_id.map_or("  ", |id| if session.session_id == id { "* " } else { "  " });
    format!(
        "{}{}: {} ({})",
        marker,
        &session.session_id[..SESSION_ID_LEN.min(session.session_id.len())],
        session.title.as_deref().unwrap_or("Untitled"),
        session.updated_at.format("%Y-%m-%d %H:%M")
    )
}

pub fn format_action_type(action: &Action) -> String {
    match action.action_type {
        ActionType::Message => format!("[{}]", action.role.as_deref().unwrap_or("?")),
        ActionType::ToolCall => format!("[tool:{}]", action.tool_name.as_deref().unwrap_or("?")),
        ActionType::ToolResult => "[result]".to_string(),
        ActionType::Thinking => "[thinking]".to_string(),
    }
}

pub fn format_timestamp(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts / 1000, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|| "unknown".to_string())
}
