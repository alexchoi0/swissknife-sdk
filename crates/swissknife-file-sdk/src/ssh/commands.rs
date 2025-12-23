use crate::{Error, Result};
use crate::ssh::SshClient;
use serde::{Deserialize, Serialize};

impl SshClient {
    pub async fn execute(&self, command: &str, options: Option<ExecuteOptions>) -> Result<CommandResult> {
        let mut body = serde_json::json!({
            "connection": self.connection_info(),
            "command": command
        });

        if let Some(opts) = options {
            if let Some(timeout) = opts.timeout_ms {
                body["timeout"] = serde_json::Value::Number(timeout.into());
            }
            if let Some(cwd) = opts.cwd {
                body["cwd"] = serde_json::Value::String(cwd);
            }
            if let Some(env) = opts.env {
                body["env"] = serde_json::to_value(env).unwrap_or_default();
            }
        }

        let response = self.client()
            .post(format!("{}/ssh/execute", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        let result: CommandResult = response.json().await
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(result)
    }

    pub async fn execute_interactive(&self, command: &str) -> Result<String> {
        let body = serde_json::json!({
            "connection": self.connection_info(),
            "command": command,
            "interactive": true
        });

        let response = self.client()
            .post(format!("{}/ssh/execute", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        let result: InteractiveResponse = response.json().await
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(result.session_id)
    }

    pub async fn send_input(&self, session_id: &str, input: &str) -> Result<SessionOutput> {
        let body = serde_json::json!({
            "sessionId": session_id,
            "input": input
        });

        let response = self.client()
            .post(format!("{}/ssh/input", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        let result: SessionOutput = response.json().await
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(result)
    }

    pub async fn close_session(&self, session_id: &str) -> Result<()> {
        let body = serde_json::json!({
            "sessionId": session_id
        });

        let response = self.client()
            .post(format!("{}/ssh/close", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        Ok(())
    }

    pub async fn upload(&self, local_content: &[u8], remote_path: &str) -> Result<()> {
        use base64::Engine;
        let content = base64::engine::general_purpose::STANDARD.encode(local_content);

        let body = serde_json::json!({
            "connection": self.connection_info(),
            "content": content,
            "remotePath": remote_path
        });

        let response = self.client()
            .post(format!("{}/ssh/upload", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        Ok(())
    }

    pub async fn download(&self, remote_path: &str) -> Result<Vec<u8>> {
        let body = serde_json::json!({
            "connection": self.connection_info(),
            "remotePath": remote_path
        });

        let response = self.client()
            .post(format!("{}/ssh/download", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            if status.as_u16() == 404 {
                return Err(Error::NotFound(remote_path.to_string()));
            }
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        let result: DownloadResponse = response.json().await
            .map_err(|e| Error::Io(e.to_string()))?;

        use base64::Engine;
        base64::engine::general_purpose::STANDARD
            .decode(&result.content)
            .map_err(|e| Error::Io(e.to_string()))
    }

    pub async fn port_forward(&self, local_port: u16, remote_host: &str, remote_port: u16) -> Result<String> {
        let body = serde_json::json!({
            "connection": self.connection_info(),
            "localPort": local_port,
            "remoteHost": remote_host,
            "remotePort": remote_port
        });

        let response = self.client()
            .post(format!("{}/ssh/forward", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        let result: ForwardResponse = response.json().await
            .map_err(|e| Error::Io(e.to_string()))?;

        Ok(result.forward_id)
    }

    pub async fn stop_forward(&self, forward_id: &str) -> Result<()> {
        let body = serde_json::json!({
            "forwardId": forward_id
        });

        let response = self.client()
            .post(format!("{}/ssh/forward/stop", self.gateway_url()))
            .header("Authorization", format!("Bearer {}", self.api_key()))
            .json(&body)
            .send()
            .await
            .map_err(|e| Error::Connection(e.to_string()))?;

        if !response.status().is_success() {
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Io(text));
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Default)]
pub struct ExecuteOptions {
    pub timeout_ms: Option<u32>,
    pub cwd: Option<String>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct InteractiveResponse {
    session_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SessionOutput {
    pub stdout: String,
    pub stderr: String,
    pub is_complete: bool,
}

#[derive(Debug, Deserialize)]
struct DownloadResponse {
    content: String,
}

#[derive(Debug, Deserialize)]
struct ForwardResponse {
    forward_id: String,
}
