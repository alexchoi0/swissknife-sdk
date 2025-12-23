mod error;

pub use error::{Error, Result};

#[cfg(feature = "shopify")]
pub mod shopify;

#[cfg(feature = "woocommerce")]
pub mod woocommerce;

#[cfg(feature = "bigcommerce")]
pub mod bigcommerce;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub handle: Option<String>,
    pub status: ProductStatus,
    pub vendor: Option<String>,
    pub product_type: Option<String>,
    pub tags: Vec<String>,
    pub variants: Vec<ProductVariant>,
    pub images: Vec<ProductImage>,
    pub options: Vec<ProductOption>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProductStatus {
    Active,
    Draft,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductVariant {
    pub id: String,
    pub title: String,
    pub sku: Option<String>,
    pub price: f64,
    pub compare_at_price: Option<f64>,
    pub inventory_quantity: Option<i32>,
    pub weight: Option<f64>,
    pub weight_unit: Option<String>,
    pub options: HashMap<String, String>,
    pub barcode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductImage {
    pub id: String,
    pub src: String,
    pub alt: Option<String>,
    pub position: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductOption {
    pub name: String,
    pub values: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub order_number: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub status: OrderStatus,
    pub financial_status: FinancialStatus,
    pub fulfillment_status: Option<FulfillmentStatus>,
    pub currency: String,
    pub subtotal_price: f64,
    pub total_tax: f64,
    pub total_discounts: f64,
    pub total_price: f64,
    pub line_items: Vec<LineItem>,
    pub shipping_address: Option<Address>,
    pub billing_address: Option<Address>,
    pub customer: Option<Customer>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Open,
    Closed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FinancialStatus {
    Pending,
    Authorized,
    PartiallyPaid,
    Paid,
    PartiallyRefunded,
    Refunded,
    Voided,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FulfillmentStatus {
    Unfulfilled,
    Partial,
    Fulfilled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineItem {
    pub id: String,
    pub product_id: Option<String>,
    pub variant_id: Option<String>,
    pub title: String,
    pub quantity: u32,
    pub price: f64,
    pub sku: Option<String>,
    pub total_discount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub company: Option<String>,
    pub address1: Option<String>,
    pub address2: Option<String>,
    pub city: Option<String>,
    pub province: Option<String>,
    pub province_code: Option<String>,
    pub country: Option<String>,
    pub country_code: Option<String>,
    pub zip: Option<String>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: String,
    pub email: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
    pub orders_count: Option<u32>,
    pub total_spent: Option<f64>,
    pub tags: Vec<String>,
    pub addresses: Vec<Address>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: String,
    pub sku: Option<String>,
    pub inventory_quantity: i32,
    pub location_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub handle: Option<String>,
    pub image: Option<ProductImage>,
    pub products_count: Option<u32>,
}

#[derive(Debug, Clone, Default)]
pub struct ListOptions {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ListResult<T> {
    pub items: Vec<T>,
    pub has_more: bool,
    pub next_cursor: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProductFilter {
    pub status: Option<ProductStatus>,
    pub vendor: Option<String>,
    pub product_type: Option<String>,
    pub collection_id: Option<String>,
    pub ids: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct OrderFilter {
    pub status: Option<OrderStatus>,
    pub financial_status: Option<FinancialStatus>,
    pub fulfillment_status: Option<FulfillmentStatus>,
    pub created_at_min: Option<DateTime<Utc>>,
    pub created_at_max: Option<DateTime<Utc>>,
}

#[async_trait]
pub trait EcommerceProvider: Send + Sync {
    async fn list_products(&self, filter: &ProductFilter, options: &ListOptions) -> Result<ListResult<Product>>;
    async fn get_product(&self, id: &str) -> Result<Product>;
    async fn create_product(&self, product: &Product) -> Result<Product>;
    async fn update_product(&self, id: &str, product: &Product) -> Result<Product>;
    async fn delete_product(&self, id: &str) -> Result<()>;

    async fn list_orders(&self, filter: &OrderFilter, options: &ListOptions) -> Result<ListResult<Order>>;
    async fn get_order(&self, id: &str) -> Result<Order>;
    async fn create_order(&self, order: &Order) -> Result<Order>;
    async fn update_order(&self, id: &str, order: &Order) -> Result<Order>;
    async fn cancel_order(&self, id: &str) -> Result<Order>;

    async fn list_customers(&self, options: &ListOptions) -> Result<ListResult<Customer>>;
    async fn get_customer(&self, id: &str) -> Result<Customer>;
    async fn create_customer(&self, customer: &Customer) -> Result<Customer>;
    async fn update_customer(&self, id: &str, customer: &Customer) -> Result<Customer>;
    async fn delete_customer(&self, id: &str) -> Result<()>;
}

#[async_trait]
pub trait InventoryProvider: Send + Sync {
    async fn get_inventory(&self, item_id: &str, location_id: Option<&str>) -> Result<InventoryItem>;
    async fn adjust_inventory(&self, item_id: &str, location_id: &str, adjustment: i32) -> Result<InventoryItem>;
    async fn set_inventory(&self, item_id: &str, location_id: &str, quantity: i32) -> Result<InventoryItem>;
}

#[async_trait]
pub trait CollectionProvider: Send + Sync {
    async fn list_collections(&self, options: &ListOptions) -> Result<ListResult<Collection>>;
    async fn get_collection(&self, id: &str) -> Result<Collection>;
    async fn create_collection(&self, title: &str, description: Option<&str>) -> Result<Collection>;
    async fn add_products_to_collection(&self, collection_id: &str, product_ids: &[&str]) -> Result<()>;
    async fn remove_products_from_collection(&self, collection_id: &str, product_ids: &[&str]) -> Result<()>;
}
