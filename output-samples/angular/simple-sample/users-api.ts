// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { subsToUrl } from "./subs-to-url.func";
import {
  ApiResponse,
  ApiResponseSchema,
  CreateUserRequest,
  User,
  UserSchema,
} from "./dto";

@Injectable({ providedIn: "root" })
export class UsersApi {
  constructor(private http: HttpClient) {}

  listAllUsers(): Observable<unknown[]> {
    const url = subsToUrl("/users", {}, {});
    return this.http.get<unknown[]>(url)
      .pipe(
        map(response => unknown[]Schema.parse(response))
      );
  }

  createNewUser(dto: CreateUserRequest): Observable<ApiResponse> {
    const url = subsToUrl("/users", {}, {});
    return this.http.post<ApiResponse>(url, dto)
      .pipe(
        map(response => ApiResponseSchema.parse(response))
      );
  }

  getUserByID(userId: number): Observable<User> {
    const url = subsToUrl("/users/{userId}", { userId: userId }, {});
    return this.http.get<User>(url)
      .pipe(
        map(response => UserSchema.parse(response))
      );
  }

  updateUserProfile(userId: number, dto: CreateUserRequest): Observable<ApiResponse> {
    const url = subsToUrl("/users/{userId}", { userId: userId }, {});
    return this.http.put<ApiResponse>(url, dto)
      .pipe(
        map(response => ApiResponseSchema.parse(response))
      );
  }

  deleteUserAccount(userId: number): Observable<ApiResponse> {
    const url = subsToUrl("/users/{userId}", { userId: userId }, {});
    return this.http.delete<ApiResponse>(url)
      .pipe(
        map(response => ApiResponseSchema.parse(response))
      );
  }

  activateUserAccount(userId: number): Observable<ApiResponse> {
    const url = subsToUrl("/users/{userId}/activate", { userId: userId }, {});
    return this.http.post<ApiResponse>(url)
      .pipe(
        map(response => ApiResponseSchema.parse(response))
      );
  }

  deactivateUserAccount(userId: number): Observable<ApiResponse> {
    const url = subsToUrl("/users/{userId}/deactivate", { userId: userId }, {});
    return this.http.post<ApiResponse>(url)
      .pipe(
        map(response => ApiResponseSchema.parse(response))
      );
  }

}