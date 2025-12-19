export interface InventoryResponse {
  data: {
    productId: string;
    productName?: string;
    inventory: Inventory;
  }[];
}
