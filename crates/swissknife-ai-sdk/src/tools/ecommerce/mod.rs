use crate::error::Result;
use crate::tool::{get_i64_param, get_required_string_param, get_string_param, Tool};
use crate::types::{OutputSchema, ParameterSchema, ToolDefinition, ToolResponse};
use async_trait::async_trait;
use std::collections::HashMap;
use swissknife_ecommerce_sdk::{EcommerceProvider, ListOptions, ProductFilter, OrderFilter};

pub struct ListProductsTool;

impl Default for ListProductsTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ListProductsTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "ecommerce_list_products",
            "List Products",
            "List products from an e-commerce platform (Shopify, WooCommerce)",
            "ecommerce",
        )
        .with_param("api_key", ParameterSchema::string("API key or access token").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: shopify, woocommerce").required())
        .with_param("store_url", ParameterSchema::string("Store URL").required())
        .with_param("status", ParameterSchema::string("Product status: active, draft, archived"))
        .with_param("limit", ParameterSchema::integer("Maximum products to return"))
        .with_output("products", OutputSchema::array("List of products", OutputSchema::json("Product")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let store_url = get_required_string_param(&params, "store_url")?;
        let _status = get_string_param(&params, "status");
        let limit = get_i64_param(&params, "limit").map(|v| v as u32);

        let filter = ProductFilter::default();
        let options = ListOptions {
            limit,
            ..Default::default()
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "shopify")]
            "shopify" => {
                use swissknife_ecommerce_sdk::shopify::ShopifyClient;
                let client = ShopifyClient::new(&store_url, &api_key);
                client.list_products(&filter, &options).await
            }
            #[cfg(feature = "woocommerce")]
            "woocommerce" => {
                use swissknife_ecommerce_sdk::woocommerce::WooCommerceClient;
                let client = WooCommerceClient::new(&store_url, &api_key, "");
                client.list_products(&filter, &options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported e-commerce provider: {}", provider)));
            }
        };

        match result {
            Ok(products) => Ok(ToolResponse::success(serde_json::json!({
                "products": products.items.iter().map(|p| serde_json::json!({
                    "id": p.id,
                    "title": p.title,
                    "description": p.description,
                    "status": format!("{:?}", p.status),
                    "vendor": p.vendor,
                    "variants": p.variants.iter().map(|v| serde_json::json!({
                        "id": v.id,
                        "title": v.title,
                        "sku": v.sku,
                        "price": v.price,
                        "inventory_quantity": v.inventory_quantity,
                    })).collect::<Vec<_>>(),
                })).collect::<Vec<_>>(),
                "has_more": products.has_more,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to list products: {}", e))),
        }
    }
}

pub struct GetProductTool;

impl Default for GetProductTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GetProductTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "ecommerce_get_product",
            "Get Product",
            "Get a product by ID from an e-commerce platform",
            "ecommerce",
        )
        .with_param("api_key", ParameterSchema::string("API key or access token").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: shopify, woocommerce").required())
        .with_param("store_url", ParameterSchema::string("Store URL").required())
        .with_param("product_id", ParameterSchema::string("Product ID").required())
        .with_output("product", OutputSchema::json("Product details"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let store_url = get_required_string_param(&params, "store_url")?;
        let product_id = get_required_string_param(&params, "product_id")?;

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "shopify")]
            "shopify" => {
                use swissknife_ecommerce_sdk::shopify::ShopifyClient;
                let client = ShopifyClient::new(&store_url, &api_key);
                client.get_product(&product_id).await
            }
            #[cfg(feature = "woocommerce")]
            "woocommerce" => {
                use swissknife_ecommerce_sdk::woocommerce::WooCommerceClient;
                let client = WooCommerceClient::new(&store_url, &api_key, "");
                client.get_product(&product_id).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported e-commerce provider: {}", provider)));
            }
        };

        match result {
            Ok(product) => Ok(ToolResponse::success(serde_json::json!({
                "product": {
                    "id": product.id,
                    "title": product.title,
                    "description": product.description,
                    "status": format!("{:?}", product.status),
                    "vendor": product.vendor,
                    "product_type": product.product_type,
                    "tags": product.tags,
                    "variants": product.variants,
                    "images": product.images,
                }
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to get product: {}", e))),
        }
    }
}

pub struct ListOrdersTool;

impl Default for ListOrdersTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ListOrdersTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "ecommerce_list_orders",
            "List Orders",
            "List orders from an e-commerce platform",
            "ecommerce",
        )
        .with_param("api_key", ParameterSchema::string("API key or access token").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: shopify, woocommerce").required())
        .with_param("store_url", ParameterSchema::string("Store URL").required())
        .with_param("status", ParameterSchema::string("Order status: open, closed, cancelled"))
        .with_param("limit", ParameterSchema::integer("Maximum orders to return"))
        .with_output("orders", OutputSchema::array("List of orders", OutputSchema::json("Order")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let store_url = get_required_string_param(&params, "store_url")?;
        let _status = get_string_param(&params, "status");
        let limit = get_i64_param(&params, "limit").map(|v| v as u32);

        let filter = OrderFilter::default();
        let options = ListOptions {
            limit,
            ..Default::default()
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "shopify")]
            "shopify" => {
                use swissknife_ecommerce_sdk::shopify::ShopifyClient;
                let client = ShopifyClient::new(&store_url, &api_key);
                client.list_orders(&filter, &options).await
            }
            #[cfg(feature = "woocommerce")]
            "woocommerce" => {
                use swissknife_ecommerce_sdk::woocommerce::WooCommerceClient;
                let client = WooCommerceClient::new(&store_url, &api_key, "");
                client.list_orders(&filter, &options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported e-commerce provider: {}", provider)));
            }
        };

        match result {
            Ok(orders) => Ok(ToolResponse::success(serde_json::json!({
                "orders": orders.items.iter().map(|o| serde_json::json!({
                    "id": o.id,
                    "order_number": o.order_number,
                    "email": o.email,
                    "status": format!("{:?}", o.status),
                    "financial_status": format!("{:?}", o.financial_status),
                    "fulfillment_status": o.fulfillment_status.map(|s| format!("{:?}", s)),
                    "total_price": o.total_price,
                    "currency": o.currency,
                    "line_items_count": o.line_items.len(),
                    "created_at": o.created_at.to_rfc3339(),
                })).collect::<Vec<_>>(),
                "has_more": orders.has_more,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to list orders: {}", e))),
        }
    }
}

pub struct GetOrderTool;

impl Default for GetOrderTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for GetOrderTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "ecommerce_get_order",
            "Get Order",
            "Get an order by ID from an e-commerce platform",
            "ecommerce",
        )
        .with_param("api_key", ParameterSchema::string("API key or access token").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: shopify, woocommerce").required())
        .with_param("store_url", ParameterSchema::string("Store URL").required())
        .with_param("order_id", ParameterSchema::string("Order ID").required())
        .with_output("order", OutputSchema::json("Order details"))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let store_url = get_required_string_param(&params, "store_url")?;
        let order_id = get_required_string_param(&params, "order_id")?;

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "shopify")]
            "shopify" => {
                use swissknife_ecommerce_sdk::shopify::ShopifyClient;
                let client = ShopifyClient::new(&store_url, &api_key);
                client.get_order(&order_id).await
            }
            #[cfg(feature = "woocommerce")]
            "woocommerce" => {
                use swissknife_ecommerce_sdk::woocommerce::WooCommerceClient;
                let client = WooCommerceClient::new(&store_url, &api_key, "");
                client.get_order(&order_id).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported e-commerce provider: {}", provider)));
            }
        };

        match result {
            Ok(order) => Ok(ToolResponse::success(serde_json::json!({
                "order": {
                    "id": order.id,
                    "order_number": order.order_number,
                    "email": order.email,
                    "status": format!("{:?}", order.status),
                    "financial_status": format!("{:?}", order.financial_status),
                    "fulfillment_status": order.fulfillment_status.map(|s| format!("{:?}", s)),
                    "subtotal_price": order.subtotal_price,
                    "total_tax": order.total_tax,
                    "total_discounts": order.total_discounts,
                    "total_price": order.total_price,
                    "currency": order.currency,
                    "line_items": order.line_items,
                    "shipping_address": order.shipping_address,
                    "billing_address": order.billing_address,
                    "customer": order.customer,
                    "note": order.note,
                }
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to get order: {}", e))),
        }
    }
}

pub struct ListCustomersTool;

impl Default for ListCustomersTool {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl Tool for ListCustomersTool {
    fn definition(&self) -> ToolDefinition {
        ToolDefinition::new(
            "ecommerce_list_customers",
            "List Customers",
            "List customers from an e-commerce platform",
            "ecommerce",
        )
        .with_param("api_key", ParameterSchema::string("API key or access token").required().user_only())
        .with_param("provider", ParameterSchema::string("Provider: shopify, woocommerce").required())
        .with_param("store_url", ParameterSchema::string("Store URL").required())
        .with_param("limit", ParameterSchema::integer("Maximum customers to return"))
        .with_output("customers", OutputSchema::array("List of customers", OutputSchema::json("Customer")))
    }

    async fn execute(&self, params: HashMap<String, serde_json::Value>) -> Result<ToolResponse> {
        let api_key = get_required_string_param(&params, "api_key")?;
        let provider = get_required_string_param(&params, "provider")?;
        let store_url = get_required_string_param(&params, "store_url")?;
        let limit = get_i64_param(&params, "limit").map(|v| v as u32);

        let options = ListOptions {
            limit,
            ..Default::default()
        };

        let result = match provider.to_lowercase().as_str() {
            #[cfg(feature = "shopify")]
            "shopify" => {
                use swissknife_ecommerce_sdk::shopify::ShopifyClient;
                let client = ShopifyClient::new(&store_url, &api_key);
                client.list_customers(&options).await
            }
            #[cfg(feature = "woocommerce")]
            "woocommerce" => {
                use swissknife_ecommerce_sdk::woocommerce::WooCommerceClient;
                let client = WooCommerceClient::new(&store_url, &api_key, "");
                client.list_customers(&options).await
            }
            _ => {
                return Ok(ToolResponse::error(format!("Unsupported e-commerce provider: {}", provider)));
            }
        };

        match result {
            Ok(customers) => Ok(ToolResponse::success(serde_json::json!({
                "customers": customers.items.iter().map(|c| serde_json::json!({
                    "id": c.id,
                    "email": c.email,
                    "first_name": c.first_name,
                    "last_name": c.last_name,
                    "phone": c.phone,
                    "orders_count": c.orders_count,
                    "total_spent": c.total_spent,
                    "tags": c.tags,
                })).collect::<Vec<_>>(),
                "has_more": customers.has_more,
            }))),
            Err(e) => Ok(ToolResponse::error(format!("Failed to list customers: {}", e))),
        }
    }
}
