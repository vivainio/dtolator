// Generated by dtolator --from-openapi full-sample.json --angular
// Do not modify manually

import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { fillUrl } from './fill-url';
import {
  Category,
  CreateCategoryRequest,
} from './dto';

@Injectable({ providedIn: 'root' })
export class CategoriesApi {
  constructor(private http: HttpClient) {}

  /**
   * Get All Product Categories
   *
   * @returns Observable<Category[]> - Categories list
   */
  getAllProductCategories(): Observable<Category[]> {
    const url = fillUrl('/categories', {}, {});
    return this.http.get<Category[]>(url);
  }

  /**
   * Create New Category
   *
   * @param dto - Request body of type CreateCategoryRequest
   * @returns Observable<Category> - Category created
   */
  createNewCategory(dto: CreateCategoryRequest): Observable<Category> {
    const url = fillUrl('/categories', {}, {});
    return this.http.post<Category>(url, dto);
  }

}

