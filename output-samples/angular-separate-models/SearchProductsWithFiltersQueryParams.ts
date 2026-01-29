/**
 * Query parameters for Search Products With Filters
 */
export type SearchProductsWithFiltersQueryParams = Partial<{
  category: ProductCategory;
  minPrice: number;
  maxPrice: number;
}>;
