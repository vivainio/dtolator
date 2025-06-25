// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import {
  Inventory,
  InventoryResponse,
  InventoryResponseSchema,
  InventorySchema,
  UpdateInventoryRequest,
  UpdateInventoryRequestSchema,
} from "./dto";
import { subsToUrl } from "./subs-to-url.func";

@Injectable({ providedIn: "root" })
export class InventoryApi {
  constructor(private http: HttpClient) {}

  getInventoryLevels(
    queryParams?: { lowStock?: boolean; },
  ): Observable<InventoryResponse> {
    const url = subsToUrl("/inventory", {}, queryParams || {});
    return this.http.get<InventoryResponse>(url)
      .pipe(
        map(response => InventoryResponseSchema.parse(response)),
      );
  }

  updateProductInventory(
    productId: string,
    dto: UpdateInventoryRequest,
  ): Observable<Inventory> {
    // Validate request body with Zod
    const validatedDto = UpdateInventoryRequestSchema.parse(dto);

    const url = subsToUrl(
      "/inventory/{productId}",
      { productId: productId },
      {},
    );
    return this.http.put<Inventory>(url, validatedDto)
      .pipe(
        map(response => InventorySchema.parse(response)),
      );
  }
}
