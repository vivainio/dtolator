// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from "@angular/common/http";
import { Injectable } from "@angular/core";
import { Observable } from "rxjs";
import { lastValueFrom } from "rxjs";
import { subsToUrl } from "./subs-to-url.func";
import {
  PostMessageDto,
} from "./dto";

@Injectable({ providedIn: "root" })
export class ChatApi {
  constructor(private http: HttpClient) {}

  fetchMessages(queryParams?: { id?: string }): Promise<void> {
    const url = subsToUrl("/chat/fetch", {}, queryParams || {});
    return lastValueFrom(this.http.get<void>(url));
  }

  pushMessages(dto: PostMessageDto): Promise<void> {
    const url = subsToUrl("/chat/push", {}, {});
    return lastValueFrom(this.http.post<void>(url, dto));
  }

  getMqttConnection(): Promise<void> {
    const url = subsToUrl("/chat/mqtt", {}, {});
    return lastValueFrom(this.http.get<void>(url));
  }

}

