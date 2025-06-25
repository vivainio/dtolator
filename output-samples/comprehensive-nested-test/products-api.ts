// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { map } from "rxjs/operators";
import { subsToUrl } from "./subs-to-url.func";
import {
  CreateUserRequestSchema,
  type CreateUserRequest,
  UserSchema,
  type User,
  UserListResponseSchema,
  type UserListResponse,
} from "./dto";

@Injectable({ providedIn: "root" })
export class UsersApi {
  constructor(private http: HttpClient) {}

  getAllUsersWithPagination(queryParams?: { page?: number, limit?: number }): Observable<UserListResponse> {
    const url = subsToUrl("/users", {}, queryParams || {});
    return this.http.get<UserListResponse>(url)
      .pipe(
        map(response => UserListResponseSchema.parse(response))
      );
  }

  createNewUserAccount(dto: CreateUserRequest): Observable<User> {
    // Validate request body with Zod
    const validatedDto = CreateUserRequestSchema.parse(dto);

    const url = subsToUrl("/users", {}, {});
    return this.http.post<User>(url, validatedDto)
      .pipe(
        map(response => UserSchema.parse(response))
      );
  }

  getUserProfileByID(userId: string): Observable<User> {
    const url = subsToUrl("/users/{userId}", { userId: userId }, {});
    return this.http.get<User>(url)
      .pipe(
        map(response => UserSchema.parse(response))
      );
  }

}
