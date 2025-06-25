// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { subsToUrl } from "./subs-to-url.func";
import {
  CreateOrderRequest,
  Order,
  UpdateOrderStatusRequest,
} from "./dto";

@Injectable({ providedIn: "root" })
export class OrdersApi {
  constructor(private http: HttpClient) {}

  createNewOrder(dto: CreateOrderRequest): Observable<Order> {
    const url = subsToUrl("/orders", {}, {});
    return this.http.post<Order>(url, dto);
  }

  getOrderByID(orderId: string): Observable<Order> {
    const url = subsToUrl("/orders/{orderId}", { orderId: orderId }, {});
    return this.http.get<Order>(url);
  }

  updateOrderStatus(orderId: string, dto: UpdateOrderStatusRequest): Observable<Order> {
    const url = subsToUrl("/orders/{orderId}", { orderId: orderId }, {});
    return this.http.patch<Order>(url, dto);
  }

}
