use crate::{Error, Result};
use crate::apify::ApifyClient;
use serde::{Deserialize, Serialize};

impl ApifyClient {
    pub async fn list_actors(&self, params: Option<ListActorsParams>) -> Result<ActorsResponse> {
        let mut request = self.client()
            .get(format!("{}/acts", self.base_url()))
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

        let result: ActorsResponse = response.json().await?;
        Ok(result)
    }

    pub async fn get_actor(&self, actor_id: &str) -> Result<Actor> {
        let response = self.client()
            .get(format!("{}/acts/{}", self.base_url(), actor_id))
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

        let result: ActorResponse = response.json().await?;
        Ok(result.data)
    }

    pub async fn run_actor(&self, actor_id: &str, input: Option<serde_json::Value>, options: Option<RunActorOptions>) -> Result<ActorRun> {
        let mut request = self.client()
            .post(format!("{}/acts/{}/runs", self.base_url(), actor_id))
            .query(&[("token", self.api_token())]);

        if let Some(opts) = options {
            let mut query: Vec<(&str, String)> = Vec::new();
            if let Some(timeout) = opts.timeout_secs {
                query.push(("timeout", timeout.to_string()));
            }
            if let Some(memory) = opts.memory_mbytes {
                query.push(("memory", memory.to_string()));
            }
            if let Some(build) = opts.build {
                query.push(("build", build));
            }
            if opts.wait_for_finish.unwrap_or(false) {
                query.push(("waitForFinish", "300".to_string()));
            }
            if !query.is_empty() {
                request = request.query(&query);
            }
        }

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

    pub async fn run_actor_sync(&self, actor_id: &str, input: Option<serde_json::Value>, timeout_secs: Option<u32>) -> Result<serde_json::Value> {
        let timeout = timeout_secs.unwrap_or(300);

        let response = self.client()
            .post(format!("{}/acts/{}/run-sync-get-dataset-items", self.base_url(), actor_id))
            .query(&[("token", self.api_token()), ("timeout", &timeout.to_string())])
            .json(&input.unwrap_or(serde_json::json!({})))
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

        let result: serde_json::Value = response.json().await?;
        Ok(result)
    }

    pub async fn get_actor_builds(&self, actor_id: &str) -> Result<Vec<ActorBuild>> {
        let response = self.client()
            .get(format!("{}/acts/{}/builds", self.base_url(), actor_id))
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

        let result: ActorBuildsResponse = response.json().await?;
        Ok(result.data.items)
    }
}

#[derive(Debug, Clone, Default)]
pub struct ListActorsParams {
    pub offset: Option<u32>,
    pub limit: Option<u32>,
    pub desc: Option<bool>,
}

#[derive(Debug, Clone, Default)]
pub struct RunActorOptions {
    pub timeout_secs: Option<u32>,
    pub memory_mbytes: Option<u32>,
    pub build: Option<String>,
    pub wait_for_finish: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActorsResponse {
    pub data: ActorsData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActorsData {
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<Actor>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActorResponse {
    pub data: Actor,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Actor {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    pub name: String,
    pub username: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "restartOnError")]
    pub restart_on_error: Option<bool>,
    #[serde(rename = "isPublic")]
    pub is_public: Option<bool>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "modifiedAt")]
    pub modified_at: Option<String>,
    pub stats: Option<ActorStats>,
    #[serde(rename = "defaultRunOptions")]
    pub default_run_options: Option<ActorRunOptions>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActorStats {
    #[serde(rename = "totalBuilds")]
    pub total_builds: Option<u32>,
    #[serde(rename = "totalRuns")]
    pub total_runs: Option<u32>,
    #[serde(rename = "totalUsers")]
    pub total_users: Option<u32>,
    #[serde(rename = "totalUsers7Days")]
    pub total_users_7_days: Option<u32>,
    #[serde(rename = "totalUsers30Days")]
    pub total_users_30_days: Option<u32>,
    #[serde(rename = "totalUsers90Days")]
    pub total_users_90_days: Option<u32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActorRunOptions {
    pub build: Option<String>,
    #[serde(rename = "timeoutSecs")]
    pub timeout_secs: Option<u32>,
    #[serde(rename = "memoryMbytes")]
    pub memory_mbytes: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActorRunResponse {
    pub data: ActorRun,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActorRun {
    pub id: String,
    #[serde(rename = "actId")]
    pub act_id: String,
    #[serde(rename = "userId")]
    pub user_id: Option<String>,
    #[serde(rename = "startedAt")]
    pub started_at: Option<String>,
    #[serde(rename = "finishedAt")]
    pub finished_at: Option<String>,
    pub status: String,
    #[serde(rename = "statusMessage")]
    pub status_message: Option<String>,
    #[serde(rename = "isStatusMessageTerminal")]
    pub is_status_message_terminal: Option<bool>,
    pub meta: Option<ActorRunMeta>,
    pub stats: Option<ActorRunStats>,
    pub options: Option<ActorRunOptions>,
    #[serde(rename = "buildId")]
    pub build_id: Option<String>,
    #[serde(rename = "buildNumber")]
    pub build_number: Option<String>,
    #[serde(rename = "exitCode")]
    pub exit_code: Option<i32>,
    #[serde(rename = "defaultKeyValueStoreId")]
    pub default_key_value_store_id: Option<String>,
    #[serde(rename = "defaultDatasetId")]
    pub default_dataset_id: Option<String>,
    #[serde(rename = "defaultRequestQueueId")]
    pub default_request_queue_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActorRunMeta {
    pub origin: Option<String>,
    #[serde(rename = "clientIp")]
    pub client_ip: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActorRunStats {
    #[serde(rename = "inputBodyLen")]
    pub input_body_len: Option<u64>,
    #[serde(rename = "restartCount")]
    pub restart_count: Option<u32>,
    #[serde(rename = "durationMillis")]
    pub duration_millis: Option<u64>,
    #[serde(rename = "resurrectCount")]
    pub resurrect_count: Option<u32>,
    #[serde(rename = "runTimeSecs")]
    pub run_time_secs: Option<f64>,
    #[serde(rename = "metamorph")]
    pub metamorph: Option<u32>,
    #[serde(rename = "computeUnits")]
    pub compute_units: Option<f64>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActorBuildsResponse {
    pub data: ActorBuildsData,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActorBuildsData {
    pub total: u32,
    pub offset: u32,
    pub limit: u32,
    pub items: Vec<ActorBuild>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActorBuild {
    pub id: String,
    #[serde(rename = "actId")]
    pub act_id: String,
    pub status: String,
    #[serde(rename = "startedAt")]
    pub started_at: Option<String>,
    #[serde(rename = "finishedAt")]
    pub finished_at: Option<String>,
    #[serde(rename = "buildNumber")]
    pub build_number: Option<String>,
}
