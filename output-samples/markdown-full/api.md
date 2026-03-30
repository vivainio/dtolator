# E-Commerce API

**Version:** 2.1.0

A comprehensive e-commerce API with user management, product catalog, and order processing

## Users

<details>
<summary><code>GET</code> /users — <strong>Get All Users With Pagination</strong></summary>

Retrieve a paginated list of users.
Supports cursor-based and offset pagination.
Results are sorted by creation date by default.

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `page` | query | `integer` |  |  |
| `limit` | query | `integer` |  |  |

**Responses:**

- **200**: Successful response → `UserListResponse`
- **400**: Bad request → `ErrorResponse`

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

</details>

<details>
<summary><code>POST</code> /users — <strong>Create New User Account</strong></summary>

Create a new user account.

The email address must be unique across all accounts.
A verification email is sent upon successful creation.

**Request body:** `CreateUserRequest`

**Responses:**

- **201**: User created successfully → `User`
- **400**: Bad request → `ErrorResponse`

```typescript
// Request body for creating a new user account
type CreateUserRequest {
  email: string  // email
  password: string
  profile: UserProfile
  preferences?: UserPreferences
}

```

</details>

<details>
<summary><code>GET</code> /users/{userId} — <strong>Get User Profile By ID</strong></summary>

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `userId` | path | `string` | yes |  |

**Responses:**

- **200**: User found → `User`
- **404**: User not found → `ErrorResponse`

</details>

## Products

<details>
<summary><code>GET</code> /products — <strong>Search Products With Filters</strong></summary>

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `category` | query | `ProductCategory` |  |  |
| `minPrice` | query | `number` |  |  |
| `maxPrice` | query | `number` |  |  |

**Responses:**

- **200**: Products list → `ProductListResponse`

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

</details>

<details>
<summary><code>GET</code> /products/{productId} — <strong>Get Product By ID</strong></summary>

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `productId` | path | `string` | yes |  |

**Responses:**

- **200**: Product found → `Product`
- **404**: Product not found

</details>

<details>
<summary><code>PUT</code> /products/{productId} — <strong>Update Product</strong></summary>

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `productId` | path | `string` | yes |  |

**Request body:** `UpdateProductRequest`

**Responses:**

- **200**: Product updated → `Product`

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

</details>

## Orders

<details>
<summary><code>POST</code> /orders — <strong>Create New Order</strong></summary>

**Request body:** `CreateOrderRequest`

**Responses:**

- **201**: Order created → `Order`

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

</details>

<details>
<summary><code>GET</code> /orders/{orderId} — <strong>Get Order By ID</strong></summary>

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `orderId` | path | `string` | yes |  |

**Responses:**

- **200**: Order found → `Order`

</details>

<details>
<summary><code>PATCH</code> /orders/{orderId} — <strong>Update Order Status</strong></summary>

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `orderId` | path | `string` | yes |  |

**Request body:** `UpdateOrderStatusRequest`

**Responses:**

- **200**: Order status updated → `Order`

```typescript
// Request body for updating an order's status
type UpdateOrderStatusRequest {
  status: OrderStatus
  trackingNumber?: string
}

```

</details>

## Categories

<details>
<summary><code>GET</code> /categories — <strong>Get All Product Categories</strong></summary>

**Responses:**

- **200**: Categories list → `Category[]`

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

</details>

<details>
<summary><code>POST</code> /categories — <strong>Create New Category</strong></summary>

**Request body:** `CreateCategoryRequest`

**Responses:**

- **201**: Category created → `Category`

```typescript
// Request body for creating a new category
type CreateCategoryRequest {
  name: string
  slug: string
  description?: string
  parentId?: string  // uuid
}

```

</details>

## Inventory

<details>
<summary><code>GET</code> /inventory — <strong>Get Inventory Levels</strong></summary>

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `lowStock` | query | `boolean` |  |  |

**Responses:**

- **200**: Inventory levels → `InventoryResponse`

```typescript
// Inventory levels for a list of products
type InventoryResponse {
  data: object[]
}

```

</details>

<details>
<summary><code>PUT</code> /inventory/{productId} — <strong>Update Product Inventory</strong></summary>

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `productId` | path | `string` | yes |  |

**Request body:** `UpdateInventoryRequest`

**Responses:**

- **200**: Inventory updated → `Inventory`

```typescript
// Request body for updating product inventory
type UpdateInventoryRequest {
  quantity: integer
  lowStockThreshold?: integer
}

```

</details>

## Analytics

<details>
<summary><code>GET</code> /analytics/sales — <strong>Get Sales Analytics</strong></summary>

**Parameters:**

| Name | In | Type | Required | Description |
|------|-----|------|----------|-------------|
| `startDate` | query | `string` |  |  |
| `endDate` | query | `string` |  |  |

**Responses:**

- **200**: Sales analytics data → `SalesAnalytics`

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

</details>

<details>
<summary><code>GET</code> /analytics/products — <strong>Get Product Analytics</strong></summary>

**Responses:**

- **200**: Product analytics data → `ProductAnalytics`

```typescript
// Catalog statistics and stock health metrics
type ProductAnalytics {
  totalProducts: integer
  activeProducts: integer
  categoryBreakdown?: Map<string, integer>
  lowStockProducts?: object[]
}

```

</details>

