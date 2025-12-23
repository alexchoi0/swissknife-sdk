use crate::{Error, Result};
use crate::typeform::TypeformClient;
use serde::{Deserialize, Serialize};

impl TypeformClient {
    pub async fn list_forms(&self, page_size: Option<u32>) -> Result<FormsResponse> {
        let response = self.client()
            .get(format!("{}/forms", self.base_url()))
            .header("Authorization", format!("Bearer {}", self.access_token()))
            .query(&[("page_size", page_size.unwrap_or(10).to_string())])
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

        let forms: FormsResponse = response.json().await?;
        Ok(forms)
    }

    pub async fn get_form(&self, form_id: &str) -> Result<TypeformForm> {
        let response = self.client()
            .get(format!("{}/forms/{}", self.base_url(), form_id))
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

        let form: TypeformForm = response.json().await?;
        Ok(form)
    }

    pub async fn create_form(&self, request: CreateFormRequest) -> Result<TypeformForm> {
        let response = self.client()
            .post(format!("{}/forms", self.base_url()))
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

        let form: TypeformForm = response.json().await?;
        Ok(form)
    }

    pub async fn update_form(&self, form_id: &str, request: CreateFormRequest) -> Result<TypeformForm> {
        let response = self.client()
            .put(format!("{}/forms/{}", self.base_url(), form_id))
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

        let form: TypeformForm = response.json().await?;
        Ok(form)
    }

    pub async fn delete_form(&self, form_id: &str) -> Result<()> {
        let response = self.client()
            .delete(format!("{}/forms/{}", self.base_url(), form_id))
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

        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormsResponse {
    pub total_items: Option<i32>,
    pub page_count: Option<i32>,
    pub items: Vec<FormListItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormListItem {
    pub id: String,
    pub title: String,
    pub last_updated_at: Option<String>,
    #[serde(rename = "self")]
    pub self_link: Option<FormLink>,
    #[serde(rename = "_links")]
    pub links: Option<FormLinks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormLink {
    pub href: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormLinks {
    pub display: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TypeformForm {
    pub id: String,
    pub title: String,
    #[serde(rename = "type")]
    pub form_type: Option<String>,
    pub theme: Option<FormTheme>,
    pub workspace: Option<FormWorkspace>,
    pub settings: Option<FormSettings>,
    pub fields: Option<Vec<FormField>>,
    pub welcome_screens: Option<Vec<WelcomeScreen>>,
    pub thankyou_screens: Option<Vec<ThankYouScreen>>,
    #[serde(rename = "_links")]
    pub links: Option<FormLinks>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormTheme {
    pub href: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FormWorkspace {
    pub href: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormSettings {
    pub language: Option<String>,
    pub progress_bar: Option<String>,
    pub meta: Option<FormMeta>,
    pub is_public: Option<bool>,
    pub is_trial: Option<bool>,
    pub show_progress_bar: Option<bool>,
    pub show_typeform_branding: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormMeta {
    pub title: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub id: Option<String>,
    #[serde(rename = "ref")]
    pub field_ref: Option<String>,
    pub title: String,
    #[serde(rename = "type")]
    pub field_type: String,
    pub properties: Option<FieldProperties>,
    pub validations: Option<FieldValidations>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldProperties {
    pub description: Option<String>,
    pub choices: Option<Vec<FieldChoice>>,
    pub allow_multiple_selection: Option<bool>,
    pub allow_other_choice: Option<bool>,
    pub alphabetical_order: Option<bool>,
    pub randomize: Option<bool>,
    pub steps: Option<i32>,
    pub shape: Option<String>,
    pub labels: Option<ScaleLabels>,
    pub separator: Option<String>,
    pub structure: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldChoice {
    pub id: Option<String>,
    #[serde(rename = "ref")]
    pub choice_ref: Option<String>,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaleLabels {
    pub left: Option<String>,
    pub right: Option<String>,
    pub center: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidations {
    pub required: Option<bool>,
    pub max_length: Option<i32>,
    pub min_value: Option<i32>,
    pub max_value: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WelcomeScreen {
    #[serde(rename = "ref")]
    pub screen_ref: Option<String>,
    pub title: String,
    pub properties: Option<ScreenProperties>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThankYouScreen {
    #[serde(rename = "ref")]
    pub screen_ref: Option<String>,
    pub title: String,
    pub properties: Option<ScreenProperties>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenProperties {
    pub show_button: Option<bool>,
    pub button_text: Option<String>,
    pub button_mode: Option<String>,
    pub redirect_url: Option<String>,
    pub share_icons: Option<bool>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CreateFormRequest {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settings: Option<FormSettings>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Vec<FormField>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub welcome_screens: Option<Vec<WelcomeScreen>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thankyou_screens: Option<Vec<ThankYouScreen>>,
}
