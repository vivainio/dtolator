export interface Product {
  id: string;
  name: string;
  description?: string | null;
  price: Price;
  category: ProductCategory;
  tags?: string[];
  images?: ImageUrl[];
  inventory?: Inventory;
  specifications?: Record<string, unknown>;
  isActive?: boolean;
  createdAt?: string;
}
