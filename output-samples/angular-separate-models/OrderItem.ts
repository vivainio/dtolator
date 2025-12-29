export interface OrderItem {
  productId: string;
  quantity: number;
  price: Price;
  productSnapshot?: Product;
}
