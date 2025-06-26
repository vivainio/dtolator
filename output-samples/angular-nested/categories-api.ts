// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { subsToUrl } from "./subs-to-url.func";
import {
  Category,
  CreateCategoryRequest,
} from "./dto";

@Injectable({ providedIn: "root" })
export class CategoriesApi {
  constructor(private http: HttpClient) {}

  getAllProductCategories(): Observable<Category[]> {
    const url = subsToUrl("/categories", {}, {});
    return this.http.get<Category[]>(url);
  }

  createNewCategory(dto: CreateCategoryRequest): Observable<Category> {
    const url = subsToUrl("/categories", {}, {});
    return this.http.post<Category>(url, dto);
  }

}

