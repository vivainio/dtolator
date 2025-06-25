// Generated TypeScript interfaces from OpenAPI schema
// Do not modify this file manually

export interface User {
  id: string;
  email: string;
  profile: UserProfile;
  preferences?: UserPreferences;
  createdAt?: string;
  updatedAt?: string;
  isActive?: boolean;
  roles?: UserRole[];
}

export interface UserProfile {
  firstName: string;
  lastName: string;
  dateOfBirth?: string | null;
  phoneNumber?: string | null;
  avatar?: ImageUrl;
  address?: Address;
}

export interface UserPreferences {
  language?: "en" | "es" | "fr" | "de" | "it";
  currency?: "USD" | "EUR" | "GBP" | "JPY";
  notifications?: NotificationSettings;
  theme?: "light" | "dark" | "auto";
}

export interface NotificationSettings {
  email?: boolean;
  push?: boolean;
  sms?: boolean;
  marketing?: boolean;
}

export type UserRole = "customer" | "admin" | "moderator" | "vendor";

export interface Address {
  street: string;
  street2?: string | null;
  city: string;
  state?: string | null;
  country: string;
  postalCode: string;
}

export interface ImageUrl {
  url: string;
  alt?: string;
  width?: number;
  height?: number;
}

export interface Product {
  id: string;
  name: string;
  description?: string | null;
  price: Price;
  category: ProductCategory;
  tags?: string[];
  images?: ImageUrl[];
  inventory?: Inventory;
  specifications?: Record<string, unknown>;
  isActive?: boolean;
  createdAt?: string;
}

export interface Price {
  amount: number;
  currency: "USD" | "EUR" | "GBP" | "JPY";
  originalAmount?: number | null;
}

export type ProductCategory =
  | "electronics"
  | "clothing"
  | "home"
  | "books"
  | "sports"
  | "beauty"
  | "automotive";

export interface Inventory {
  quantity: number;
  status: "in_stock" | "low_stock" | "out_of_stock" | "discontinued";
  lowStockThreshold?: number;
}

export interface Order {
  id: string;
  userId: string;
  items: OrderItem[];
  total: Price;
  status: OrderStatus;
  shippingAddress?: Address;
  billingAddress?: Address;
  paymentMethod?: PaymentMethod;
  orderDate?: string;
  estimatedDelivery?: string | null;
  trackingNumber?: string | null;
}

export interface OrderItem {
  productId: string;
  quantity: number;
  price: Price;
  productSnapshot?: Product;
}

export type OrderStatus =
  | "pending"
  | "confirmed"
  | "processing"
  | "shipped"
  | "delivered"
  | "cancelled"
  | "refunded";

export interface PaymentMethod {
  type: "credit_card" | "debit_card" | "paypal" | "bank_transfer" | "crypto";
  last4?: string;
  brand?: "visa" | "mastercard" | "amex" | "discover";
}

export interface CreateUserRequest {
  email: string;
  password: string;
  profile: UserProfile;
  preferences?: UserPreferences;
}

export interface CreateOrderRequest {
  items: {
    productId: string;
    quantity: number;
  }[];
  shippingAddress: Address;
  billingAddress?: Address;
  paymentMethod?: PaymentMethod;
}

export interface UserListResponse {
  data: User[];
  pagination: PaginationInfo;
}

export interface ProductListResponse {
  data: Product[];
  pagination: PaginationInfo;
  filters?: {
    categories?: ProductCategory[];
    priceRange?: {
      min?: number;
      max?: number;
    };
  };
}

export interface PaginationInfo {
  page: number;
  limit: number;
  total: number;
  totalPages: number;
  hasNext?: boolean;
  hasPrev?: boolean;
}

export interface ErrorResponse {
  error: {
    code: string;
    message: string;
    details?: {
      field?: string;
      message?: string;
    }[];
    requestId?: string;
    timestamp?: string;
  };
}

export interface UpdateProductRequest {
  name?: string;
  description?: string;
  price?: Price;
  category?: ProductCategory;
  isActive?: boolean;
}

export interface UpdateOrderStatusRequest {
  status: OrderStatus;
  trackingNumber?: string;
}

export interface Category {
  id: string;
  name: string;
  slug: string;
  description?: string;
  parentId?: string | null;
  isActive?: boolean;
}

export interface CreateCategoryRequest {
  name: string;
  slug: string;
  description?: string;
  parentId?: string;
}

export interface InventoryResponse {
  data: {
    productId: string;
    productName?: string;
    inventory: Inventory;
  }[];
}

export interface UpdateInventoryRequest {
  quantity: number;
  lowStockThreshold?: number;
}

export interface SalesAnalytics {
  totalRevenue: number;
  totalOrders: number;
  averageOrderValue: number;
  topProducts?: {
    productId?: string;
    productName?: string;
    revenue?: number;
    unitsSold?: number;
  }[];
  period?: {
    startDate?: string;
    endDate?: string;
  };
}

export interface ProductAnalytics {
  totalProducts: number;
  activeProducts: number;
  categoryBreakdown?: Record<string, unknown>;
  lowStockProducts?: {
    productId?: string;
    productName?: string;
    currentStock?: number;
    threshold?: number;
  }[];
}
