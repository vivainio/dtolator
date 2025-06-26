// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { lastValueFrom } from "rxjs";
import { map } from "rxjs/operators";
import { z } from "zod";
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

  listAllUsers(): Promise<User[]> {
    const url = subsToUrl("/users", {}, {});
    return lastValueFrom(this.http.get<User[]>(url)
      .pipe(
        map(response => z.array(UserSchema).parse(response))
      ));
  }

  createNewUser(dto: CreateUserRequest): Promise<ApiResponse> {
    const url = subsToUrl("/users", {}, {});
    return lastValueFrom(this.http.post<ApiResponse>(url, dto)
      .pipe(
        map(response => ApiResponseSchema.parse(response))
      ));
  }

  getUserByID(userId: number): Promise<User> {
    const url = subsToUrl("/users/{userId}", { userId: userId }, {});
    return lastValueFrom(this.http.get<User>(url)
      .pipe(
        map(response => UserSchema.parse(response))
      ));
  }

  updateUserProfile(userId: number, dto: CreateUserRequest): Promise<ApiResponse> {
    const url = subsToUrl("/users/{userId}", { userId: userId }, {});
    return lastValueFrom(this.http.put<ApiResponse>(url, dto)
      .pipe(
        map(response => ApiResponseSchema.parse(response))
      ));
  }

  deleteUserAccount(userId: number): Promise<ApiResponse> {
    const url = subsToUrl("/users/{userId}", { userId: userId }, {});
    return lastValueFrom(this.http.delete<ApiResponse>(url)
      .pipe(
        map(response => ApiResponseSchema.parse(response))
      ));
  }

  activateUserAccount(userId: number): Promise<ApiResponse> {
    const url = subsToUrl("/users/{userId}/activate", { userId: userId }, {});
    return lastValueFrom(this.http.post<ApiResponse>(url, null)
      .pipe(
        map(response => ApiResponseSchema.parse(response))
      ));
  }

  deactivateUserAccount(userId: number): Promise<ApiResponse> {
    const url = subsToUrl("/users/{userId}/deactivate", { userId: userId }, {});
    return lastValueFrom(this.http.post<ApiResponse>(url, null)
      .pipe(
        map(response => ApiResponseSchema.parse(response))
      ));
  }

}

