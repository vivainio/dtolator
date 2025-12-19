export interface ProductListResponse {
  data: Product[];
  pagination: PaginationInfo;
  filters?: {
    categories?: ProductCategory[];
    priceRange?: {
    min?: number;
    max?: number;
  };
  };
}
