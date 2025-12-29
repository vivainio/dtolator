export interface UpdateOrderStatusRequest {
  status: OrderStatus;
  trackingNumber?: string;
}
