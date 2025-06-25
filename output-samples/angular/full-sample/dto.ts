// Generated TypeScript interfaces from OpenAPI schema
// Do not modify this file manually

import {
  UserSchema,
  UserProfileSchema,
  UserPreferencesSchema,
  NotificationSettingsSchema,
  UserRoleSchema,
  AddressSchema,
  ImageUrlSchema,
  ProductSchema,
  PriceSchema,
  ProductCategorySchema,
  InventorySchema,
  OrderSchema,
  OrderItemSchema,
  OrderStatusSchema,
  PaymentMethodSchema,
  UserListResponseSchema,
  ProductListResponseSchema,
  PaginationInfoSchema,
  ErrorResponseSchema,
  CategorySchema,
  InventoryResponseSchema,
  SalesAnalyticsSchema,
  ProductAnalyticsSchema,
} from "./schema";
import { z } from "zod";

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
export interface CreateCategoryRequest {
  name: string;
  slug: string;
  description?: string;
  parentId?: string;
}
export interface UpdateInventoryRequest {
  quantity: number;
  lowStockThreshold?: number;
}

export type User = z.infer<typeof UserSchema>;
export type UserProfile = z.infer<typeof UserProfileSchema>;
export type UserPreferences = z.infer<typeof UserPreferencesSchema>;
export type NotificationSettings = z.infer<typeof NotificationSettingsSchema>;
export type UserRole = z.infer<typeof UserRoleSchema>;
export type Address = z.infer<typeof AddressSchema>;
export type ImageUrl = z.infer<typeof ImageUrlSchema>;
export type Product = z.infer<typeof ProductSchema>;
export type Price = z.infer<typeof PriceSchema>;
export type ProductCategory = z.infer<typeof ProductCategorySchema>;
export type Inventory = z.infer<typeof InventorySchema>;
export type Order = z.infer<typeof OrderSchema>;
export type OrderItem = z.infer<typeof OrderItemSchema>;
export type OrderStatus = z.infer<typeof OrderStatusSchema>;
export type PaymentMethod = z.infer<typeof PaymentMethodSchema>;
export type UserListResponse = z.infer<typeof UserListResponseSchema>;
export type ProductListResponse = z.infer<typeof ProductListResponseSchema>;
export type PaginationInfo = z.infer<typeof PaginationInfoSchema>;
export type ErrorResponse = z.infer<typeof ErrorResponseSchema>;
export type Category = z.infer<typeof CategorySchema>;
export type InventoryResponse = z.infer<typeof InventoryResponseSchema>;
export type SalesAnalytics = z.infer<typeof SalesAnalyticsSchema>;
export type ProductAnalytics = z.infer<typeof ProductAnalyticsSchema>;

export { UserSchema };
export { UserProfileSchema };
export { UserPreferencesSchema };
export { NotificationSettingsSchema };
export { UserRoleSchema };
export { AddressSchema };
export { ImageUrlSchema };
export { ProductSchema };
export { PriceSchema };
export { ProductCategorySchema };
export { InventorySchema };
export { OrderSchema };
export { OrderItemSchema };
export { OrderStatusSchema };
export { PaymentMethodSchema };
export { UserListResponseSchema };
export { ProductListResponseSchema };
export { PaginationInfoSchema };
export { ErrorResponseSchema };
export { CategorySchema };
export { InventoryResponseSchema };
export { SalesAnalyticsSchema };
export { ProductAnalyticsSchema };

