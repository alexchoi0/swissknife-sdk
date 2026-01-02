use swissknife_ai_sdk::memory::DuckDBMemory;
use uuid::Uuid;

use crate::format::truncate;

pub struct SessionManager<'a> {
    memory: &'a DuckDBMemory,
    pub session_id: String,
}

impl<'a> SessionManager<'a> {
    pub fn new(
        memory: &'a DuckDBMemory,
        session_id: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let session_id = match session_id {
            Some(id) => {
                let session = memory.get_or_create_session(&id)?;
                eprintln!(
                    "Session: {} ({})",
                    session.session_id,
                    session.title.as_deref().unwrap_or("Untitled")
                );
                session.session_id
            }
            None => {
                let sessions = memory.list_sessions(1)?;
                if let Some(session) = sessions.into_iter().next() {
                    eprintln!(
                        "Resuming session: {} ({})",
                        session.session_id,
                        session.title.as_deref().unwrap_or("Untitled")
                    );
                    session.session_id
                } else {
                    let new_id = Uuid::new_v4().to_string();
                    memory.create_session(&new_id, None)?;
                    eprintln!("New session: {}", new_id);
                    new_id
                }
            }
        };
        Ok(Self { memory, session_id })
    }

    pub fn update_title_if_needed(&self) -> Result<(), Box<dyn std::error::Error>> {
        let count = self.memory.action_count(&self.session_id)?;
        if count == 2 {
            if let Some(action) = self.memory.get_messages(&self.session_id)?.first() {
                let title = truncate(&action.content, 50);
                self.memory.update_session_title(&self.session_id, &title)?;
            }
        }
        Ok(())
    }
}
