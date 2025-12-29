export interface Inventory {
  quantity: number;
  status: "in_stock" | "low_stock" | "out_of_stock" | "discontinued";
  lowStockThreshold?: number;
}
