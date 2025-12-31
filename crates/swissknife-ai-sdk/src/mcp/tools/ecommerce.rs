use rmcp::{tool_router, handler::server::router::tool::ToolRouter};
use schemars::JsonSchema;
use serde::Deserialize;

#[cfg(feature = "ecommerce")]
use swissknife_ecommerce_sdk as ecommerce;

#[derive(Clone)]
pub struct EcommerceTools {
    #[cfg(feature = "shopify")]
    pub shopify: Option<ecommerce::shopify::ShopifyClient>,
    #[cfg(feature = "woocommerce")]
    pub woocommerce: Option<ecommerce::woocommerce::WooCommerceClient>,
}

impl EcommerceTools {
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "shopify")]
            shopify: None,
            #[cfg(feature = "woocommerce")]
            woocommerce: None,
        }
    }

    #[cfg(feature = "shopify")]
    pub fn with_shopify(mut self, client: ecommerce::shopify::ShopifyClient) -> Self {
        self.shopify = Some(client);
        self
    }

    #[cfg(feature = "woocommerce")]
    pub fn with_woocommerce(mut self, client: ecommerce::woocommerce::WooCommerceClient) -> Self {
        self.woocommerce = Some(client);
        self
    }
}

impl Default for EcommerceTools {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ShopifyListProductsRequest {
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub collection_id: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ShopifyGetProductRequest {
    pub product_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ShopifyCreateProductRequest {
    pub title: String,
    #[serde(default)]
    pub body_html: Option<String>,
    #[serde(default)]
    pub vendor: Option<String>,
    #[serde(default)]
    pub product_type: Option<String>,
    #[serde(default)]
    pub tags: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ShopifyListOrdersRequest {
    #[serde(default)]
    pub limit: Option<u32>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ShopifyGetOrderRequest {
    pub order_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ShopifyListCustomersRequest {
    #[serde(default)]
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WooCommerceListProductsRequest {
    #[serde(default)]
    pub per_page: Option<u32>,
    #[serde(default)]
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WooCommerceGetProductRequest {
    pub product_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WooCommerceCreateProductRequest {
    pub name: String,
    #[serde(default)]
    pub regular_price: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub short_description: Option<String>,
    #[serde(default)]
    pub sku: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WooCommerceListOrdersRequest {
    #[serde(default)]
    pub per_page: Option<u32>,
    #[serde(default)]
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct WooCommerceGetOrderRequest {
    pub order_id: i64,
}

#[tool_router]
impl EcommerceTools {
    #[cfg(feature = "shopify")]
    #[rmcp::tool(description = "List products from Shopify store")]
    pub async fn shopify_list_products(
        &self,
        #[rmcp::tool(aggr)] req: ShopifyListProductsRequest,
    ) -> Result<String, String> {
        let client = self.shopify.as_ref()
            .ok_or_else(|| "Shopify client not configured".to_string())?;

        let products = client.list_products(req.limit, req.collection_id.as_deref()).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&products).map_err(|e| e.to_string())
    }

    #[cfg(feature = "shopify")]
    #[rmcp::tool(description = "Get a Shopify product by ID")]
    pub async fn shopify_get_product(
        &self,
        #[rmcp::tool(aggr)] req: ShopifyGetProductRequest,
    ) -> Result<String, String> {
        let client = self.shopify.as_ref()
            .ok_or_else(|| "Shopify client not configured".to_string())?;

        let product = client.get_product(&req.product_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&product).map_err(|e| e.to_string())
    }

    #[cfg(feature = "shopify")]
    #[rmcp::tool(description = "Create a new product in Shopify")]
    pub async fn shopify_create_product(
        &self,
        #[rmcp::tool(aggr)] req: ShopifyCreateProductRequest,
    ) -> Result<String, String> {
        let client = self.shopify.as_ref()
            .ok_or_else(|| "Shopify client not configured".to_string())?;

        let product = client.create_product(
            &req.title,
            req.body_html.as_deref(),
            req.vendor.as_deref(),
            req.product_type.as_deref(),
            req.tags.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&product).map_err(|e| e.to_string())
    }

    #[cfg(feature = "shopify")]
    #[rmcp::tool(description = "List orders from Shopify store")]
    pub async fn shopify_list_orders(
        &self,
        #[rmcp::tool(aggr)] req: ShopifyListOrdersRequest,
    ) -> Result<String, String> {
        let client = self.shopify.as_ref()
            .ok_or_else(|| "Shopify client not configured".to_string())?;

        let orders = client.list_orders(req.limit, req.status.as_deref()).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&orders).map_err(|e| e.to_string())
    }

    #[cfg(feature = "shopify")]
    #[rmcp::tool(description = "Get a Shopify order by ID")]
    pub async fn shopify_get_order(
        &self,
        #[rmcp::tool(aggr)] req: ShopifyGetOrderRequest,
    ) -> Result<String, String> {
        let client = self.shopify.as_ref()
            .ok_or_else(|| "Shopify client not configured".to_string())?;

        let order = client.get_order(&req.order_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&order).map_err(|e| e.to_string())
    }

    #[cfg(feature = "shopify")]
    #[rmcp::tool(description = "List customers from Shopify store")]
    pub async fn shopify_list_customers(
        &self,
        #[rmcp::tool(aggr)] req: ShopifyListCustomersRequest,
    ) -> Result<String, String> {
        let client = self.shopify.as_ref()
            .ok_or_else(|| "Shopify client not configured".to_string())?;

        let customers = client.list_customers(req.limit).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&customers).map_err(|e| e.to_string())
    }

    #[cfg(feature = "woocommerce")]
    #[rmcp::tool(description = "List products from WooCommerce store")]
    pub async fn woocommerce_list_products(
        &self,
        #[rmcp::tool(aggr)] req: WooCommerceListProductsRequest,
    ) -> Result<String, String> {
        let client = self.woocommerce.as_ref()
            .ok_or_else(|| "WooCommerce client not configured".to_string())?;

        let products = client.list_products(req.per_page, req.category.as_deref()).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&products).map_err(|e| e.to_string())
    }

    #[cfg(feature = "woocommerce")]
    #[rmcp::tool(description = "Get a WooCommerce product by ID")]
    pub async fn woocommerce_get_product(
        &self,
        #[rmcp::tool(aggr)] req: WooCommerceGetProductRequest,
    ) -> Result<String, String> {
        let client = self.woocommerce.as_ref()
            .ok_or_else(|| "WooCommerce client not configured".to_string())?;

        let product = client.get_product(req.product_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&product).map_err(|e| e.to_string())
    }

    #[cfg(feature = "woocommerce")]
    #[rmcp::tool(description = "Create a new product in WooCommerce")]
    pub async fn woocommerce_create_product(
        &self,
        #[rmcp::tool(aggr)] req: WooCommerceCreateProductRequest,
    ) -> Result<String, String> {
        let client = self.woocommerce.as_ref()
            .ok_or_else(|| "WooCommerce client not configured".to_string())?;

        let product = client.create_product(
            &req.name,
            req.regular_price.as_deref(),
            req.description.as_deref(),
            req.short_description.as_deref(),
            req.sku.as_deref(),
        ).await.map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&product).map_err(|e| e.to_string())
    }

    #[cfg(feature = "woocommerce")]
    #[rmcp::tool(description = "List orders from WooCommerce store")]
    pub async fn woocommerce_list_orders(
        &self,
        #[rmcp::tool(aggr)] req: WooCommerceListOrdersRequest,
    ) -> Result<String, String> {
        let client = self.woocommerce.as_ref()
            .ok_or_else(|| "WooCommerce client not configured".to_string())?;

        let orders = client.list_orders(req.per_page, req.status.as_deref()).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&orders).map_err(|e| e.to_string())
    }

    #[cfg(feature = "woocommerce")]
    #[rmcp::tool(description = "Get a WooCommerce order by ID")]
    pub async fn woocommerce_get_order(
        &self,
        #[rmcp::tool(aggr)] req: WooCommerceGetOrderRequest,
    ) -> Result<String, String> {
        let client = self.woocommerce.as_ref()
            .ok_or_else(|| "WooCommerce client not configured".to_string())?;

        let order = client.get_order(req.order_id).await
            .map_err(|e| e.to_string())?;

        serde_json::to_string_pretty(&order).map_err(|e| e.to_string())
    }
}
