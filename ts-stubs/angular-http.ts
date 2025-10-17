// Angular HTTP stubs for TypeScript checking
import { Observable } from './rxjs-stubs';

export interface HttpRequest<T = any> {
  body: T | null;
  headers: any;
  method: string;
  url: string;
}

export interface HttpResponse<T = any> {
  body: T | null;
  headers: any;
  status: number;
  statusText: string;
  url: string | null;
}

export interface HttpErrorResponse extends HttpResponse<any> {
  error: any | null;
  message: string;
  name: string;
}

export class HttpHeaders {
  constructor(headers?: string | { [name: string]: string | string[] }) {}
  
  has(name: string): boolean {
    return false;
  }
  
  get(name: string): string | null {
    return null;
  }
  
  keys(): string[] {
    return [];
  }
  
  getAll(name: string): string[] | null {
    return null;
  }
  
  append(name: string, value: string | string[]): HttpHeaders {
    return new HttpHeaders();
  }
  
  set(name: string, value: string | string[]): HttpHeaders {
    return new HttpHeaders();
  }
  
  delete(name: string, value?: string | string[]): HttpHeaders {
    return new HttpHeaders();
  }
}

export class HttpClient {
  get<T>(url: string, options?: any): Observable<T> {
    return new Observable<T>();
  }
  
  post<T>(url: string, body: any, options?: any): Observable<T> {
    return new Observable<T>();
  }
  
  put<T>(url: string, body: any, options?: any): Observable<T> {
    return new Observable<T>();
  }
  
  delete<T>(url: string, options?: any): Observable<T> {
    return new Observable<T>();
  }
  
  patch<T>(url: string, body: any, options?: any): Observable<T> {
    return new Observable<T>();
  }
}
