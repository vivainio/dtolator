// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { subsToUrl } from "./subs-to-url.func";
import {
  ProductAnalytics,
  ProductAnalyticsSchema,
  SalesAnalytics,
  SalesAnalyticsSchema,
} from "./dto";

@Injectable({ providedIn: "root" })
export class AnalyticsApi {
  constructor(private http: HttpClient) {}

  getSalesAnalytics(queryParams?: { startDate?: string, endDate?: string }): Observable<SalesAnalytics> {
    const url = subsToUrl("/analytics/sales", {}, queryParams || {});
    return this.http.get<SalesAnalytics>(url)
      .pipe(
        map(response => SalesAnalyticsSchema.parse(response))
      );
  }

  getProductAnalytics(): Observable<ProductAnalytics> {
    const url = subsToUrl("/analytics/products", {}, {});
    return this.http.get<ProductAnalytics>(url)
      .pipe(
        map(response => ProductAnalyticsSchema.parse(response))
      );
  }

}