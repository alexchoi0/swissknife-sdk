use rmcp::{tool_router, handler::server::router::tool::ToolRouter};
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "crm")]
use swissknife_crm_sdk as crm;

#[derive(Clone)]
pub struct CrmTools {
    #[cfg(feature = "salesforce")]
    pub salesforce: Option<crm::salesforce::SalesforceClient>,
    #[cfg(feature = "hubspot")]
    pub hubspot: Option<crm::hubspot::HubSpotClient>,
    #[cfg(feature = "pipedrive")]
    pub pipedrive: Option<crm::pipedrive::PipedriveClient>,
}

impl CrmTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "salesforce")]
            salesforce: None,
            #[cfg(feature = "hubspot")]
            hubspot: None,
            #[cfg(feature = "pipedrive")]
            pipedrive: None,
        }
    }

    #[cfg(feature = "salesforce")]
    pub fn with_salesforce(mut self, client: crm::salesforce::SalesforceClient) -> Self {
        self.salesforce = Some(client);
        self
    }

    #[cfg(feature = "hubspot")]
    pub fn with_hubspot(mut self, client: crm::hubspot::HubSpotClient) -> Self {
        self.hubspot = Some(client);
        self
    }

    #[cfg(feature = "pipedrive")]
    pub fn with_pipedrive(mut self, client: crm::pipedrive::PipedriveClient) -> Self {
        self.pipedrive = Some(client);
        self
    }
}

impl Default for CrmTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HubSpotCreateContactRequest {
    pub email: String,
    #[serde(default)]
    pub firstname: Option<String>,
    #[serde(default)]
    pub lastname: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub company: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HubSpotGetContactRequest {
    pub contact_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HubSpotSearchContactsRequest {
    pub query: String,
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HubSpotCreateDealRequest {
    pub dealname: String,
    pub pipeline: String,
    pub dealstage: String,
    #[serde(default)]
    pub amount: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct HubSpotCreateCompanyRequest {
    pub name: String,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub industry: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SalesforceQueryRequest {
    pub soql: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SalesforceCreateRecordRequest {
    pub object_type: String,
    pub fields: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SalesforceUpdateRecordRequest {
    pub object_type: String,
    pub record_id: String,
    pub fields: serde_json::Value,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SalesforceGetRecordRequest {
    pub object_type: String,
    pub record_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipedriveCreatePersonRequest {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub org_id: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct PipedriveCreateDealRequest {
    pub title: String,
    #[serde(default)]
    pub value: Option<f64>,
    #[serde(default)]
    pub currency: Option<String>,
    #[serde(default)]
    pub person_id: Option<i64>,
    #[serde(default)]
    pub org_id: Option<i64>,
}

#[tool_router]
impl CrmTools {
    #[cfg(feature = "hubspot")]
    #[rmcp::tool(description = "Create a new contact in HubSpot")]
    pub async fn hubspot_create_contact(
        &self,
        #[rmcp::tool(aggr)] req: HubSpotCreateContactRequest,
    ) -> Result<String, String> {
        let client = self.hubspot.as_ref()
            .ok_or_else(|| "HubSpot client not configured".to_string())?;

        let mut properties = std::collections::HashMap::new();
        properties.insert("email".to_string(), req.email);
        if let Some(v) = req.firstname {
            properties.insert("firstname".to_string(), v);
        }
        if let Some(v) = req.lastname {
            properties.insert("lastname".to_string(), v);
        }
        if let Some(v) = req.phone {
            properties.insert("phone".to_string(), v);
        }
        if let Some(v) = req.company {
            properties.insert("company".to_string(), v);
        }

        let contact = client.create_contact(properties).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": contact.id
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "hubspot")]
    #[rmcp::tool(description = "Get a HubSpot contact by ID")]
    pub async fn hubspot_get_contact(
        &self,
        #[rmcp::tool(aggr)] req: HubSpotGetContactRequest,
    ) -> Result<String, String> {
        let client = self.hubspot.as_ref()
            .ok_or_else(|| "HubSpot client not configured".to_string())?;

        let contact = client.get_contact(&req.contact_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&contact).map_err(|e| e.to_string())
    }

    #[cfg(feature = "hubspot")]
    #[rmcp::tool(description = "Search for contacts in HubSpot")]
    pub async fn hubspot_search_contacts(
        &self,
        #[rmcp::tool(aggr)] req: HubSpotSearchContactsRequest,
    ) -> Result<String, String> {
        let client = self.hubspot.as_ref()
            .ok_or_else(|| "HubSpot client not configured".to_string())?;

        let contacts = client.search_contacts(&req.query, req.limit).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&contacts).map_err(|e| e.to_string())
    }

    #[cfg(feature = "hubspot")]
    #[rmcp::tool(description = "Create a new deal in HubSpot")]
    pub async fn hubspot_create_deal(
        &self,
        #[rmcp::tool(aggr)] req: HubSpotCreateDealRequest,
    ) -> Result<String, String> {
        let client = self.hubspot.as_ref()
            .ok_or_else(|| "HubSpot client not configured".to_string())?;

        let mut properties = std::collections::HashMap::new();
        properties.insert("dealname".to_string(), req.dealname);
        properties.insert("pipeline".to_string(), req.pipeline);
        properties.insert("dealstage".to_string(), req.dealstage);
        if let Some(v) = req.amount {
            properties.insert("amount".to_string(), v);
        }

        let deal = client.create_deal(properties).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": deal.id
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "hubspot")]
    #[rmcp::tool(description = "Create a new company in HubSpot")]
    pub async fn hubspot_create_company(
        &self,
        #[rmcp::tool(aggr)] req: HubSpotCreateCompanyRequest,
    ) -> Result<String, String> {
        let client = self.hubspot.as_ref()
            .ok_or_else(|| "HubSpot client not configured".to_string())?;

        let mut properties = std::collections::HashMap::new();
        properties.insert("name".to_string(), req.name);
        if let Some(v) = req.domain {
            properties.insert("domain".to_string(), v);
        }
        if let Some(v) = req.industry {
            properties.insert("industry".to_string(), v);
        }

        let company = client.create_company(properties).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&serde_json::json!({
            "id": company.id
        })).map_err(|e| e.to_string())
    }

    #[cfg(feature = "salesforce")]
    #[rmcp::tool(description = "Execute a SOQL query in Salesforce")]
    pub async fn salesforce_query(
        &self,
        #[rmcp::tool(aggr)] req: SalesforceQueryRequest,
    ) -> Result<String, String> {
        let client = self.salesforce.as_ref()
            .ok_or_else(|| "Salesforce client not configured".to_string())?;

        let result = client.query(&req.soql).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "salesforce")]
    #[rmcp::tool(description = "Create a new record in Salesforce")]
    pub async fn salesforce_create_record(
        &self,
        #[rmcp::tool(aggr)] req: SalesforceCreateRecordRequest,
    ) -> Result<String, String> {
        let client = self.salesforce.as_ref()
            .ok_or_else(|| "Salesforce client not configured".to_string())?;

        let result = client.create(&req.object_type, req.fields).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "salesforce")]
    #[rmcp::tool(description = "Update a record in Salesforce")]
    pub async fn salesforce_update_record(
        &self,
        #[rmcp::tool(aggr)] req: SalesforceUpdateRecordRequest,
    ) -> Result<String, String> {
        let client = self.salesforce.as_ref()
            .ok_or_else(|| "Salesforce client not configured".to_string())?;

        client.update(&req.object_type, &req.record_id, req.fields).await
            .map_err(|e| e.to_string())?;

        Ok(format!("Record {} updated successfully", req.record_id))
    }

    #[cfg(feature = "salesforce")]
    #[rmcp::tool(description = "Get a record from Salesforce")]
    pub async fn salesforce_get_record(
        &self,
        #[rmcp::tool(aggr)] req: SalesforceGetRecordRequest,
    ) -> Result<String, String> {
        let client = self.salesforce.as_ref()
            .ok_or_else(|| "Salesforce client not configured".to_string())?;

        let result = client.get(&req.object_type, &req.record_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&result).map_err(|e| e.to_string())
    }

    #[cfg(feature = "pipedrive")]
    #[rmcp::tool(description = "Create a new person in Pipedrive")]
    pub async fn pipedrive_create_person(
        &self,
        #[rmcp::tool(aggr)] req: PipedriveCreatePersonRequest,
    ) -> Result<String, String> {
        let client = self.pipedrive.as_ref()
            .ok_or_else(|| "Pipedrive client not configured".to_string())?;

        let person = client.create_person(
            &req.name,
            req.email.as_deref(),
            req.phone.as_deref(),
            req.org_id,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&person).map_err(|e| e.to_string())
    }

    #[cfg(feature = "pipedrive")]
    #[rmcp::tool(description = "Create a new deal in Pipedrive")]
    pub async fn pipedrive_create_deal(
        &self,
        #[rmcp::tool(aggr)] req: PipedriveCreateDealRequest,
    ) -> Result<String, String> {
        let client = self.pipedrive.as_ref()
            .ok_or_else(|| "Pipedrive client not configured".to_string())?;

        let deal = client.create_deal(
            &req.title,
            req.value,
            req.currency.as_deref(),
            req.person_id,
            req.org_id,
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&deal).map_err(|e| e.to_string())
    }
}
