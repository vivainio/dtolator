export interface UpdateProductRequest {
  name?: string;
  description?: string;
  price?: Price;
  category?: ProductCategory;
  isActive?: boolean;
}
