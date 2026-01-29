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
