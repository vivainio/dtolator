use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub street: String,
    pub street2: Option<String>,
    pub city: String,
    pub state: Option<String>,
    pub country: String,
    #[serde(rename = "postalCode")]
    pub postal_code: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateCategoryRequest {
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    #[serde(rename = "parentId")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorResponse {
    pub error: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ImageUrl {
    pub url: String,
    pub alt: Option<String>,
    pub width: Option<i64>,
    pub height: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Inventory {
    pub quantity: i64,
    pub status: String,
    #[serde(rename = "lowStockThreshold")]
    pub low_stock_threshold: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub email: Option<bool>,
    pub push: Option<bool>,
    pub sms: Option<bool>,
    pub marketing: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    #[serde(rename = "pending")]
    Pending,
    #[serde(rename = "confirmed")]
    Confirmed,
    #[serde(rename = "processing")]
    Processing,
    #[serde(rename = "shipped")]
    Shipped,
    #[serde(rename = "delivered")]
    Delivered,
    #[serde(rename = "cancelled")]
    Cancelled,
    #[serde(rename = "refunded")]
    Refunded,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaginationInfo {
    pub page: i64,
    pub limit: i64,
    pub total: i64,
    #[serde(rename = "totalPages")]
    pub total_pages: i64,
    #[serde(rename = "hasNext")]
    pub has_next: Option<bool>,
    #[serde(rename = "hasPrev")]
    pub has_prev: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMethod {
    pub type: String,
    pub last4: Option<String>,
    pub brand: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Price {
    pub amount: f64,
    pub currency: String,
    #[serde(rename = "originalAmount")]
    pub original_amount: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProductAnalytics {
    #[serde(rename = "totalProducts")]
    pub total_products: i64,
    #[serde(rename = "activeProducts")]
    pub active_products: i64,
    #[serde(rename = "categoryBreakdown")]
    pub category_breakdown: Option<serde_json::Value>,
    #[serde(rename = "lowStockProducts")]
    pub low_stock_products: Option<Vec<serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProductCategory {
    #[serde(rename = "electronics")]
    Electronics,
    #[serde(rename = "clothing")]
    Clothing,
    #[serde(rename = "home")]
    Home,
    #[serde(rename = "books")]
    Books,
    #[serde(rename = "sports")]
    Sports,
    #[serde(rename = "beauty")]
    Beauty,
    #[serde(rename = "automotive")]
    Automotive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SalesAnalytics {
    #[serde(rename = "totalRevenue")]
    pub total_revenue: f64,
    #[serde(rename = "totalOrders")]
    pub total_orders: i64,
    #[serde(rename = "averageOrderValue")]
    pub average_order_value: f64,
    #[serde(rename = "topProducts")]
    pub top_products: Option<Vec<serde_json::Value>>,
    pub period: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInventoryRequest {
    pub quantity: i64,
    #[serde(rename = "lowStockThreshold")]
    pub low_stock_threshold: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum UserRole {
    #[serde(rename = "customer")]
    Customer,
    #[serde(rename = "admin")]
    Admin,
    #[serde(rename = "moderator")]
    Moderator,
    #[serde(rename = "vendor")]
    Vendor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(rename = "lastName")]
    pub last_name: String,
    #[serde(rename = "dateOfBirth")]
    pub date_of_birth: Option<String>,
    #[serde(rename = "phoneNumber")]
    pub phone_number: Option<String>,
    pub avatar: Option<ImageUrl>,
    pub address: Option<Address>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InventoryResponse {
    pub data: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserPreferences {
    pub language: Option<String>,
    pub currency: Option<String>,
    pub notifications: Option<NotificationSettings>,
    pub theme: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateOrderStatusRequest {
    pub status: OrderStatus,
    #[serde(rename = "trackingNumber")]
    pub tracking_number: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateOrderRequest {
    pub items: Vec<serde_json::Value>,
    #[serde(rename = "shippingAddress")]
    pub shipping_address: Address,
    #[serde(rename = "billingAddress")]
    pub billing_address: Option<Address>,
    #[serde(rename = "paymentMethod")]
    pub payment_method: Option<PaymentMethod>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Product {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub price: Price,
    pub category: ProductCategory,
    pub tags: Option<Vec<String>>,
    pub images: Option<Vec<ImageUrl>>,
    pub inventory: Option<Inventory>,
    pub specifications: Option<serde_json::Value>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProductRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub price: Option<Price>,
    pub category: Option<ProductCategory>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub profile: UserProfile,
    pub preferences: Option<UserPreferences>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: String,
    pub email: String,
    pub profile: UserProfile,
    pub preferences: Option<UserPreferences>,
    #[serde(rename = "createdAt")]
    pub created_at: Option<String>,
    #[serde(rename = "updatedAt")]
    pub updated_at: Option<String>,
    #[serde(rename = "isActive")]
    pub is_active: Option<bool>,
    pub roles: Option<Vec<UserRole>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct OrderItem {
    #[serde(rename = "productId")]
    pub product_id: String,
    pub quantity: i64,
    pub price: Price,
    #[serde(rename = "productSnapshot")]
    pub product_snapshot: Option<Product>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProductListResponse {
    pub data: Vec<Product>,
    pub pagination: PaginationInfo,
    pub filters: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UserListResponse {
    pub data: Vec<User>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    pub id: String,
    #[serde(rename = "userId")]
    pub user_id: String,
    pub items: Vec<OrderItem>,
    pub total: Price,
    pub status: OrderStatus,
    #[serde(rename = "shippingAddress")]
    pub shipping_address: Option<Address>,
    #[serde(rename = "billingAddress")]
    pub billing_address: Option<Address>,
    #[serde(rename = "paymentMethod")]
    pub payment_method: Option<PaymentMethod>,
    #[serde(rename = "orderDate")]
    pub order_date: Option<String>,
    #[serde(rename = "estimatedDelivery")]
    pub estimated_delivery: Option<String>,
    #[serde(rename = "trackingNumber")]
    pub tracking_number: Option<String>,
}


