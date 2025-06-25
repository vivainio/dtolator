import { z } from 'zod';

export const UserSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
  profile: UserProfileSchema,
  preferences: UserPreferencesSchema.optional(),
  createdAt: z.string().datetime().optional(),
  updatedAt: z.string().datetime().optional(),
  isActive: z.boolean().optional(),
  roles: z.array(UserRoleSchema).optional()
});

export type User = z.infer<typeof UserSchema>;

export const UserProfileSchema = z.object({
  firstName: z.string().min(1).max(50),
  lastName: z.string().min(1).max(50),
  dateOfBirth: z.string().date().nullable().optional(),
  phoneNumber: z.string().regex(new RegExp("^\+?[1-9]\d{1,14}$")).nullable().optional(),
  avatar: ImageUrlSchema.optional(),
  address: AddressSchema.optional()
});

export type UserProfile = z.infer<typeof UserProfileSchema>;

export const UserPreferencesSchema = z.object({
  language: z.enum(["en", "es", "fr", "de", "it"]).optional(),
  currency: z.enum(["USD", "EUR", "GBP", "JPY"]).optional(),
  notifications: NotificationSettingsSchema.optional(),
  theme: z.enum(["light", "dark", "auto"]).optional()
});

export type UserPreferences = z.infer<typeof UserPreferencesSchema>;

export const NotificationSettingsSchema = z.object({
  email: z.boolean().optional(),
  push: z.boolean().optional(),
  sms: z.boolean().optional(),
  marketing: z.boolean().optional()
});

export type NotificationSettings = z.infer<typeof NotificationSettingsSchema>;

export const UserRoleSchema = z.enum(["customer", "admin", "moderator", "vendor"]);

export type UserRole = z.infer<typeof UserRoleSchema>;

export const AddressSchema = z.object({
  street: z.string().min(1).max(100),
  street2: z.string().max(100).nullable().optional(),
  city: z.string().min(1).max(50),
  state: z.string().max(50).nullable().optional(),
  country: z.string().regex(new RegExp("^[A-Z]{2}$")),
  postalCode: z.string().min(3).max(10)
});

export type Address = z.infer<typeof AddressSchema>;

export const ImageUrlSchema = z.object({
  url: z.string().url(),
  alt: z.string().optional(),
  width: z.number().min(1).int().optional(),
  height: z.number().min(1).int().optional()
});

export type ImageUrl = z.infer<typeof ImageUrlSchema>;

export const ProductSchema = z.object({
  id: z.string().uuid(),
  name: z.string().min(1).max(200),
  description: z.string().max(2000).nullable().optional(),
  price: PriceSchema,
  category: ProductCategorySchema,
  tags: z.array(z.string()).optional(),
  images: z.array(ImageUrlSchema).optional(),
  inventory: InventorySchema.optional(),
  specifications: z.object({}).optional(),
  isActive: z.boolean().optional(),
  createdAt: z.string().datetime().optional()
});

export type Product = z.infer<typeof ProductSchema>;

export const PriceSchema = z.object({
  amount: z.number().min(0),
  currency: z.enum(["USD", "EUR", "GBP", "JPY"]),
  originalAmount: z.number().min(0).nullable().optional()
});

export type Price = z.infer<typeof PriceSchema>;

export const ProductCategorySchema = z.enum(["electronics", "clothing", "home", "books", "sports", "beauty", "automotive"]);

export type ProductCategory = z.infer<typeof ProductCategorySchema>;

export const InventorySchema = z.object({
  quantity: z.number().min(0).int(),
  status: z.enum(["in_stock", "low_stock", "out_of_stock", "discontinued"]),
  lowStockThreshold: z.number().min(0).int().optional()
});

export type Inventory = z.infer<typeof InventorySchema>;

export const OrderSchema = z.object({
  id: z.string().uuid(),
  userId: z.string().uuid(),
  items: z.array(OrderItemSchema),
  total: PriceSchema,
  status: OrderStatusSchema,
  shippingAddress: AddressSchema.optional(),
  billingAddress: AddressSchema.optional(),
  paymentMethod: PaymentMethodSchema.optional(),
  orderDate: z.string().datetime().optional(),
  estimatedDelivery: z.string().date().nullable().optional(),
  trackingNumber: z.string().nullable().optional()
});

export type Order = z.infer<typeof OrderSchema>;

export const OrderItemSchema = z.object({
  productId: z.string().uuid(),
  quantity: z.number().min(1).int(),
  price: PriceSchema,
  productSnapshot: ProductSchema.optional()
});

export type OrderItem = z.infer<typeof OrderItemSchema>;

export const OrderStatusSchema = z.enum(["pending", "confirmed", "processing", "shipped", "delivered", "cancelled", "refunded"]);

export type OrderStatus = z.infer<typeof OrderStatusSchema>;

export const PaymentMethodSchema = z.object({
  type: z.enum(["credit_card", "debit_card", "paypal", "bank_transfer", "crypto"]),
  last4: z.string().regex(new RegExp("^[0-9]{4}$")).optional(),
  brand: z.enum(["visa", "mastercard", "amex", "discover"]).optional()
});

export type PaymentMethod = z.infer<typeof PaymentMethodSchema>;

export const UserListResponseSchema = z.object({
  data: z.array(UserSchema),
  pagination: PaginationInfoSchema
});

export type UserListResponse = z.infer<typeof UserListResponseSchema>;

export const ProductListResponseSchema = z.object({
  data: z.array(ProductSchema),
  pagination: PaginationInfoSchema,
  filters: z.object({
  categories: z.array(ProductCategorySchema).optional(),
  priceRange: z.object({
  min: z.number().optional(),
  max: z.number().optional()
}).optional()
}).optional()
});

export type ProductListResponse = z.infer<typeof ProductListResponseSchema>;

export const PaginationInfoSchema = z.object({
  page: z.number().min(1).int(),
  limit: z.number().min(1).int(),
  total: z.number().min(0).int(),
  totalPages: z.number().min(0).int(),
  hasNext: z.boolean().optional(),
  hasPrev: z.boolean().optional()
});

export type PaginationInfo = z.infer<typeof PaginationInfoSchema>;

export const ErrorResponseSchema = z.object({
  error: z.object({
  code: z.string(),
  message: z.string(),
  details: z.array(z.object({
  field: z.string().optional(),
  message: z.string().optional()
})).optional(),
  requestId: z.string().uuid().optional(),
  timestamp: z.string().datetime().optional()
})
});

export type ErrorResponse = z.infer<typeof ErrorResponseSchema>;

export const CategorySchema = z.object({
  id: z.string().uuid(),
  name: z.string().min(1).max(100),
  slug: z.string().regex(new RegExp("^[a-z0-9-]+$")),
  description: z.string().max(500).optional(),
  parentId: z.string().uuid().nullable().optional(),
  isActive: z.boolean().optional()
});

export type Category = z.infer<typeof CategorySchema>;

export const InventoryResponseSchema = z.object({
  data: z.array(z.object({
  productId: z.string().uuid(),
  productName: z.string().optional(),
  inventory: InventorySchema
}))
});

export type InventoryResponse = z.infer<typeof InventoryResponseSchema>;

export const SalesAnalyticsSchema = z.object({
  totalRevenue: z.number().min(0),
  totalOrders: z.number().min(0).int(),
  averageOrderValue: z.number().min(0),
  topProducts: z.array(z.object({
  productId: z.string().uuid().optional(),
  productName: z.string().optional(),
  revenue: z.number().optional(),
  unitsSold: z.number().int().optional()
})).optional(),
  period: z.object({
  startDate: z.string().date().optional(),
  endDate: z.string().date().optional()
}).optional()
});

export type SalesAnalytics = z.infer<typeof SalesAnalyticsSchema>;

export const ProductAnalyticsSchema = z.object({
  totalProducts: z.number().min(0).int(),
  activeProducts: z.number().min(0).int(),
  categoryBreakdown: z.object({}).optional(),
  lowStockProducts: z.array(z.object({
  productId: z.string().uuid().optional(),
  productName: z.string().optional(),
  currentStock: z.number().int().optional(),
  threshold: z.number().int().optional()
})).optional()
});

export type ProductAnalytics = z.infer<typeof ProductAnalyticsSchema>;

