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
