use crate::oauth2::{OAuth2Client, OAuth2Config, PkceChallenge};
use crate::{AuthSession, Error, Result, TokenResponse, User};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const RIPPLING_AUTH_URL: &str = "https://app.rippling.com/apps/PLATFORM/authorize";
const RIPPLING_TOKEN_URL: &str = "https://app.rippling.com/api/o/token/";
const RIPPLING_API_BASE: &str = "https://api.rippling.com/platform/api";

pub struct RipplingClient {
    oauth: OAuth2Client,
    http: reqwest::Client,
    api_key: Option<String>,
}

impl RipplingClient {
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self> {
        let config = OAuth2Config::new(
            client_id,
            RIPPLING_AUTH_URL,
            RIPPLING_TOKEN_URL,
            redirect_uri,
        )
        .client_secret(client_secret)
        .scope("employee:read")
        .scope("company:read");

        let oauth = OAuth2Client::new(config)?;
        let http = reqwest::Client::new();

        Ok(Self {
            oauth,
            http,
            api_key: None,
        })
    }

    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    pub fn authorization_url(&self) -> (String, PkceChallenge) {
        self.oauth.authorization_url_with_pkce()
    }

    pub async fn exchange_code(&self, code: &str, pkce_verifier: &str) -> Result<TokenResponse> {
        self.oauth.exchange_code(code, Some(pkce_verifier)).await
    }

    pub async fn refresh_token(&self, refresh_token: &str) -> Result<TokenResponse> {
        self.oauth.refresh_token(refresh_token).await
    }

    async fn api_get<T: for<'de> Deserialize<'de>>(&self, path: &str, access_token: Option<&str>) -> Result<T> {
        let mut request = self.http.get(format!("{}{}", RIPPLING_API_BASE, path));

        if let Some(token) = access_token {
            request = request.bearer_auth(token);
        } else if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        } else {
            return Err(Error::AuthFailed("No access token or API key provided".into()));
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::AuthFailed(format!("Rippling API error {}: {}", status, text)));
        }

        Ok(response.json().await?)
    }

    async fn api_post<T: for<'de> Deserialize<'de>, B: Serialize>(&self, path: &str, body: &B, access_token: Option<&str>) -> Result<T> {
        let mut request = self.http.post(format!("{}{}", RIPPLING_API_BASE, path));

        if let Some(token) = access_token {
            request = request.bearer_auth(token);
        } else if let Some(api_key) = &self.api_key {
            request = request.bearer_auth(api_key);
        } else {
            return Err(Error::AuthFailed("No access token or API key provided".into()));
        }

        let response = request.json(body).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::AuthFailed(format!("Rippling API error {}: {}", status, text)));
        }

        Ok(response.json().await?)
    }

    pub async fn get_current_user(&self, access_token: &str) -> Result<User> {
        let employee: RipplingEmployee = self.api_get("/employees/me", Some(access_token)).await?;
        Ok(employee.into_user())
    }

    pub async fn authenticate(&self, code: &str, pkce_verifier: &str) -> Result<AuthSession> {
        let tokens = self.exchange_code(code, pkce_verifier).await?;
        let user = self.get_current_user(&tokens.access_token).await?;
        Ok(AuthSession::new(user, tokens))
    }

    pub async fn get_company(&self, access_token: Option<&str>) -> Result<RipplingCompany> {
        self.api_get("/companies/current", access_token).await
    }

    pub async fn list_employees(&self, access_token: Option<&str>) -> Result<Vec<RipplingEmployee>> {
        let response: EmployeesResponse = self.api_get("/employees", access_token).await?;
        Ok(response.results)
    }

    pub async fn get_employee(&self, employee_id: &str, access_token: Option<&str>) -> Result<RipplingEmployee> {
        self.api_get(&format!("/employees/{}", employee_id), access_token).await
    }

    pub async fn list_departments(&self, access_token: Option<&str>) -> Result<Vec<RipplingDepartment>> {
        let response: DepartmentsResponse = self.api_get("/departments", access_token).await?;
        Ok(response.results)
    }

    pub async fn list_work_locations(&self, access_token: Option<&str>) -> Result<Vec<RipplingWorkLocation>> {
        let response: WorkLocationsResponse = self.api_get("/work_locations", access_token).await?;
        Ok(response.results)
    }

    pub async fn list_teams(&self, access_token: Option<&str>) -> Result<Vec<RipplingTeam>> {
        let response: TeamsResponse = self.api_get("/teams", access_token).await?;
        Ok(response.results)
    }

    pub async fn list_groups(&self, access_token: Option<&str>) -> Result<Vec<RipplingGroup>> {
        let response: GroupsResponse = self.api_get("/groups", access_token).await?;
        Ok(response.results)
    }

    pub async fn list_custom_fields(&self, access_token: Option<&str>) -> Result<Vec<RipplingCustomField>> {
        let response: CustomFieldsResponse = self.api_get("/custom_fields", access_token).await?;
        Ok(response.results)
    }

    pub async fn list_levels(&self, access_token: Option<&str>) -> Result<Vec<RipplingLevel>> {
        let response: LevelsResponse = self.api_get("/levels", access_token).await?;
        Ok(response.results)
    }

    pub async fn get_leave_balances(&self, employee_id: &str, access_token: Option<&str>) -> Result<Vec<RipplingLeaveBalance>> {
        let response: LeaveBalancesResponse = self.api_get(&format!("/employees/{}/leave_balances", employee_id), access_token).await?;
        Ok(response.results)
    }

    pub async fn list_leave_requests(&self, access_token: Option<&str>) -> Result<Vec<RipplingLeaveRequest>> {
        let response: LeaveRequestsResponse = self.api_get("/leave_requests", access_token).await?;
        Ok(response.results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplingCompany {
    pub id: String,
    pub name: String,
    pub ein: Option<String>,
    pub address: Option<RipplingAddress>,
    pub phone: Option<String>,
    pub primary_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplingAddress {
    pub street_line_1: Option<String>,
    pub street_line_2: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub zip: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplingEmployee {
    pub id: String,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub work_email: Option<String>,
    pub personal_email: Option<String>,
    pub phone: Option<String>,
    pub role_id: Option<String>,
    pub role_name: Option<String>,
    pub department_id: Option<String>,
    pub department_name: Option<String>,
    pub manager_id: Option<String>,
    pub employment_type: Option<String>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub status: Option<String>,
    pub work_location_id: Option<String>,
    pub work_location_name: Option<String>,
    pub photo_url: Option<String>,
    pub custom_fields: Option<HashMap<String, serde_json::Value>>,
}

impl RipplingEmployee {
    fn into_user(self) -> User {
        let name = match (&self.first_name, &self.last_name) {
            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
            (Some(first), None) => Some(first.clone()),
            (None, Some(last)) => Some(last.clone()),
            (None, None) => self.display_name.clone(),
        };

        let mut extra = HashMap::new();
        if let Some(dept) = &self.department_name {
            extra.insert("department".to_string(), serde_json::Value::String(dept.clone()));
        }
        if let Some(role) = &self.role_name {
            extra.insert("role".to_string(), serde_json::Value::String(role.clone()));
        }
        if let Some(manager) = &self.manager_id {
            extra.insert("manager_id".to_string(), serde_json::Value::String(manager.clone()));
        }
        if let Some(status) = &self.status {
            extra.insert("status".to_string(), serde_json::Value::String(status.clone()));
        }
        if let Some(emp_type) = &self.employment_type {
            extra.insert("employment_type".to_string(), serde_json::Value::String(emp_type.clone()));
        }

        User {
            id: self.id,
            email: self.work_email.or(self.personal_email),
            email_verified: Some(true),
            name,
            picture: self.photo_url,
            provider: Some("rippling".into()),
            extra,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplingDepartment {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplingWorkLocation {
    pub id: String,
    pub name: String,
    pub address: Option<RipplingAddress>,
    pub timezone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplingTeam {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplingGroup {
    pub id: String,
    pub name: String,
    pub group_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplingCustomField {
    pub id: String,
    pub name: String,
    pub field_type: String,
    pub options: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplingLevel {
    pub id: String,
    pub name: String,
    pub rank: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplingLeaveBalance {
    pub leave_type_id: String,
    pub leave_type_name: String,
    pub balance: f64,
    pub unit: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RipplingLeaveRequest {
    pub id: String,
    pub employee_id: String,
    pub leave_type_id: String,
    pub leave_type_name: Option<String>,
    pub start_date: String,
    pub end_date: String,
    pub status: String,
    pub requested_days: Option<f64>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
struct EmployeesResponse {
    results: Vec<RipplingEmployee>,
}

#[derive(Debug, Deserialize)]
struct DepartmentsResponse {
    results: Vec<RipplingDepartment>,
}

#[derive(Debug, Deserialize)]
struct WorkLocationsResponse {
    results: Vec<RipplingWorkLocation>,
}

#[derive(Debug, Deserialize)]
struct TeamsResponse {
    results: Vec<RipplingTeam>,
}

#[derive(Debug, Deserialize)]
struct GroupsResponse {
    results: Vec<RipplingGroup>,
}

#[derive(Debug, Deserialize)]
struct CustomFieldsResponse {
    results: Vec<RipplingCustomField>,
}

#[derive(Debug, Deserialize)]
struct LevelsResponse {
    results: Vec<RipplingLevel>,
}

#[derive(Debug, Deserialize)]
struct LeaveBalancesResponse {
    results: Vec<RipplingLeaveBalance>,
}

#[derive(Debug, Deserialize)]
struct LeaveRequestsResponse {
    results: Vec<RipplingLeaveRequest>,
}

pub struct RipplingScimClient {
    base_url: String,
    bearer_token: String,
    http: reqwest::Client,
}

impl RipplingScimClient {
    pub fn new(bearer_token: impl Into<String>) -> Self {
        Self {
            base_url: "https://api.rippling.com/platform/scim/v2".to_string(),
            bearer_token: bearer_token.into(),
            http: reqwest::Client::new(),
        }
    }

    async fn get<T: for<'de> Deserialize<'de>>(&self, path: &str) -> Result<T> {
        let response = self.http
            .get(format!("{}{}", self.base_url, path))
            .bearer_auth(&self.bearer_token)
            .header("Accept", "application/scim+json")
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(Error::AuthFailed(format!("SCIM error {}: {}", status, text)));
        }

        Ok(response.json().await?)
    }

    pub async fn list_users(&self, filter: Option<&str>, start_index: Option<u32>, count: Option<u32>) -> Result<ScimListResponse<ScimUser>> {
        let mut url = "/Users".to_string();
        let mut params = Vec::new();

        if let Some(f) = filter {
            params.push(format!("filter={}", urlencoding::encode(f)));
        }
        if let Some(s) = start_index {
            params.push(format!("startIndex={}", s));
        }
        if let Some(c) = count {
            params.push(format!("count={}", c));
        }

        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        self.get(&url).await
    }

    pub async fn get_user(&self, user_id: &str) -> Result<ScimUser> {
        self.get(&format!("/Users/{}", user_id)).await
    }

    pub async fn list_groups(&self, filter: Option<&str>) -> Result<ScimListResponse<ScimGroup>> {
        let url = if let Some(f) = filter {
            format!("/Groups?filter={}", urlencoding::encode(f))
        } else {
            "/Groups".to_string()
        };

        self.get(&url).await
    }

    pub async fn get_group(&self, group_id: &str) -> Result<ScimGroup> {
        self.get(&format!("/Groups/{}", group_id)).await
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimListResponse<T> {
    pub schemas: Vec<String>,
    pub total_results: u64,
    pub items_per_page: Option<u32>,
    pub start_index: Option<u32>,
    #[serde(rename = "Resources")]
    pub resources: Vec<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimUser {
    pub schemas: Vec<String>,
    pub id: String,
    pub external_id: Option<String>,
    pub user_name: String,
    pub name: Option<ScimName>,
    pub display_name: Option<String>,
    pub emails: Option<Vec<ScimEmail>>,
    pub phone_numbers: Option<Vec<ScimPhoneNumber>>,
    pub active: bool,
    pub groups: Option<Vec<ScimGroupRef>>,
    #[serde(rename = "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User")]
    pub enterprise: Option<ScimEnterpriseUser>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimName {
    pub formatted: Option<String>,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub middle_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimEmail {
    pub value: String,
    #[serde(rename = "type")]
    pub email_type: Option<String>,
    pub primary: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimPhoneNumber {
    pub value: String,
    #[serde(rename = "type")]
    pub phone_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroupRef {
    pub value: String,
    #[serde(rename = "$ref")]
    pub ref_url: Option<String>,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimEnterpriseUser {
    pub employee_number: Option<String>,
    pub department: Option<String>,
    pub division: Option<String>,
    pub organization: Option<String>,
    pub manager: Option<ScimManager>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimManager {
    pub value: Option<String>,
    #[serde(rename = "$ref")]
    pub ref_url: Option<String>,
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimGroup {
    pub schemas: Vec<String>,
    pub id: String,
    pub display_name: String,
    pub members: Option<Vec<ScimGroupMember>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroupMember {
    pub value: String,
    #[serde(rename = "$ref")]
    pub ref_url: Option<String>,
    pub display: Option<String>,
}

impl ScimUser {
    pub fn primary_email(&self) -> Option<&str> {
        self.emails.as_ref().and_then(|emails| {
            emails.iter()
                .find(|e| e.primary == Some(true))
                .or_else(|| emails.first())
                .map(|e| e.value.as_str())
        })
    }

    pub fn into_user(self) -> User {
        let name = self.name.as_ref()
            .and_then(|n| n.formatted.clone())
            .or_else(|| {
                match (&self.name.as_ref().and_then(|n| n.given_name.clone()), &self.name.as_ref().and_then(|n| n.family_name.clone())) {
                    (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
                    (Some(first), None) => Some(first.clone()),
                    (None, Some(last)) => Some(last.clone()),
                    _ => None,
                }
            })
            .or(self.display_name.clone());

        let mut extra = HashMap::new();
        if let Some(enterprise) = &self.enterprise {
            if let Some(dept) = &enterprise.department {
                extra.insert("department".to_string(), serde_json::Value::String(dept.clone()));
            }
            if let Some(emp_num) = &enterprise.employee_number {
                extra.insert("employee_number".to_string(), serde_json::Value::String(emp_num.clone()));
            }
        }
        extra.insert("active".to_string(), serde_json::Value::Bool(self.active));

        let email = self.primary_email().map(String::from);

        User {
            id: self.id,
            email,
            email_verified: Some(true),
            name,
            picture: None,
            provider: Some("rippling".into()),
            extra,
        }
    }
}
