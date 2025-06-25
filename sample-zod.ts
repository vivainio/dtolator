import { z } from 'zod';

export const User = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
  profile: UserProfile,
  preferences: UserPreferences.optional(),
  createdAt: z.string().datetime().optional(),
  updatedAt: z.string().datetime().optional(),
  isActive: z.boolean().optional(),
  roles: z.array(UserRole).optional()
});

export type User = z.infer<typeof User>;

export const UserProfile = z.object({
  firstName: z.string().min(1).max(50),
  lastName: z.string().min(1).max(50),
  dateOfBirth: z.string().date().nullable().optional(),
  phoneNumber: z.string().regex(new RegExp("^\+?[1-9]\d{1,14}$")).nullable().optional(),
  avatar: ImageUrl.optional(),
  address: Address.optional()
});

export type UserProfile = z.infer<typeof UserProfile>;

export const UserPreferences = z.object({
  language: z.enum(["en", "es", "fr", "de", "it"]).optional(),
  currency: z.enum(["USD", "EUR", "GBP", "JPY"]).optional(),
  notifications: NotificationSettings.optional(),
  theme: z.enum(["light", "dark", "auto"]).optional()
});

export type UserPreferences = z.infer<typeof UserPreferences>;

export const NotificationSettings = z.object({
  email: z.boolean().optional(),
  push: z.boolean().optional(),
  sms: z.boolean().optional(),
  marketing: z.boolean().optional()
});

export type NotificationSettings = z.infer<typeof NotificationSettings>;

export const UserRole = z.enum(["customer", "admin", "moderator", "vendor"]);

export type UserRole = z.infer<typeof UserRole>;

export const Address = z.object({
  street: z.string().min(1).max(100),
  street2: z.string().max(100).nullable().optional(),
  city: z.string().min(1).max(50),
  state: z.string().max(50).nullable().optional(),
  country: z.string().regex(new RegExp("^[A-Z]{2}$")),
  postalCode: z.string().min(3).max(10)
});

export type Address = z.infer<typeof Address>;

export const ImageUrl = z.object({
  url: z.string().url(),
  alt: z.string().optional(),
  width: z.number().min(1).int().optional(),
  height: z.number().min(1).int().optional()
});

export type ImageUrl = z.infer<typeof ImageUrl>;

export const Product = z.object({
  id: z.string().uuid(),
  name: z.string().min(1).max(200),
  description: z.string().max(2000).nullable().optional(),
  price: Price,
  category: ProductCategory,
  tags: z.array(z.string()).optional(),
  images: z.array(ImageUrl).optional(),
  inventory: Inventory.optional(),
  specifications: z.object({}).optional(),
  isActive: z.boolean().optional(),
  createdAt: z.string().datetime().optional()
});

export type Product = z.infer<typeof Product>;

export const Price = z.object({
  amount: z.number().min(0),
  currency: z.enum(["USD", "EUR", "GBP", "JPY"]),
  originalAmount: z.number().min(0).nullable().optional()
});

export type Price = z.infer<typeof Price>;

export const ProductCategory = z.enum(["electronics", "clothing", "home", "books", "sports", "beauty", "automotive"]);

export type ProductCategory = z.infer<typeof ProductCategory>;

export const Inventory = z.object({
  quantity: z.number().min(0).int(),
  status: z.enum(["in_stock", "low_stock", "out_of_stock", "discontinued"]),
  lowStockThreshold: z.number().min(0).int().optional()
});

export type Inventory = z.infer<typeof Inventory>;

export const Order = z.object({
  id: z.string().uuid(),
  userId: z.string().uuid(),
  items: z.array(OrderItem),
  total: Price,
  status: OrderStatus,
  shippingAddress: Address.optional(),
  billingAddress: Address.optional(),
  paymentMethod: PaymentMethod.optional(),
  orderDate: z.string().datetime().optional(),
  estimatedDelivery: z.string().date().nullable().optional(),
  trackingNumber: z.string().nullable().optional()
});

export type Order = z.infer<typeof Order>;

export const OrderItem = z.object({
  productId: z.string().uuid(),
  quantity: z.number().min(1).int(),
  price: Price,
  productSnapshot: Product.optional()
});

export type OrderItem = z.infer<typeof OrderItem>;

export const OrderStatus = z.enum(["pending", "confirmed", "processing", "shipped", "delivered", "cancelled", "refunded"]);

export type OrderStatus = z.infer<typeof OrderStatus>;

export const PaymentMethod = z.object({
  type: z.enum(["credit_card", "debit_card", "paypal", "bank_transfer", "crypto"]),
  last4: z.string().regex(new RegExp("^[0-9]{4}$")).optional(),
  brand: z.enum(["visa", "mastercard", "amex", "discover"]).optional()
});

export type PaymentMethod = z.infer<typeof PaymentMethod>;

export const CreateUserRequest = z.object({
  email: z.string().email(),
  password: z.string().min(8).max(128).regex(new RegExp("^(?=.*[a-z])(?=.*[A-Z])(?=.*\d)(?=.*[@$!%*?&])[A-Za-z\d@$!%*?&]")),
  profile: UserProfile,
  preferences: UserPreferences.optional()
});

export type CreateUserRequest = z.infer<typeof CreateUserRequest>;

export const CreateOrderRequest = z.object({
  items: z.array(z.object({
  productId: z.string().uuid(),
  quantity: z.number().min(1).int()
})),
  shippingAddress: Address,
  billingAddress: Address.optional(),
  paymentMethod: PaymentMethod.optional()
});

export type CreateOrderRequest = z.infer<typeof CreateOrderRequest>;

export const UserListResponse = z.object({
  data: z.array(User),
  pagination: PaginationInfo
});

export type UserListResponse = z.infer<typeof UserListResponse>;

export const ProductListResponse = z.object({
  data: z.array(Product),
  pagination: PaginationInfo,
  filters: z.object({
  categories: z.array(ProductCategory).optional(),
  priceRange: z.object({
  min: z.number().optional(),
  max: z.number().optional()
}).optional()
}).optional()
});

export type ProductListResponse = z.infer<typeof ProductListResponse>;

export const PaginationInfo = z.object({
  page: z.number().min(1).int(),
  limit: z.number().min(1).int(),
  total: z.number().min(0).int(),
  totalPages: z.number().min(0).int(),
  hasNext: z.boolean().optional(),
  hasPrev: z.boolean().optional()
});

export type PaginationInfo = z.infer<typeof PaginationInfo>;

export const ErrorResponse = z.object({
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

export type ErrorResponse = z.infer<typeof ErrorResponse>;

