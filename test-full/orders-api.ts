// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { z } from "zod";
import { subsToUrl } from "./subs-to-url.func";
import {
  Product,
  ProductSchema,
  ProductListResponse,
  ProductListResponseSchema,
  UpdateProductRequest,
} from "./dto";

@Injectable({ providedIn: "root" })
export class ProductsApi {
  constructor(private http: HttpClient) {}

  searchProductsWithFilters(queryParams?: { category?: ProductCategory, minPrice?: number, maxPrice?: number }): Observable<ProductListResponse> {
    const url = subsToUrl("/products", {}, queryParams || {});
    return this.http.get<ProductListResponse>(url)
      .pipe(
        map(response => ProductListResponseSchema.parse(response))
      );
  }

  getProductByID(productId: string): Observable<Product> {
    const url = subsToUrl("/products/{productId}", { productId: productId }, {});
    return this.http.get<Product>(url)
      .pipe(
        map(response => ProductSchema.parse(response))
      );
  }

  updateProduct(productId: string, dto: UpdateProductRequest): Observable<Product> {
    const url = subsToUrl("/products/{productId}", { productId: productId }, {});
    return this.http.put<Product>(url, dto)
      .pipe(
        map(response => ProductSchema.parse(response))
      );
  }

}
