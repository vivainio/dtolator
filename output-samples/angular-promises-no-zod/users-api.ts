// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { lastValueFrom } from "rxjs";
import { subsToUrl } from "./subs-to-url.func";
import {
  ApiResponse,
  CreateUserRequest,
  User,
} from "./dto";

@Injectable({ providedIn: "root" })
export class UsersApi {
  constructor(private http: HttpClient) {}

  listAllUsers(): Promise<User[]> {
    const url = subsToUrl("/users", {}, {});
    return lastValueFrom(this.http.get<User[]>(url));
  }

  createNewUser(dto: CreateUserRequest): Promise<ApiResponse> {
    const url = subsToUrl("/users", {}, {});
    return lastValueFrom(this.http.post<ApiResponse>(url, dto));
  }

  getUserByID(userId: number): Promise<User> {
    const url = subsToUrl("/users/{userId}", { userId: userId }, {});
    return lastValueFrom(this.http.get<User>(url));
  }

  updateUserProfile(userId: number, dto: CreateUserRequest): Promise<ApiResponse> {
    const url = subsToUrl("/users/{userId}", { userId: userId }, {});
    return lastValueFrom(this.http.put<ApiResponse>(url, dto));
  }

  deleteUserAccount(userId: number): Promise<ApiResponse> {
    const url = subsToUrl("/users/{userId}", { userId: userId }, {});
    return lastValueFrom(this.http.delete<ApiResponse>(url));
  }

  activateUserAccount(userId: number): Promise<ApiResponse> {
    const url = subsToUrl("/users/{userId}/activate", { userId: userId }, {});
    return lastValueFrom(this.http.post<ApiResponse>(url, null));
  }

  deactivateUserAccount(userId: number): Promise<ApiResponse> {
    const url = subsToUrl("/users/{userId}/deactivate", { userId: userId }, {});
    return lastValueFrom(this.http.post<ApiResponse>(url, null));
  }

}

