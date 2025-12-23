use crate::{Error, Result, Task, TaskProvider, TaskStatus, TaskPriority};
use crate::microsoft::MicrosoftClient;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

impl MicrosoftClient {
    pub async fn list_plans(&self, group_id: &str) -> Result<PlansResponse> {
        let response = self.client()
            .get(format!("{}/groups/{}/planner/plans", self.base_url(), group_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let plans: PlansResponse = response.json().await?;
        Ok(plans)
    }

    pub async fn get_plan(&self, plan_id: &str) -> Result<PlannerPlan> {
        let response = self.client()
            .get(format!("{}/planner/plans/{}", self.base_url(), plan_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let plan: PlannerPlan = response.json().await?;
        Ok(plan)
    }

    pub async fn list_plan_tasks(&self, plan_id: &str) -> Result<TasksResponse> {
        let response = self.client()
            .get(format!("{}/planner/plans/{}/tasks", self.base_url(), plan_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let tasks: TasksResponse = response.json().await?;
        Ok(tasks)
    }

    pub async fn get_planner_task(&self, task_id: &str) -> Result<PlannerTask> {
        let response = self.client()
            .get(format!("{}/planner/tasks/{}", self.base_url(), task_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let task: PlannerTask = response.json().await?;
        Ok(task)
    }

    pub async fn create_planner_task(&self, request: CreatePlannerTaskRequest) -> Result<PlannerTask> {
        let response = self.client()
            .post(format!("{}/planner/tasks", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&request)
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

        let task: PlannerTask = response.json().await?;
        Ok(task)
    }

    pub async fn update_planner_task(&self, task_id: &str, etag: &str, updates: UpdatePlannerTaskRequest) -> Result<PlannerTask> {
        let response = self.client()
            .patch(format!("{}/planner/tasks/{}", self.base_url(), task_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("If-Match", etag)
            .json(&updates)
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

        let task: PlannerTask = response.json().await?;
        Ok(task)
    }

    pub async fn delete_planner_task(&self, task_id: &str, etag: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/planner/tasks/{}", self.base_url(), task_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .header("If-Match", etag)
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

        Ok(())
    }

    pub async fn list_plan_buckets(&self, plan_id: &str) -> Result<BucketsResponse> {
        let response = self.client()
            .get(format!("{}/planner/plans/{}/buckets", self.base_url(), plan_id))
            .header("Authorization", format!("Bearer {}", self.access_token()))
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

        let buckets: BucketsResponse = response.json().await?;
        Ok(buckets)
    }

    pub async fn create_bucket(&self, plan_id: &str, name: &str) -> Result<PlannerBucket> {
        let body = serde_json::json!({
            "name": name,
            "planId": plan_id,
            "orderHint": " !"
        });

        let response = self.client()
            .post(format!("{}/planner/buckets", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .json(&body)
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

        let bucket: PlannerBucket = response.json().await?;
        Ok(bucket)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlansResponse {
    pub value: Vec<PlannerPlan>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlannerPlan {
    pub id: String,
    pub title: String,
    pub owner: Option<String>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TasksResponse {
    pub value: Vec<PlannerTask>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlannerTask {
    pub id: String,
    #[serde(rename = "planId")]
    pub plan_id: String,
    #[serde(rename = "bucketId")]
    pub bucket_id: Option<String>,
    pub title: String,
    #[serde(rename = "percentComplete")]
    pub percent_complete: Option<i32>,
    pub priority: Option<i32>,
    #[serde(rename = "dueDateTime")]
    pub due_date_time: Option<String>,
    #[serde(rename = "startDateTime")]
    pub start_date_time: Option<String>,
    #[serde(rename = "createdDateTime")]
    pub created_date_time: Option<String>,
    #[serde(rename = "completedDateTime")]
    pub completed_date_time: Option<String>,
    pub assignments: Option<serde_json::Value>,
    #[serde(rename = "@odata.etag")]
    pub etag: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreatePlannerTaskRequest {
    #[serde(rename = "planId")]
    pub plan_id: String,
    pub title: String,
    #[serde(rename = "bucketId", skip_serializing_if = "Option::is_none")]
    pub bucket_id: Option<String>,
    #[serde(rename = "dueDateTime", skip_serializing_if = "Option::is_none")]
    pub due_date_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignments: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Default)]
pub struct UpdatePlannerTaskRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(rename = "percentComplete", skip_serializing_if = "Option::is_none")]
    pub percent_complete: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(rename = "dueDateTime", skip_serializing_if = "Option::is_none")]
    pub due_date_time: Option<String>,
    #[serde(rename = "bucketId", skip_serializing_if = "Option::is_none")]
    pub bucket_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BucketsResponse {
    pub value: Vec<PlannerBucket>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PlannerBucket {
    pub id: String,
    pub name: String,
    #[serde(rename = "planId")]
    pub plan_id: String,
    #[serde(rename = "orderHint")]
    pub order_hint: Option<String>,
}

fn map_planner_priority(priority: Option<i32>) -> TaskPriority {
    match priority {
        Some(1) => TaskPriority::Urgent,
        Some(3) => TaskPriority::High,
        Some(5) => TaskPriority::Medium,
        Some(9) => TaskPriority::Low,
        _ => TaskPriority::None,
    }
}

fn map_task_priority_to_planner(priority: &TaskPriority) -> i32 {
    match priority {
        TaskPriority::Urgent => 1,
        TaskPriority::High => 3,
        TaskPriority::Medium => 5,
        TaskPriority::Low => 9,
        TaskPriority::None => 5,
    }
}

fn map_planner_status(percent_complete: Option<i32>) -> TaskStatus {
    match percent_complete {
        Some(100) => TaskStatus::Done,
        Some(50) => TaskStatus::InProgress,
        _ => TaskStatus::Todo,
    }
}

pub struct PlannerProvider {
    client: MicrosoftClient,
    plan_id: String,
}

impl PlannerProvider {
    pub fn new(access_token: &str, plan_id: &str) -> Self {
        Self {
            client: MicrosoftClient::new(access_token),
            plan_id: plan_id.to_string(),
        }
    }
}

#[async_trait]
impl TaskProvider for PlannerProvider {
    async fn list_tasks(&self, _project_id: Option<&str>) -> Result<Vec<Task>> {
        let response = self.client.list_plan_tasks(&self.plan_id).await?;

        let tasks = response.value.into_iter().map(|t| Task {
            id: t.id,
            title: t.title,
            description: None,
            status: map_planner_status(t.percent_complete),
            priority: map_planner_priority(t.priority),
            assignee: None,
            due_date: t.due_date_time.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            created_at: t.created_date_time.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: None,
            labels: vec![],
            url: None,
            parent_id: t.bucket_id,
        }).collect();

        Ok(tasks)
    }

    async fn get_task(&self, task_id: &str) -> Result<Task> {
        let t = self.client.get_planner_task(task_id).await?;

        Ok(Task {
            id: t.id,
            title: t.title,
            description: None,
            status: map_planner_status(t.percent_complete),
            priority: map_planner_priority(t.priority),
            assignee: None,
            due_date: t.due_date_time.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            created_at: t.created_date_time.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: None,
            labels: vec![],
            url: None,
            parent_id: t.bucket_id,
        })
    }

    async fn create_task(&self, task: &Task) -> Result<Task> {
        let request = CreatePlannerTaskRequest {
            plan_id: self.plan_id.clone(),
            title: task.title.clone(),
            bucket_id: task.parent_id.clone(),
            due_date_time: task.due_date.map(|d| d.to_rfc3339()),
            priority: Some(map_task_priority_to_planner(&task.priority)),
            assignments: None,
        };

        let t = self.client.create_planner_task(request).await?;

        Ok(Task {
            id: t.id,
            title: t.title,
            description: None,
            status: map_planner_status(t.percent_complete),
            priority: map_planner_priority(t.priority),
            assignee: None,
            due_date: t.due_date_time.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            created_at: t.created_date_time.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: None,
            labels: vec![],
            url: None,
            parent_id: t.bucket_id,
        })
    }

    async fn update_task(&self, task_id: &str, task: &Task) -> Result<Task> {
        let current = self.client.get_planner_task(task_id).await?;
        let etag = current.etag.ok_or_else(|| Error::Api {
            message: "No etag found for task".to_string(),
            code: None,
        })?;

        let percent_complete = match task.status {
            TaskStatus::Done => Some(100),
            TaskStatus::InProgress => Some(50),
            TaskStatus::Todo => Some(0),
            _ => None,
        };

        let updates = UpdatePlannerTaskRequest {
            title: Some(task.title.clone()),
            percent_complete,
            priority: Some(map_task_priority_to_planner(&task.priority)),
            due_date_time: task.due_date.map(|d| d.to_rfc3339()),
            bucket_id: task.parent_id.clone(),
        };

        let t = self.client.update_planner_task(task_id, &etag, updates).await?;

        Ok(Task {
            id: t.id,
            title: t.title,
            description: None,
            status: map_planner_status(t.percent_complete),
            priority: map_planner_priority(t.priority),
            assignee: None,
            due_date: t.due_date_time.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            created_at: t.created_date_time.and_then(|d| chrono::DateTime::parse_from_rfc3339(&d).ok()).map(|dt| dt.with_timezone(&chrono::Utc)),
            updated_at: None,
            labels: vec![],
            url: None,
            parent_id: t.bucket_id,
        })
    }

    async fn delete_task(&self, task_id: &str) -> Result<()> {
        let current = self.client.get_planner_task(task_id).await?;
        let etag = current.etag.ok_or_else(|| Error::Api {
            message: "No etag found for task".to_string(),
            code: None,
        })?;

        self.client.delete_planner_task(task_id, &etag).await
    }
}
