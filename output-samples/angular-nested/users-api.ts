// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { subsToUrl } from "./subs-to-url.func";
import {
  CreateUserRequest,
  User,
  UserListResponse,
} from "./dto";

@Injectable({ providedIn: "root" })
export class UsersApi {
  constructor(private http: HttpClient) {}

  getAllUsersWithPagination(queryParams?: { page?: number, limit?: number }): Observable<UserListResponse> {
    const url = subsToUrl("/users", {}, queryParams || {});
    return this.http.get<UserListResponse>(url);
  }

  createNewUserAccount(dto: CreateUserRequest): Observable<User> {
    const url = subsToUrl("/users", {}, {});
    return this.http.post<User>(url, dto);
  }

  getUserProfileByID(userId: string): Observable<User> {
    const url = subsToUrl("/users/{userId}", { userId: userId }, {});
    return this.http.get<User>(url);
  }

}

