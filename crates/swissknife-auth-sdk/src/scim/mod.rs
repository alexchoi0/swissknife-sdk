use crate::{Error, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub const SCIM_CONTENT_TYPE: &str = "application/scim+json";
pub const SCIM_SCHEMA_USER: &str = "urn:ietf:params:scim:schemas:core:2.0:User";
pub const SCIM_SCHEMA_GROUP: &str = "urn:ietf:params:scim:schemas:core:2.0:Group";
pub const SCIM_SCHEMA_ENTERPRISE_USER: &str = "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User";
pub const SCIM_SCHEMA_LIST_RESPONSE: &str = "urn:ietf:params:scim:api:messages:2.0:ListResponse";
pub const SCIM_SCHEMA_ERROR: &str = "urn:ietf:params:scim:api:messages:2.0:Error";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimUser {
    pub schemas: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub user_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<ScimName>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nick_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred_language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,
    #[serde(default)]
    pub active: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub emails: Vec<ScimEmail>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub phone_numbers: Vec<ScimPhoneNumber>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub addresses: Vec<ScimAddress>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<ScimGroupMembership>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ScimMeta>,
    #[serde(rename = "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User", skip_serializing_if = "Option::is_none")]
    pub enterprise_user: Option<ScimEnterpriseUser>,
}

impl ScimUser {
    pub fn new(user_name: impl Into<String>) -> Self {
        Self {
            schemas: vec![SCIM_SCHEMA_USER.into()],
            id: None,
            external_id: None,
            user_name: user_name.into(),
            name: None,
            display_name: None,
            nick_name: None,
            profile_url: None,
            title: None,
            user_type: None,
            preferred_language: None,
            locale: None,
            timezone: None,
            active: true,
            password: None,
            emails: Vec::new(),
            phone_numbers: Vec::new(),
            addresses: Vec::new(),
            groups: Vec::new(),
            meta: None,
            enterprise_user: None,
        }
    }

    pub fn with_name(mut self, given_name: impl Into<String>, family_name: impl Into<String>) -> Self {
        let given = given_name.into();
        let family = family_name.into();
        self.name = Some(ScimName {
            formatted: Some(format!("{} {}", given, family)),
            family_name: Some(family),
            given_name: Some(given),
            middle_name: None,
            honorific_prefix: None,
            honorific_suffix: None,
        });
        self
    }

    pub fn with_email(mut self, email: impl Into<String>, primary: bool) -> Self {
        self.emails.push(ScimEmail {
            value: email.into(),
            type_: Some("work".into()),
            primary,
        });
        self
    }

    pub fn with_enterprise_extension(mut self) -> Self {
        if !self.schemas.contains(&SCIM_SCHEMA_ENTERPRISE_USER.to_string()) {
            self.schemas.push(SCIM_SCHEMA_ENTERPRISE_USER.into());
        }
        if self.enterprise_user.is_none() {
            self.enterprise_user = Some(ScimEnterpriseUser::default());
        }
        self
    }

    pub fn primary_email(&self) -> Option<&str> {
        self.emails.iter()
            .find(|e| e.primary)
            .or_else(|| self.emails.first())
            .map(|e| e.value.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub middle_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub honorific_prefix: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub honorific_suffix: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimEmail {
    pub value: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimPhoneNumber {
    pub value: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimAddress {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub formatted: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub street_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locality: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(default)]
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroupMembership {
    pub value: String,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimEnterpriseUser {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub employee_number: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cost_center: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub division: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<ScimManager>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimManager {
    pub value: String,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimMeta {
    pub resource_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_modified: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimGroup {
    pub schemas: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,
    pub display_name: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<ScimMember>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ScimMeta>,
}

impl ScimGroup {
    pub fn new(display_name: impl Into<String>) -> Self {
        Self {
            schemas: vec![SCIM_SCHEMA_GROUP.into()],
            id: None,
            external_id: None,
            display_name: display_name.into(),
            members: Vec::new(),
            meta: None,
        }
    }

    pub fn add_member(mut self, user_id: impl Into<String>, display: Option<String>) -> Self {
        self.members.push(ScimMember {
            value: user_id.into(),
            ref_: None,
            display,
        });
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimMember {
    pub value: String,
    #[serde(rename = "$ref", skip_serializing_if = "Option::is_none")]
    pub ref_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimListResponse<T> {
    pub schemas: Vec<String>,
    pub total_results: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub items_per_page: Option<u64>,
    #[serde(rename = "Resources")]
    pub resources: Vec<T>,
}

impl<T> ScimListResponse<T> {
    pub fn new(resources: Vec<T>, total_results: u64) -> Self {
        Self {
            schemas: vec![SCIM_SCHEMA_LIST_RESPONSE.into()],
            total_results,
            start_index: Some(1),
            items_per_page: Some(resources.len() as u64),
            resources,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScimError {
    pub schemas: Vec<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scim_type: Option<String>,
    pub detail: String,
}

impl ScimError {
    pub fn new(status: u16, detail: impl Into<String>) -> Self {
        Self {
            schemas: vec![SCIM_SCHEMA_ERROR.into()],
            status: status.to_string(),
            scim_type: None,
            detail: detail.into(),
        }
    }

    pub fn not_found(detail: impl Into<String>) -> Self {
        Self::new(404, detail)
    }

    pub fn conflict(detail: impl Into<String>) -> Self {
        let mut error = Self::new(409, detail);
        error.scim_type = Some("uniqueness".into());
        error
    }

    pub fn bad_request(detail: impl Into<String>) -> Self {
        Self::new(400, detail)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimPatchOp {
    pub schemas: Vec<String>,
    #[serde(rename = "Operations")]
    pub operations: Vec<ScimPatchOperation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimPatchOperation {
    pub op: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

#[async_trait]
pub trait ScimUserProvider: Send + Sync {
    async fn create_user(&self, user: ScimUser) -> Result<ScimUser>;
    async fn get_user(&self, id: &str) -> Result<Option<ScimUser>>;
    async fn update_user(&self, id: &str, user: ScimUser) -> Result<ScimUser>;
    async fn patch_user(&self, id: &str, patch: ScimPatchOp) -> Result<ScimUser>;
    async fn delete_user(&self, id: &str) -> Result<()>;
    async fn list_users(&self, filter: Option<&str>, start_index: u64, count: u64) -> Result<ScimListResponse<ScimUser>>;
}

#[async_trait]
pub trait ScimGroupProvider: Send + Sync {
    async fn create_group(&self, group: ScimGroup) -> Result<ScimGroup>;
    async fn get_group(&self, id: &str) -> Result<Option<ScimGroup>>;
    async fn update_group(&self, id: &str, group: ScimGroup) -> Result<ScimGroup>;
    async fn patch_group(&self, id: &str, patch: ScimPatchOp) -> Result<ScimGroup>;
    async fn delete_group(&self, id: &str) -> Result<()>;
    async fn list_groups(&self, filter: Option<&str>, start_index: u64, count: u64) -> Result<ScimListResponse<ScimGroup>>;
}

pub struct ScimClient {
    http: reqwest::Client,
    base_url: String,
    auth_token: String,
}

impl ScimClient {
    pub fn new(base_url: impl Into<String>, auth_token: impl Into<String>) -> Self {
        Self {
            http: reqwest::Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            auth_token: auth_token.into(),
        }
    }

    pub async fn create_user(&self, user: &ScimUser) -> Result<ScimUser> {
        let url = format!("{}/Users", self.base_url);

        let response = self.http
            .post(&url)
            .header("Content-Type", SCIM_CONTENT_TYPE)
            .bearer_auth(&self.auth_token)
            .json(user)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ScimError = response.json().await?;
            return Err(Error::Provider(error.detail));
        }

        let created: ScimUser = response.json().await?;
        Ok(created)
    }

    pub async fn get_user(&self, id: &str) -> Result<Option<ScimUser>> {
        let url = format!("{}/Users/{}", self.base_url, id);

        let response = self.http
            .get(&url)
            .header("Accept", SCIM_CONTENT_TYPE)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            let error: ScimError = response.json().await?;
            return Err(Error::Provider(error.detail));
        }

        let user: ScimUser = response.json().await?;
        Ok(Some(user))
    }

    pub async fn update_user(&self, id: &str, user: &ScimUser) -> Result<ScimUser> {
        let url = format!("{}/Users/{}", self.base_url, id);

        let response = self.http
            .put(&url)
            .header("Content-Type", SCIM_CONTENT_TYPE)
            .bearer_auth(&self.auth_token)
            .json(user)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ScimError = response.json().await?;
            return Err(Error::Provider(error.detail));
        }

        let updated: ScimUser = response.json().await?;
        Ok(updated)
    }

    pub async fn delete_user(&self, id: &str) -> Result<()> {
        let url = format!("{}/Users/{}", self.base_url, id);

        let response = self.http
            .delete(&url)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;

        if !response.status().is_success() && response.status().as_u16() != 404 {
            let error: ScimError = response.json().await?;
            return Err(Error::Provider(error.detail));
        }

        Ok(())
    }

    pub async fn list_users(&self, filter: Option<&str>, start_index: Option<u64>, count: Option<u64>) -> Result<ScimListResponse<ScimUser>> {
        let mut url = format!("{}/Users", self.base_url);

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

        let response = self.http
            .get(&url)
            .header("Accept", SCIM_CONTENT_TYPE)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ScimError = response.json().await?;
            return Err(Error::Provider(error.detail));
        }

        let list: ScimListResponse<ScimUser> = response.json().await?;
        Ok(list)
    }

    pub async fn create_group(&self, group: &ScimGroup) -> Result<ScimGroup> {
        let url = format!("{}/Groups", self.base_url);

        let response = self.http
            .post(&url)
            .header("Content-Type", SCIM_CONTENT_TYPE)
            .bearer_auth(&self.auth_token)
            .json(group)
            .send()
            .await?;

        if !response.status().is_success() {
            let error: ScimError = response.json().await?;
            return Err(Error::Provider(error.detail));
        }

        let created: ScimGroup = response.json().await?;
        Ok(created)
    }

    pub async fn get_group(&self, id: &str) -> Result<Option<ScimGroup>> {
        let url = format!("{}/Groups/{}", self.base_url, id);

        let response = self.http
            .get(&url)
            .header("Accept", SCIM_CONTENT_TYPE)
            .bearer_auth(&self.auth_token)
            .send()
            .await?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            let error: ScimError = response.json().await?;
            return Err(Error::Provider(error.detail));
        }

        let group: ScimGroup = response.json().await?;
        Ok(Some(group))
    }
}
