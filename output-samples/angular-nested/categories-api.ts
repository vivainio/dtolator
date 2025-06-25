// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { subsToUrl } from "./subs-to-url.func";
import {
  Inventory,
  InventoryResponse,
  UpdateInventoryRequest,
} from "./dto";

@Injectable({ providedIn: "root" })
export class InventoryApi {
  constructor(private http: HttpClient) {}

  getInventoryLevels(queryParams?: { lowStock?: boolean }): Observable<InventoryResponse> {
    const url = subsToUrl("/inventory", {}, queryParams || {});
    return this.http.get<InventoryResponse>(url);
  }

  updateProductInventory(productId: string, dto: UpdateInventoryRequest): Observable<Inventory> {
    const url = subsToUrl("/inventory/{productId}", { productId: productId }, {});
    return this.http.put<Inventory>(url, dto);
  }

}
