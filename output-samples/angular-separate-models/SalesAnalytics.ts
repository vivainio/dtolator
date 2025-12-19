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
