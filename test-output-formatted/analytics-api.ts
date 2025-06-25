// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { ProductAnalytics, SalesAnalytics } from "./dto";
import { subsToUrl } from "./subs-to-url.func";

@Injectable({ providedIn: "root" })
export class AnalyticsApi {
  constructor(private http: HttpClient) {}

  getSalesAnalytics(
    queryParams?: { startDate?: string; endDate?: string; },
  ): Observable<SalesAnalytics> {
    const url = subsToUrl("/analytics/sales", {}, queryParams || {});
    return this.http.get<SalesAnalytics>(url);
  }

  getProductAnalytics(): Observable<ProductAnalytics> {
    const url = subsToUrl("/analytics/products", {}, {});
    return this.http.get<ProductAnalytics>(url);
  }
}
