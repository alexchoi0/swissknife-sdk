use crate::{Error, Result};
use crate::apify::ApifyClient;
use crate::apify::actors::ActorRun;
use serde::Deserialize;

impl ApifyClient {
    pub async fn get_run(&self, run_id: &str) -> Result<ActorRun> {
        let response = self.client()
            .get(format!("{}/actor-runs/{}", self.base_url(), run_id))
            .query(&[("token", self.api_token())])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ActorRunResponse = response.json().await?;
        Ok(result.data)
    }

    pub async fn list_runs(&self, params: Option<ListRunsParams>) -> Result<RunsResponse> {
        let mut request = self.client()
            .get(format!("{}/actor-runs", self.base_url()))
            .query(&[("token", self.api_token())]);

        if let Some(p) = params {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(offset) = p.offset {
                query.push(("offset", offset.to_string()));
            }
            if let Some(limit) = p.limit {
                query.push(("limit", limit.to_string()));
            }
            if let Some(desc) = p.desc {
                query.push(("desc", desc.to_string()));
            }
            if let Some(status) = p.status {
                query.push(("status", status));
            }
            if !query.is_empty() {
                request = request.query(&query);
            }
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: RunsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn abort_run(&self, run_id: &str, gracefully: Option<bool>) -> Result<ActorRun> {
        let mut request = self.client()
            .post(format!("{}/actor-runs/{}/abort", self.base_url(), run_id))
            .query(&[("token", self.api_token())]);

        if let Some(g) = gracefully {
            request = request.query(&[("gracefully", g.to_string())]);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ActorRunResponse = response.json().await?;
        Ok(result.data)
    }

    pub async fn metamorph_run(&self, run_id: &str, target_actor_id: &str, input: Option<serde_json::Value>) -> Result<ActorRun> {
        let mut request = self.client()
            .post(format!("{}/actor-runs/{}/metamorph", self.base_url(), run_id))
            .query(&[("token", self.api_token()), ("targetActorId", target_actor_id)]);

        if let Some(inp) = input {
            request = request.json(&inp);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ActorRunResponse = response.json().await?;
        Ok(result.data)
    }

    pub async fn resurrect_run(&self, run_id: &str) -> Result<ActorRun> {
        let response = self.client()
            .post(format!("{}/actor-runs/{}/resurrect", self.base_url(), run_id))
            .query(&[("token", self.api_token())])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ActorRunResponse = response.json().await?;
        Ok(result.data)
    }

    pub async fn reboot_run(&self, run_id: &str) -> Result<ActorRun> {
        let response = self.client()
            .post(format!("{}/actor-runs/{}/reboot", self.base_url(), run_id))
            .query(&[("token", self.api_token())])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let result: ActorRunResponse = response.json().await?;
        Ok(result.data)
    }

    pub async fn get_run_log(&self, run_id: &str) -> Result<String> {
        let response = self.client()
            .get(format!("{}/actor-runs/{}/log", self.base_url(), run_id))
            .query(&[("token", self.api_token())])
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::Api {
                message: text,
                code: Some(status.to_string()),
            });
        }

        let log = response.text().await?;
        Ok(log)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListRunsParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub desc: Option<bool>,
    pub status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActorRunResponse {
    pub data: ActorRun,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RunsResponse {
    pub data: RunsData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RunsData {
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<ActorRun>,
}
