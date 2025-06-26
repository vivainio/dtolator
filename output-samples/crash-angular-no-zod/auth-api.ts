// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { lastValueFrom } from "rxjs";
import { subsToUrl } from "./subs-to-url.func";
import {
  AccessRequest,
  AccessResponse,
  OnBoardDto,
} from "./dto";

@Injectable({ providedIn: "root" })
export class AuthApi {
  constructor(private http: HttpClient) {}

  validateToken(): Promise<void> {
    const url = subsToUrl("/auth/validate", {}, {});
    return lastValueFrom(this.http.get<void>(url));
  }

  getAccess(dto: AccessRequest): Observable<AccessResponse> {
    const url = subsToUrl("/auth/access", {}, {});
    return this.http.post<AccessResponse>(url, dto);
  }

  login(): Promise<void> {
    const url = subsToUrl("/auth/login", {}, {});
    return lastValueFrom(this.http.post<void>(url, null));
  }

  onboard(dto: OnBoardDto): Promise<void> {
    const url = subsToUrl("/auth/onboard", {}, {});
    return lastValueFrom(this.http.post<void>(url, dto));
  }

}

