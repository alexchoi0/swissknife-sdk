use crate::error::{Error, Result};
use crate::tool::Tool;
use crate::types::{ToolSpec, ToolOutput};
use std::collections::HashMap;
use std::sync::Arc;

pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: Tool + 'static>(&mut self, tool: T) -> &mut Self {
        let id = tool.id();
        self.tools.insert(id, Arc::new(tool));
        self
    }

    pub fn register_arc(&mut self, tool: Arc<dyn Tool>) -> &mut Self {
        let id = tool.id();
        self.tools.insert(id, tool);
        self
    }

    pub fn get(&self, id: &str) -> Option<Arc<dyn Tool>> {
        self.tools.get(id).cloned()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.tools.contains_key(id)
    }

    pub fn remove(&mut self, id: &str) -> Option<Arc<dyn Tool>> {
        self.tools.remove(id)
    }

    pub fn list(&self) -> Vec<&str> {
        self.tools.keys().map(|s| s.as_str()).collect()
    }

    pub fn list_by_category(&self, category: &str) -> Vec<&str> {
        self.tools
            .iter()
            .filter(|(_, tool)| tool.category() == category)
            .map(|(id, _)| id.as_str())
            .collect()
    }

    pub fn categories(&self) -> Vec<String> {
        let mut categories: Vec<_> = self
            .tools
            .values()
            .map(|t| t.category())
            .collect();
        categories.sort();
        categories.dedup();
        categories
    }

    pub fn definitions(&self) -> Vec<ToolSpec> {
        self.tools.values().map(|t| t.definition()).collect()
    }

    pub fn definitions_by_category(&self, category: &str) -> Vec<ToolSpec> {
        self.tools
            .values()
            .filter(|t| t.category() == category)
            .map(|t| t.definition())
            .collect()
    }

    pub async fn execute(
        &self,
        tool_id: &str,
        params: HashMap<String, serde_json::Value>,
    ) -> Result<ToolOutput> {
        let tool = self.get(tool_id).ok_or_else(|| Error::ToolNotFound(tool_id.to_string()))?;
        tool.execute(params).await
    }

    pub fn to_openai_functions(&self) -> Vec<serde_json::Value> {
        self.definitions()
            .iter()
            .map(|d| d.to_openai_function())
            .collect()
    }

    pub fn to_anthropic_tools(&self) -> Vec<serde_json::Value> {
        self.definitions()
            .iter()
            .map(|d| d.to_anthropic_tool())
            .collect()
    }

    pub fn len(&self) -> usize {
        self.tools.len()
    }

    pub fn is_empty(&self) -> bool {
        self.tools.is_empty()
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "payments")]
impl ToolRegistry {
    pub fn with_payment_tools(mut self) -> Self {
        use crate::tools::payments::*;

        #[cfg(feature = "stripe")]
        {
            self.register(StripeCreateCustomerTool::default());
            self.register(StripeGetCustomerTool::default());
            self.register(StripeChargeTool::default());
            self.register(StripeRefundTool::default());
        }

        self
    }
}

#[cfg(feature = "crm")]
impl ToolRegistry {
    pub fn with_crm_tools(mut self) -> Self {
        use crate::tools::crm::*;

        #[cfg(feature = "salesforce")]
        {
            self.register(SalesforceCreateContactTool::default());
            self.register(SalesforceGetContactTool::default());
            self.register(SalesforceCreateDealTool::default());
        }

        #[cfg(feature = "hubspot")]
        {
            self.register(HubSpotCreateContactTool::default());
            self.register(HubSpotGetContactTool::default());
        }

        self
    }
}

#[cfg(feature = "communication")]
impl ToolRegistry {
    pub fn with_communication_tools(mut self) -> Self {
        use crate::tools::communication::*;

        #[cfg(feature = "twilio")]
        {
            self.register(TwilioSendSmsTool::default());
        }

        #[cfg(feature = "sendgrid")]
        {
            self.register(SendGridSendEmailTool::default());
        }

        #[cfg(feature = "resend")]
        {
            self.register(ResendSendEmailTool::default());
        }

        self
    }
}

#[cfg(feature = "social")]
impl ToolRegistry {
    pub fn with_social_tools(mut self) -> Self {
        use crate::tools::social::*;

        #[cfg(feature = "slack")]
        {
            self.register(SlackSendMessageTool::default());
            self.register(SlackListChannelsTool::default());
        }

        #[cfg(feature = "discord")]
        {
            self.register(DiscordSendMessageTool::default());
        }

        self
    }
}

#[cfg(feature = "hr")]
impl ToolRegistry {
    pub fn with_hr_tools(mut self) -> Self {
        use crate::tools::hr::*;

        #[cfg(feature = "bamboohr")]
        {
            self.register(BambooHRListEmployeesTool::default());
            self.register(BambooHRGetEmployeeTool::default());
        }

        self
    }
}

#[cfg(feature = "banking")]
impl ToolRegistry {
    pub fn with_banking_tools(mut self) -> Self {
        use crate::tools::banking::*;

        #[cfg(feature = "plaid")]
        {
            self.register(PlaidGetAccountsTool::default());
            self.register(PlaidGetTransactionsTool::default());
        }

        self
    }
}
