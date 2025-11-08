# Advanced Topics

## Generated API Endpoints

Generate API endpoint types using the `--endpoints` flag:

```bash
dtolator --from-openapi your-api.json --endpoints
dtolator --from-openapi your-api.json --endpoints -o ./output
```

This generates strongly-typed endpoint definitions that enable compile-time type safety for all API calls.

> **Note:** For Angular users, there is easier built-in support for generating complete API clients directly. See the [Angular Integration](../README.md#angular-integration) section in the README, or generate services with `dtolator --from-openapi your-api.json --angular -o ./output`. This approach handles routing, HTTP methods, and service injection automatically.

### Format

```typescript
// Generated API endpoint types from OpenAPI schema
// Do not modify this file manually

import { User, UserListResponse, CreateUserRequest } from './types';

export type ApiEndpoints = {
  "GET /users": {
    query?: {
      page?: number;
      limit?: number;
    };
    response: UserListResponse;
  };
  "POST /users": {
    request: CreateUserRequest;
    response: User;
  };
  "GET /users/{userId}": {
    params: {
      userId: string;
    };
    response: User;
  };
};

// Helper types for API client usage
export type ExtractEndpointParams<T extends keyof ApiEndpoints> = 
  ApiEndpoints[T] extends { params: infer P } ? P : never;

export type ExtractEndpointResponse<T extends keyof ApiEndpoints> = 
  ApiEndpoints[T] extends { response: infer R } ? R : never;

export type ExtractEndpointRequest<T extends keyof ApiEndpoints> = 
  ApiEndpoints[T] extends { request: infer R } ? R : never;

export type ExtractEndpointQuery<T extends keyof ApiEndpoints> = 
  ApiEndpoints[T] extends { query: infer Q } ? Q : never;
```

## How to Use Generated ApiEndpoints

The generated `ApiEndpoints` type provides complete type safety for your API calls. Here are practical examples:

### 1. Type-Safe Fetch Wrapper

```typescript
import { ApiEndpoints, ExtractEndpointResponse, ExtractEndpointParams } from './api-endpoints';

class TypedApiClient {
  constructor(private baseUrl: string) {}

  async call<K extends keyof ApiEndpoints>(
    endpoint: K,
    options: (
      ApiEndpoints[K] extends { request: infer R } ? { body: R } : {}
    ) & (
      ApiEndpoints[K] extends { params: infer P } ? { params: P } : {}
    ) & (
      ApiEndpoints[K] extends { query: infer Q } ? { query?: Q } : {}
    )
  ): Promise<ExtractEndpointResponse<K>> {
    
    const [method, pathTemplate] = endpoint.split(' ') as [string, string];
    
    // Replace path parameters
    let url = pathTemplate;
    if ('params' in options && options.params) {
      Object.entries(options.params as Record<string, string>).forEach(([key, value]) => {
        url = url.replace(`{${key}}`, encodeURIComponent(value));
      });
    }
    
    // Add query parameters
    if ('query' in options && options.query) {
      const searchParams = new URLSearchParams();
      Object.entries(options.query as Record<string, any>).forEach(([key, value]) => {
        if (value !== undefined) searchParams.append(key, String(value));
      });
      if (searchParams.toString()) url += `?${searchParams.toString()}`;
    }
    
    const response = await fetch(`${this.baseUrl}${url}`, {
      method,
      headers: { 'Content-Type': 'application/json' },
      body: 'body' in options ? JSON.stringify(options.body) : undefined,
    });
    
    if (!response.ok) throw new Error(`HTTP ${response.status}`);
    return response.json();
  }
}

// Usage examples with full type safety:
const api = new TypedApiClient('https://api.example.com/v2');

// ✅ GET /users with query parameters (fully typed)
const users = await api.call('GET /users', {
  query: { page: 1, limit: 20 }  // TypeScript knows these are the right query params
});
// users is automatically typed as UserListResponse

// ✅ GET /users/{userId} with path parameters (fully typed)
const user = await api.call('GET /users/{userId}', {
  params: { userId: '123' }  // TypeScript knows userId is required
});
// user is automatically typed as User

// ✅ POST /users with request body (fully typed)
const newUser = await api.call('POST /users', {
  body: {  // TypeScript knows this should be CreateUserRequest
    email: 'john@example.com',
    password: 'SecurePass123!',
    profile: { firstName: 'John', lastName: 'Doe' }
  }
});
// newUser is automatically typed as User

// ❌ TypeScript will catch these errors:
// api.call('GET /users', { body: {} });           // Error: GET doesn't accept body
// api.call('POST /users', {});                    // Error: POST requires body
// api.call('GET /users/{userId}', {});            // Error: Missing required params
```

### 2. Angular API Service Integration

```typescript
import { Injectable } from '@angular/core';
import { HttpClient, HttpParams } from '@angular/common/http';
import { Observable } from 'rxjs';
import { ApiEndpoints, ExtractEndpointResponse, ExtractEndpointParams, ExtractEndpointRequest, ExtractEndpointQuery } from './api-endpoints';

@Injectable({
  providedIn: 'root'
})
export class TypedApiService {
  private baseUrl = 'https://api.example.com/v2';

  constructor(private http: HttpClient) {}

  call<K extends keyof ApiEndpoints>(
    endpoint: K,
    options: (
      ApiEndpoints[K] extends { request: infer R } ? { body: R } : {}
    ) & (
      ApiEndpoints[K] extends { params: infer P } ? { params: P } : {}
    ) & (
      ApiEndpoints[K] extends { query: infer Q } ? { query?: Q } : {}
    )
  ): Observable<ExtractEndpointResponse<K>> {
    
    const [method, pathTemplate] = endpoint.split(' ') as [string, string];
    
    // Replace path parameters
    let url = pathTemplate;
    if ('params' in options && options.params) {
      Object.entries(options.params as Record<string, string>).forEach(([key, value]) => {
        url = url.replace(`{${key}}`, encodeURIComponent(value));
      });
    }
    
    // Add query parameters
    let httpParams = new HttpParams();
    if ('query' in options && options.query) {
      Object.entries(options.query as Record<string, any>).forEach(([key, value]) => {
        if (value !== undefined) {
          httpParams = httpParams.append(key, String(value));
        }
      });
    }
    
    const fullUrl = `${this.baseUrl}${url}`;
    const httpOptions = { params: httpParams };
    
    switch (method) {
      case 'GET':
        return this.http.get<ExtractEndpointResponse<K>>(fullUrl, httpOptions);
      case 'POST':
        return this.http.post<ExtractEndpointResponse<K>>(
          fullUrl, 
          'body' in options ? options.body : null, 
          httpOptions
        );
      case 'PUT':
        return this.http.put<ExtractEndpointResponse<K>>(
          fullUrl, 
          'body' in options ? options.body : null, 
          httpOptions
        );
      case 'DELETE':
        return this.http.delete<ExtractEndpointResponse<K>>(fullUrl, httpOptions);
      case 'PATCH':
        return this.http.patch<ExtractEndpointResponse<K>>(
          fullUrl, 
          'body' in options ? options.body : null, 
          httpOptions
        );
      default:
        throw new Error(`Unsupported HTTP method: ${method}`);
    }
  }
}

// Angular component using the service
import { Component, OnInit, Input } from '@angular/core';

@Component({
  selector: 'app-user-profile',
  template: `
    <div *ngIf="loading">Loading...</div>
    <div *ngIf="error" class="error">Error: {{ error }}</div>
    <div *ngIf="user && !loading" class="user-profile">
      <h1>{{ user.profile.firstName }} {{ user.profile.lastName }}</h1>
      <p>Email: {{ user.email }}</p>
      <p>Status: {{ user.status }}</p>
    </div>
  `
})
export class UserProfileComponent implements OnInit {
  @Input() userId!: string;
  
  user: ExtractEndpointResponse<'GET /users/{userId}'> | null = null;
  loading = false;
  error: string | null = null;

  constructor(private apiService: TypedApiService) {}

  ngOnInit() {
    this.loadUser();
  }

  loadUser() {
    this.loading = true;
    this.error = null;
    
    // Fully typed API call - TypeScript knows userId is required
    this.apiService.call('GET /users/{userId}', {
      params: { userId: this.userId }
    }).subscribe({
      next: (user) => {
        this.user = user; // TypeScript knows this is User type
        this.loading = false;
      },
      error: (err) => {
        this.error = err.message || 'Unknown error';
        this.loading = false;
      }
    });
  }

  // Example of creating a new user
  createUser(userData: ExtractEndpointRequest<'POST /users'>) {
    return this.apiService.call('POST /users', {
      body: userData // TypeScript ensures this matches CreateUserRequest
    });
  }

  // Example of getting users list with query parameters
  getUsers(page: number = 1, limit: number = 20) {
    return this.apiService.call('GET /users', {
      query: { page, limit } // TypeScript knows these are the correct query params
    });
  }
}
```

### 3. Type Utilities for Advanced Usage

```typescript
// Extract specific types from endpoints
type UserParams = ExtractEndpointParams<'GET /users/{userId}'>;
// Result: { userId: string }

type UserResponse = ExtractEndpointResponse<'GET /users/{userId}'>;
// Result: User

type CreateUserBody = ExtractEndpointRequest<'POST /users'>;
// Result: CreateUserRequest

type UsersQuery = ExtractEndpointQuery<'GET /users'>;
// Result: { page?: number; limit?: number } | never

// Filter endpoints by HTTP method
type GetEndpoints = {
  [K in keyof ApiEndpoints as K extends `GET ${string}` ? K : never]: ApiEndpoints[K]
};

type PostEndpoints = {
  [K in keyof ApiEndpoints as K extends `POST ${string}` ? K : never]: ApiEndpoints[K]
};
```

### 4. Testing with Mock Data

```typescript
// Type-safe mock API client for testing
class MockApiClient {
  private mocks = new Map<keyof ApiEndpoints, any>();

  mock<K extends keyof ApiEndpoints>(
    endpoint: K,
    response: ExtractEndpointResponse<K>
  ) {
    this.mocks.set(endpoint, response);
    return this;
  }

  async call<K extends keyof ApiEndpoints>(
    endpoint: K,
    options: any
  ): Promise<ExtractEndpointResponse<K>> {
    const mockResponse = this.mocks.get(endpoint);
    if (!mockResponse) throw new Error(`No mock for ${endpoint}`);
    return mockResponse;
  }
}

// Test example
const mockApi = new MockApiClient()
  .mock('GET /users/{userId}', {
    id: '123',
    email: 'test@example.com',
    profile: { firstName: 'Test', lastName: 'User' }
  });

const user = await mockApi.call('GET /users/{userId}', { params: { userId: '123' } });
// user is fully typed as User
```

