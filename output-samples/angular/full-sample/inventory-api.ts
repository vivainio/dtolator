// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { subsToUrl } from "./subs-to-url.func";
import {
  CreateOrderRequestSchema,
  type CreateOrderRequest,
  OrderSchema,
  type Order,
  UpdateOrderStatusRequestSchema,
  type UpdateOrderStatusRequest,
} from "./dto";

@Injectable({ providedIn: "root" })
export class OrdersApi {
  constructor(private http: HttpClient) {}

  createNewOrder(dto: CreateOrderRequest): Observable<Order> {
    // Validate request body with Zod
    const validatedDto = CreateOrderRequestSchema.parse(dto);

    const url = subsToUrl("/orders", {}, {});
    return this.http.post<Order>(url, validatedDto)
      .pipe(
        map(response => OrderSchema.parse(response))
      );
  }

  getOrderByID(orderId: string): Observable<Order> {
    const url = subsToUrl("/orders/{orderId}", { orderId: orderId }, {});
    return this.http.get<Order>(url)
      .pipe(
        map(response => OrderSchema.parse(response))
      );
  }

  updateOrderStatus(orderId: string, dto: UpdateOrderStatusRequest): Observable<Order> {
    // Validate request body with Zod
    const validatedDto = UpdateOrderStatusRequestSchema.parse(dto);

    const url = subsToUrl("/orders/{orderId}", { orderId: orderId }, {});
    return this.http.patch<Order>(url, validatedDto)
      .pipe(
        map(response => OrderSchema.parse(response))
      );
  }

}
