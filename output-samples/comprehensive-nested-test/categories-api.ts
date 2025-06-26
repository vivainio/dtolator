// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { z } from "zod";
import { subsToUrl } from "./subs-to-url.func";
import {
  Category,
  CategorySchema,
  CreateCategoryRequest,
} from "./dto";

@Injectable({ providedIn: "root" })
export class CategoriesApi {
  constructor(private http: HttpClient) {}

  getAllProductCategories(): Observable<Category[]> {
    const url = subsToUrl("/categories", {}, {});
    return this.http.get<Category[]>(url)
      .pipe(
        map(response => z.array(CategorySchema).parse(response))
      );
  }

  createNewCategory(dto: CreateCategoryRequest): Observable<Category> {
    const url = subsToUrl("/categories", {}, {});
    return this.http.post<Category>(url, dto)
      .pipe(
        map(response => CategorySchema.parse(response))
      );
  }

}

