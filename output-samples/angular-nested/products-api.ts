// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { subsToUrl } from "./subs-to-url.func";
import {
  Product,
  ProductCategory,
  ProductListResponse,
  UpdateProductRequest,
} from "./dto";

@Injectable({ providedIn: "root" })
export class ProductsApi {
  constructor(private http: HttpClient) {}

  /**
   * Search Products With Filters
   *
   * @param queryParams - Query parameters object
   * @param queryParams.category - optional parameter of type ProductCategory
   * @param queryParams.minPrice - optional parameter of type number
   * @param queryParams.maxPrice - optional parameter of type number
   * @returns Observable<ProductListResponse> - Products list
   */
  searchProductsWithFilters(queryParams?: { category?: ProductCategory, minPrice?: number, maxPrice?: number }): Observable<ProductListResponse> {
    const url = subsToUrl("/products", {}, queryParams || {});
    return this.http.get<ProductListResponse>(url);
  }

  /**
   * Get Product By ID
   *
   * @param productId - Path parameter of type string
   * @returns Observable<Product> - Product found
   */
  getProductByID(productId: string): Observable<Product> {
    const url = subsToUrl("/products/{productId}", { productId: productId }, {});
    return this.http.get<Product>(url);
  }

  /**
   * Update Product
   *
   * @param productId - Path parameter of type string
   * @param dto - Request body of type UpdateProductRequest
   * @returns Observable<Product> - Product updated
   */
  updateProduct(productId: string, dto: UpdateProductRequest): Observable<Product> {
    const url = subsToUrl("/products/{productId}", { productId: productId }, {});
    return this.http.put<Product>(url, dto);
  }

}

