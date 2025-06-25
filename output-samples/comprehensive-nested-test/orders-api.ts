// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import {
  Product,
  ProductListResponse,
  ProductListResponseSchema,
  ProductSchema,
  UpdateProductRequest,
  UpdateProductRequestSchema,
} from "./dto";
import { subsToUrl } from "./subs-to-url.func";

@Injectable({ providedIn: "root" })
export class ProductsApi {
  constructor(private http: HttpClient) {}

  searchProductsWithFilters(
    queryParams?: { category?: unknown; minPrice?: number; maxPrice?: number; },
  ): Observable<ProductListResponse> {
    const url = subsToUrl("/products", {}, queryParams || {});
    return this.http.get<ProductListResponse>(url)
      .pipe(
        map(response => ProductListResponseSchema.parse(response)),
      );
  }

  getProductByID(productId: string): Observable<Product> {
    const url = subsToUrl(
      "/products/{productId}",
      { productId: productId },
      {},
    );
    return this.http.get<Product>(url)
      .pipe(
        map(response => ProductSchema.parse(response)),
      );
  }

  updateProduct(
    productId: string,
    dto: UpdateProductRequest,
  ): Observable<Product> {
    // Validate request body with Zod
    const validatedDto = UpdateProductRequestSchema.parse(dto);

    const url = subsToUrl(
      "/products/{productId}",
      { productId: productId },
      {},
    );
    return this.http.put<Product>(url, validatedDto)
      .pipe(
        map(response => ProductSchema.parse(response)),
      );
  }
}
