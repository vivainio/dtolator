// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { Product, ProductListResponse, UpdateProductRequest } from "./dto";
import { subsToUrl } from "./subs-to-url.func";

@Injectable({ providedIn: "root" })
export class ProductsApi {
  constructor(private http: HttpClient) {}

  searchProductsWithFilters(
    queryParams?: { category?: unknown; minPrice?: number; maxPrice?: number; },
  ): Observable<ProductListResponse> {
    const url = subsToUrl("/products", {}, queryParams || {});
    return this.http.get<ProductListResponse>(url);
  }

  getProductByID(productId: string): Observable<Product> {
    const url = subsToUrl(
      "/products/{productId}",
      { productId: productId },
      {},
    );
    return this.http.get<Product>(url);
  }

  updateProduct(
    productId: string,
    dto: UpdateProductRequest,
  ): Observable<Product> {
    const url = subsToUrl(
      "/products/{productId}",
      { productId: productId },
      {},
    );
    return this.http.put<Product>(url, dto);
  }
}
