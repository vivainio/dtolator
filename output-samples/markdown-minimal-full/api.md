# E-Commerce API

## Users

`GET` /users (page?: integer, limit?: integer) → `UserListResponse`

```typescript
// Paginated list of users
type UserListResponse {
  data: User[]
  pagination: PaginationInfo
}

// Pagination metadata for list responses
type PaginationInfo {
  page: integer
  limit: integer
  total: integer
  totalPages: integer
  hasNext?: boolean
  hasPrev?: boolean
}

// Registered user account
type User {
  id: string  // uuid, Unique user identifier
  email: string  // email, User's email address
  profile: UserProfile
  preferences?: UserPreferences
  createdAt?: string  // date-time
  updatedAt?: string  // date-time
  isActive?: boolean
  roles?: UserRole[]
}

// User-configurable application preferences
type UserPreferences {
  language?: "en" | "es" | "fr" | "de" | "it"
  currency?: "USD" | "EUR" | "GBP" | "JPY"
  notifications?: NotificationSettings
  theme?: "light" | "dark" | "auto"
}

// Personal profile information for a user
type UserProfile {
  firstName: string
  lastName: string
  dateOfBirth?: string | null  // date
  phoneNumber?: string | null
  avatar?: ImageUrl
  address?: Address
}

// Role assigned to a user account
enum UserRole = "customer" | "admin" | "moderator" | "vendor"

// Per-channel notification opt-in settings
type NotificationSettings {
  email?: boolean
  push?: boolean
  sms?: boolean
  marketing?: boolean
}

// Physical mailing or shipping address.
// Used for both billing and delivery purposes.
type Address {
  street: string
  street2?: string | null
  city: string
  state?: string | null
  country: string  // ISO 3166-1 alpha-2 country code
  postalCode: string
}

// Image with URL and optional metadata
type ImageUrl {
  url: string  // uri, Image URL
  alt?: string  // Alternative text for the image
  width?: integer
  height?: integer
}

// Standard error response envelope
type ErrorResponse {
  error: object
}

```

`POST` /users `CreateUserRequest` → `User`

```typescript
// Request body for creating a new user account
type CreateUserRequest {
  email: string  // email
  password: string
  profile: UserProfile
  preferences?: UserPreferences
}

```

`GET` /users/{userId} → `User`

## Products

`GET` /products (category?: ProductCategory, minPrice?: number, maxPrice?: number) → `ProductListResponse`

```typescript
// Paginated list of products with optional filters
type ProductListResponse {
  data: Product[]
  pagination: PaginationInfo
  filters?: object
}

// A product listed in the catalog
type Product {
  id: string  // uuid
  name: string
  description?: string | null
  price: Price
  category: ProductCategory
  tags?: string[]
  images?: ImageUrl[]
  inventory?: Inventory
  specifications?: Map<string, string | number | boolean>
  isActive?: boolean
  createdAt?: string  // date-time
}

// Top-level product category
enum ProductCategory = "electronics" | "clothing" | "home" | "books" | "sports" | "beauty" | "automotive"

// Stock level and availability for a product
type Inventory {
  quantity: integer
  status: "in_stock" | "low_stock" | "out_of_stock" | "discontinued"
  lowStockThreshold?: integer
}

// Monetary amount with currency
type Price {
  amount: number
  currency: "USD" | "EUR" | "GBP" | "JPY"
  originalAmount?: number | null  // Original price before discount
}

```

`GET` /products/{productId} → `Product`

`PUT` /products/{productId} `UpdateProductRequest` → `Product`

```typescript
// Request body for partially updating a product
type UpdateProductRequest {
  name?: string
  description?: string
  price?: Price
  category?: ProductCategory
  isActive?: boolean
}

```

## Orders

`POST` /orders `CreateOrderRequest` → `Order`

```typescript
// Request body for placing a new order
type CreateOrderRequest {
  items: object[]
  shippingAddress: Address
  billingAddress?: Address
  paymentMethod?: PaymentMethod
}

// Payment instrument used for an order
type PaymentMethod {
  type: "credit_card" | "debit_card" | "paypal" | "bank_transfer" | "crypto"
  last4?: string  // Last 4 digits of card (for card payments)
  brand?: "visa" | "mastercard" | "amex" | "discover"  // Card brand (for card payments)
}

// A customer purchase order
type Order {
  id: string  // uuid
  userId: string  // uuid
  items: OrderItem[]
  total: Price
  status: OrderStatus
  shippingAddress?: Address
  billingAddress?: Address
  paymentMethod?: PaymentMethod
  orderDate?: string  // date-time
  estimatedDelivery?: string | null  // date
  trackingNumber?: string | null
}

// A single line item within an order
type OrderItem {
  productId: string  // uuid
  quantity: integer
  price: Price
  productSnapshot?: Product  // Snapshot of product at time of order
}

// Current lifecycle status of an order
enum OrderStatus = "pending" | "confirmed" | "processing" | "shipped" | "delivered" | "cancelled" | "refunded"

```

`GET` /orders/{orderId} → `Order`

`PATCH` /orders/{orderId} `UpdateOrderStatusRequest` → `Order`

```typescript
// Request body for updating an order's status
type UpdateOrderStatusRequest {
  status: OrderStatus
  trackingNumber?: string
}

```

## Categories

`GET` /categories → `Category[]`

```typescript
// Product category with optional hierarchy
type Category {
  id: string  // uuid
  name: string
  slug: string
  description?: string
  parentId?: string | null  // uuid
  isActive?: boolean
}

```

`POST` /categories `CreateCategoryRequest` → `Category`

```typescript
// Request body for creating a new category
type CreateCategoryRequest {
  name: string
  slug: string
  description?: string
  parentId?: string  // uuid
}

```

## Inventory

`GET` /inventory (lowStock?: boolean) → `InventoryResponse`

```typescript
// Inventory levels for a list of products
type InventoryResponse {
  data: object[]
}

```

`PUT` /inventory/{productId} `UpdateInventoryRequest` → `Inventory`

```typescript
// Request body for updating product inventory
type UpdateInventoryRequest {
  quantity: integer
  lowStockThreshold?: integer
}

```

## Analytics

`GET` /analytics/sales (startDate?: string, endDate?: string) → `SalesAnalytics`

```typescript
// Aggregated sales metrics for a given period
type SalesAnalytics {
  totalRevenue: number
  totalOrders: integer
  averageOrderValue: number
  topProducts?: object[]
  period?: object
}

```

`GET` /analytics/products → `ProductAnalytics`

```typescript
// Catalog statistics and stock health metrics
type ProductAnalytics {
  totalProducts: integer
  activeProducts: integer
  categoryBreakdown?: Map<string, integer>
  lowStockProducts?: object[]
}

```

