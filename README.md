# dtolator

**dtolator** (Data Type Translator) is a Rust command-line tool that converts OpenAPI schema JSON files to Zod schema definitions, TypeScript interfaces, Pydantic BaseModel classes, or API endpoint types.

## Features

- ✅ **Multiple Input Types**: OpenAPI 3.x schemas, plain JSON, and JSON Schema files
- ✅ **Multiple Output Formats**: Zod schemas, TypeScript interfaces, Pydantic models, Python TypedDict, C# classes, JSON Schema
- ✅ **Angular Integration**: Generate complete Angular API services with observables or promises
- ✅ **Type-Safe API Clients**: Generate endpoint types for compile-time safety
- ✅ **Runtime Validation**: Zod schema generation with full validation support
- ✅ **Complex Type Support**: Objects, arrays, enums, unions, nested structures
- ✅ **OpenAPI Composition**: Full support for `allOf`, `oneOf`, `anyOf` keywords
- ✅ **Validation Constraints**: min/max, length, patterns, formats, required fields
- ✅ **Nullable Types**: Proper handling of optional and nullable properties
- ✅ **Schema References**: Full `$ref` resolution and cross-references
- ✅ **API Extraction**: Path parameters, query parameters, request/response types
- ✅ **Flexible Output**: stdout, single files, or complete directory structures
- ✅ **JSON Schema Generation**: Convert OpenAPI/JSON to standardized JSON Schema format
- ✅ **Debug Support**: Detailed debug output for troubleshooting generation issues

## Installation

### Build from source

```bash
git clone <repository-url>
cd dtolator
cargo build --release
```

The binary will be available at `target/release/dtolator`.

## Usage

### Basic Usage

Generate Zod schemas to stdout (default):
```bash
dtolator --from-openapi schema.json
```

Generate TypeScript interfaces to stdout:
```bash
dtolator --from-openapi schema.json --typescript
```

Generate API endpoint types to stdout:
```bash
dtolator --from-openapi schema.json --endpoints
```

Generate Angular API services to stdout:
```bash
dtolator --from-openapi schema.json --angular
```

Generate TypeScript interfaces to directory:
```bash
dtolator --from-openapi schema.json -o ./output
```

Generate Zod schemas + TypeScript interfaces to directory:
```bash
dtolator --from-openapi schema.json -o ./output --zod
```

Generate Angular API services to directory:
```bash
dtolator --from-openapi schema.json -o ./output --angular
```

Generate Angular API services with Zod validation to directory:
```bash
dtolator --from-openapi schema.json -o ./output --angular --zod
```

Generate Angular API services using Promises to directory:
```bash
dtolator --from-openapi schema.json -o ./output --angular --promises
```

Generate Angular API services with Promises and Zod validation to directory:
```bash
dtolator --from-openapi schema.json -o ./output --angular --promises --zod
```

Generate Pydantic models to stdout:
```bash
dtolator --from-openapi schema.json --pydantic
```

Generate Pydantic models to directory:
```bash
dtolator --from-openapi schema.json -o ./output --pydantic
```

Generate Python TypedDict definitions to stdout:
```bash
dtolator --from-openapi schema.json --python-dict
```

Generate Python TypedDict definitions to directory:
```bash
dtolator --from-openapi schema.json -o ./output --python-dict
```

Generate C# classes to stdout:
```bash
dtolator --from-openapi schema.json --dotnet
```

Generate C# classes to directory:
```bash
dtolator --from-openapi schema.json -o ./output --dotnet
```

Generate JSON Schema from OpenAPI:
```bash
dtolator --from-openapi schema.json --json-schema
```

Generate TypeScript from plain JSON:
```bash
dtolator --from-json data.json --typescript
```

Generate Zod schemas from JSON Schema:
```bash
dtolator --from-json-schema schema.json --zod
```

### Input Types

dtolator supports three different input types for maximum flexibility:

#### 1. OpenAPI Schema Input (`--from-openapi`)
Use this for OpenAPI 3.x specification files. This is the most common use case and provides the richest type information including API endpoints, request/response schemas, and validation rules.

```bash
# OpenAPI schema with full API endpoint information
dtolator --from-openapi api-spec.json --angular -o ./api-client
```

#### 2. Plain JSON Input (`--from-json`)
Use this to generate types directly from JSON data, similar to tools like quicktype.io. dtolator will infer the schema structure from your JSON data.

```bash
# Generate TypeScript from JSON data
dtolator --from-json user-data.json --typescript --root User

# Generate Pydantic models from JSON
dtolator --from-json api-response.json --pydantic --root ApiResponse
```

#### 3. JSON Schema Input (`--from-json-schema`)
Use this when you already have a JSON Schema and want to generate code from it. This is useful for converting between different schema formats or when working with existing JSON Schema definitions.

```bash
# Convert JSON Schema to Zod
dtolator --from-json-schema user.schema.json --zod

# Convert JSON Schema to TypeScript
dtolator --from-json-schema product.schema.json --typescript
```

**Note:** The `--root` option is only used with `--from-json` to specify the name of the root type when generating from plain JSON data.

### Command Line Options

```
Convert OpenAPI schema JSON files to Zod schema definitions or TypeScript interfaces

Usage: dtolator [OPTIONS] <INPUT_TYPE>

Input Types (exactly one required):
      --from-openapi <FILE>     Input OpenAPI schema JSON file
      --from-json <FILE>        Input plain JSON file (for generating DTOs like quicktype.io)
      --from-json-schema <FILE> Input JSON Schema file (for generating DTOs from JSON Schema)

Options:
      --root <NAME>             Name for the root class/interface when using --from-json (default: Root)
  -o, --output <OUTPUT>         Output directory path (if specified, writes dto.ts and optionally schema.ts files)
  -t, --typescript              Generate TypeScript interfaces instead of Zod schemas (when not using output directory)
  -z, --zod                     Generate Zod schemas (creates schema.ts and makes dto.ts import from it)
  -a, --angular                 Generate Angular API services (creates multiple service files and utilities)
      --promises                Generate promises using lastValueFrom instead of Observables (only works with --angular)
      --pydantic                Generate Pydantic BaseModel classes for Python
      --python-dict             Generate Python TypedDict definitions
      --dotnet                  Generate C# classes with System.Text.Json serialization
      --json-schema             Generate JSON Schema output
  -e, --endpoints               Generate API endpoint types from OpenAPI paths

      --debug                   Enable debug output
  -h, --help                    Print help
  -V, --version                 Print version
```

## Angular Integration

dtolator provides first-class support for Angular applications by generating type-safe API services directly from your OpenAPI specifications. This eliminates the need to manually write API service code and ensures your frontend stays in sync with your backend API.

### Quick Start

Generate Angular services from your OpenAPI schema:

```bash
# Basic Angular services (returns Observables)
dtolator --from-openapi your-api-spec.json -o ./src/app/api --angular

# Angular services with Zod validation
dtolator --from-openapi your-api-spec.json -o ./src/app/api --angular --zod

# Angular services returning Promises
dtolator --from-openapi your-api-spec.json -o ./src/app/api --angular --promises

# Angular services with Promises and Zod validation
dtolator --from-openapi your-api-spec.json -o ./src/app/api --angular --promises --zod
```

### What Gets Generated

When you run dtolator with the `--angular` flag, it generates a complete Angular API client including:

- **TypeScript interfaces** (`dto.ts`) - All your API data types
- **Zod schemas** (`schema.ts`) - Runtime validation schemas (when using `--zod`)
- **Service files** (`*-api.ts`) - Injectable Angular services for each API tag
- **Utility functions** (`subs-to-url.func.ts`) - URL building helpers
- **Barrel exports** (`index.ts`) - Clean imports for your application

### Observable vs Promise APIs

dtolator supports both Observable-based and Promise-based APIs:

#### Observable-based Services (Default)

```typescript
// Generated service (default behavior)
@Injectable({ providedIn: "root" })
export class UsersApi {
  constructor(private http: HttpClient) {}

  listAllUsers(): Observable<User[]> {
    const url = subsToUrl("/users", {}, {});
    return this.http.get<User[]>(url);
  }

  createUser(dto: CreateUserRequest): Observable<ApiResponse> {
    const url = subsToUrl("/users", {}, {});
    return this.http.post<ApiResponse>(url, dto);
  }
}
```

#### Promise-based Services (with `--promises`)

```typescript
// Generated service with --promises flag
@Injectable({ providedIn: "root" })
export class UsersApi {
  constructor(private http: HttpClient) {}

  listAllUsers(): Promise<User[]> {
    const url = subsToUrl("/users", {}, {});
    return lastValueFrom(this.http.get<User[]>(url));
  }

  createUser(dto: CreateUserRequest): Promise<ApiResponse> {
    const url = subsToUrl("/users", {}, {});
    return lastValueFrom(this.http.post<ApiResponse>(url, dto));
  }
}
```

### Zod Validation Integration

When using the `--zod` flag, all API responses are automatically validated at runtime:

#### With Observables + Zod

```typescript
@Injectable({ providedIn: "root" })
export class UsersApi {
  constructor(private http: HttpClient) {}

  listAllUsers(): Observable<User[]> {
    const url = subsToUrl("/users", {}, {});
    return this.http.get<User[]>(url)
      .pipe(
        map(response => z.array(UserSchema).parse(response))
      );
  }
}
```

#### With Promises + Zod

```typescript
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
}
```

### Using Generated Services in Components

#### Observable Pattern

```typescript
import { Component, OnInit } from '@angular/core';
import { Observable } from 'rxjs';
import { UsersApi, User } from './api';

@Component({
  selector: 'app-users',
  template: `
    <div *ngFor="let user of users$ | async">
      {{ user.name }} - {{ user.email }}
    </div>
  `
})
export class UsersComponent implements OnInit {
  users$!: Observable<User[]>;

  constructor(private usersApi: UsersApi) {}

  ngOnInit() {
    this.users$ = this.usersApi.listAllUsers();
  }

  async createUser(userData: CreateUserRequest) {
    // Even with Observable services, you can use lastValueFrom for one-off operations
    this.usersApi.createUser(userData).subscribe({
      next: (response) => console.log('User created:', response),
      error: (error) => console.error('Error:', error)
    });
  }
}
```

#### Promise Pattern

```typescript
import { Component, OnInit } from '@angular/core';
import { UsersApi, User, CreateUserRequest } from './api';

@Component({
  selector: 'app-users',
  template: `
    <div *ngFor="let user of users">
      {{ user.name }} - {{ user.email }}
    </div>
  `
})
export class UsersComponent implements OnInit {
  users: User[] = [];

  constructor(private usersApi: UsersApi) {}

  async ngOnInit() {
    try {
      this.users = await this.usersApi.listAllUsers();
    } catch (error) {
      console.error('Error loading users:', error);
    }
  }

  async createUser(userData: CreateUserRequest) {
    try {
      const response = await this.usersApi.createUser(userData);
      console.log('User created:', response);
      // Refresh the list
      this.users = await this.usersApi.listAllUsers();
    } catch (error) {
      console.error('Error creating user:', error);
    }
  }
}
```

### Required Dependencies

Add these dependencies to your Angular project:

```json
{
  "dependencies": {
    "@angular/common": "^17.0.0",
    "@angular/core": "^17.0.0",
    "rxjs": "^7.8.0"
  }
}
```

If using Zod validation (recommended):

```json
{
  "dependencies": {
    "zod": "^4.1.12"
  }
}
```

### Advanced Features

#### Error Handling with Zod

When Zod validation is enabled, you get automatic runtime type checking:

```typescript
// This will throw a ZodError if the API returns unexpected data
try {
  const users = await this.usersApi.listAllUsers();
  // users is guaranteed to match User[] schema
} catch (error) {
  if (error instanceof ZodError) {
    console.error('API returned invalid data:', error.errors);
  } else {
    console.error('Network or other error:', error);
  }
}
```

#### Path Parameters and Query Parameters

dtolator automatically handles path and query parameters:

```typescript
// Generated method with path and query parameters
async getUsersByStatus(
  status: string,
  queryParams?: { page?: number; limit?: number }
): Promise<User[]> {
  const url = subsToUrl("/users/status/{status}", { status }, queryParams || {});
  return lastValueFrom(this.http.get<User[]>(url));
}

// Usage
const activeUsers = await this.usersApi.getUsersByStatus('active', { 
  page: 1, 
  limit: 10 
});
```

#### Request Body Handling

For POST/PUT requests with request bodies:

```typescript
// Generated method with request body
async updateUser(userId: number, dto: UpdateUserRequest): Promise<User> {
  const url = subsToUrl("/users/{userId}", { userId }, {});
  return lastValueFrom(this.http.put<User>(url, dto));
}

// Usage
const updatedUser = await this.usersApi.updateUser(123, {
  name: "John Doe",
  email: "john@example.com"
});
```

### Best Practices

- **Choose Observable or Promise based on your needs:** Use Observables for reactive programming and data streams with operators like `map`, `filter`, `debounce`. Use Promises for simple async operations and async/await support.
- **Enable Zod validation for production:** `dtolator --from-openapi api-spec.json -o ./src/app/api --angular --promises --zod`
- **Import specific services and types:** `import { UsersApi, User } from './api'` instead of wildcard imports
- **Work with Angular HTTP Interceptors:** Generated services automatically integrate with interceptors for authentication and logging

```typescript
// Interceptors work automatically with generated services
@Injectable()
export class AuthInterceptor implements HttpInterceptor {
  intercept(req: HttpRequest<any>, next: HttpHandler): Observable<HttpEvent<any>> {
    const authReq = req.clone({
      headers: req.headers.set('Authorization', `Bearer ${this.getToken()}`)
    });
    return next.handle(authReq);
  }
}

const users = await this.usersApi.listAllUsers(); // Includes auth header
```

## Examples

### Sample Files

dtolator includes two sample OpenAPI specifications:

- **`simple-sample.json`** - Basic example with User schema (great for learning)
- **`full-sample.json`** - Comprehensive e-commerce API with Users, Products, Orders, and multiple endpoints (real-world example)

### Sample OpenAPI Schema

```json
{
  "openapi": "3.0.3",
  "info": {
    "title": "Sample API",
    "version": "1.0.0"
  },
  "components": {
    "schemas": {
      "User": {
        "type": "object",
        "required": ["id", "email", "name"],
        "properties": {
          "id": {
            "type": "integer"
          },
          "email": {
            "type": "string",
            "format": "email"
          },
          "name": {
            "type": "string",
            "minLength": 1,
            "maxLength": 100
          },
          "status": {
            "type": "string",
            "enum": ["active", "inactive", "pending"]
          }
        }
      }
    }
  }
}
```

### Generated Zod Schema

```typescript
import { z } from 'zod';

export const User = z.object({
  id: z.number().int(),
  email: z.string().email(),
  name: z.string().min(1).max(100),
  status: z.enum(["active", "inactive", "pending"]).optional()
});

export type User = z.infer<typeof User>;
```

### Generated TypeScript Interface

```typescript
// Generated TypeScript interfaces from OpenAPI schema
// Do not modify this file manually

export interface User {
  id: number;
  email: string;
  name: string;
  status?: "active" | "inactive" | "pending";
}
```

### Generated Pydantic Models

```python
# Generated Pydantic models from OpenAPI schema
# Do not modify this file manually

from datetime import date, datetime
from enum import Enum
from typing import Any, Dict, List, Literal, Optional, Union
from uuid import UUID

from pydantic import BaseModel, EmailStr, Field, HttpUrl

class User(BaseModel):
    id: int
    email: EmailStr
    name: str = Field(min_length=1, max_length=100)
    status: Optional[Literal["active", "inactive", "pending"]] = None
```

### Generated Python TypedDict Definitions

```python
# Generated Python TypedDict definitions from OpenAPI schema
# Do not modify this file manually

from datetime import date, datetime
from enum import Enum
from typing import Any, Dict, List, Literal, Optional, Union
from typing_extensions import TypedDict
from uuid import UUID

class UserRequired(TypedDict):
    id: int
    email: str
    name: str

class User(UserRequired, total=False):
    age: int
    isActive: bool
    tags: List[str]
    status: Literal["active", "inactive", "pending"]
    profile: UserProfile
    address: Address
```

### Generated C# Classes

```csharp
// Generated C# classes from OpenAPI schema
// Do not modify this file manually

using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;

namespace GeneratedApiModels;

public class User
{
    [JsonPropertyName("id")]
    public int Id { get; set; }
    
    [JsonPropertyName("email")]
    public string Email { get; set; }
    
    [JsonPropertyName("name")]
    public string Name { get; set; }
    
    [JsonPropertyName("age")]
    public int? Age { get; set; }
    
    [JsonPropertyName("isActive")]
    public bool? IsActive { get; set; }
    
    [JsonPropertyName("tags")]
    public List<string> Tags { get; set; }
    
    [JsonPropertyName("status")]
    public string Status { get; set; }
    
    [JsonPropertyName("profile")]
    public UserProfile Profile { get; set; }
    
    [JsonPropertyName("address")]
    public Address Address { get; set; }
}

public enum UserStatus
{
    [JsonPropertyName("active")]
    Active,
    
    [JsonPropertyName("inactive")]
    Inactive,
    
    [JsonPropertyName("pending")]
    Pending
}
```

### Generated API Endpoints

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

## Supported OpenAPI Features

### Basic Types
- `string` (with format validation for Zod: email, url, uuid, date, datetime)
- `number` and `integer` (with min/max constraints)
- `boolean`
- `array` (with item type validation)
- `object` (with property validation)

### Advanced Features
- **Enums**: Converted to string literal unions
- **References**: `$ref` to other schemas
- **Nullable types**: Optional null unions
- **Composition**: `allOf`, `oneOf`, `anyOf`
- **Validation constraints**: minLength, maxLength, minimum, maximum, pattern

### Validation (Zod only)
- String format validation (email, URL, UUID, date)
- Numeric constraints (min, max, integer)
- String length constraints
- Regular expression patterns

### API Endpoint Generation
- **Path parameters**: Extracted from URL patterns like `/users/{userId}`
- **Query parameters**: Optional query string parameters with proper types
- **Request body**: Typed request bodies for POST/PUT/PATCH operations
- **Response types**: Strongly typed response data
- **HTTP methods**: Support for GET, POST, PUT, DELETE, PATCH
- **Helper types**: Utility types for extracting params, queries, requests, and responses

## Usage Examples

Test the application with the provided samples:

```bash
# Build the project
cargo build --release

# Test with simple example (Zod schemas - default)
./target/release/dtolator --from-openapi simple-sample.json

# Test with TypeScript interfaces
./target/release/dtolator --from-openapi simple-sample.json --typescript

# Test with API endpoints generation
./target/release/dtolator --from-openapi full-sample.json --endpoints

# Test with Python TypedDict generation
./target/release/dtolator --from-openapi simple-sample.json --python-dict

# Test with C# classes generation
./target/release/dtolator --from-openapi simple-sample.json --dotnet

# Generate complete type-safe API client setup
./target/release/dtolator --from-openapi full-sample.json --typescript -o ./output
./target/release/dtolator --from-openapi full-sample.json --endpoints -o ./output
./target/release/dtolator --from-openapi full-sample.json -o ./output  # Zod schemas

# Test JSON to TypeScript conversion
./target/release/dtolator --from-json test-data-simple.json --typescript

# Test JSON Schema to Zod conversion
./target/release/dtolator --from-json-schema generated-schema.json --zod
```

### Complete Project Setup

For a production-ready type-safe API setup, generate all three outputs:

```bash
# 1. Generate TypeScript interfaces for data types
dtolator --from-openapi your-api.json --typescript -o ./src/types

# 2. Generate API endpoint definitions  
dtolator --from-openapi your-api.json --endpoints -o ./src/types

# 3. Generate Zod schemas for runtime validation
dtolator --from-openapi your-api.json -o ./src/schemas --zod
```

This gives you:
- **TypeScript interfaces** for compile-time type checking
- **API endpoint types** for type-safe API calls  
- **Zod schemas** for runtime validation

See the "How to Use Generated ApiEndpoints" section above for detailed implementation examples.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Nested Objects Support

dtolator provides excellent support for complex nested object structures in all output formats. Here are examples showing how nested objects are handled:

### Complex Nested Schema Example

From our `full-sample.json`, here's a complex nested structure:

```json
{
  "User": {
    "type": "object",
    "required": ["id", "email", "profile"],
    "properties": {
      "id": { "type": "string", "format": "uuid" },
      "email": { "type": "string", "format": "email" },
      "profile": { "$ref": "#/components/schemas/UserProfile" },
      "preferences": { "$ref": "#/components/schemas/UserPreferences" },
      "roles": {
        "type": "array",
        "items": { "$ref": "#/components/schemas/UserRole" }
      }
    }
  },
  "UserProfile": {
    "type": "object",
    "required": ["firstName", "lastName"],
    "properties": {
      "firstName": { "type": "string" },
      "lastName": { "type": "string" },
      "address": { "$ref": "#/components/schemas/Address" },
      "avatar": { "$ref": "#/components/schemas/ImageUrl" }
    }
  },
  "UserPreferences": {
    "type": "object",
    "properties": {
      "notifications": { "$ref": "#/components/schemas/NotificationSettings" },
      "language": { "type": "string", "enum": ["en", "es", "fr"] }
    }
  }
}
```

### Generated Output Examples

#### 1. TypeScript Interfaces (Deep Nesting)

```typescript
export interface User {
  id: string;
  email: string;
  profile: UserProfile;           // ← Nested object reference
  preferences?: UserPreferences;  // ← Optional nested object
  roles?: UserRole[];            // ← Array of nested objects
}

export interface UserProfile {
  firstName: string;
  lastName: string;
  address?: Address;             // ← Deeply nested object
  avatar?: ImageUrl;             // ← Another nested object
}

export interface UserPreferences {
  notifications?: NotificationSettings; // ← Nested settings object
  language?: "en" | "es" | "fr";
}

export interface Address {
  street: string;
  city: string;
  country: string;
  // ... more fields
}
```

#### 2. Zod Schemas (With Cross-References)

```typescript
export const UserSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
  profile: UserProfileSchema,              // ← Schema reference
  preferences: UserPreferencesSchema.optional(), // ← Optional nested validation
  roles: z.array(UserRoleSchema).optional()      // ← Array validation with nested schema
});

export const UserProfileSchema = z.object({
  firstName: z.string().min(1).max(50),
  lastName: z.string().min(1).max(50),
  address: AddressSchema.optional(),       // ← Nested validation
  avatar: ImageUrlSchema.optional()        // ← Complex nested validation
});

export const AddressSchema = z.object({
  street: z.string().min(1).max(100),
  city: z.string().min(1).max(50),
  country: z.string().regex(/^[A-Z]{2}$/), // ← Deep validation rules
});
```

#### 3. Pydantic Models (With Validation)

```python
class User(BaseModel):
    id: UUID
    email: EmailStr
    profile: UserProfile              # ← Nested model reference
    preferences: Optional[UserPreferences] = None  # ← Optional nested model
    roles: Optional[List[UserRole]] = None         # ← List of nested models

class UserProfile(BaseModel):
    firstName: str = Field(min_length=1, max_length=50)
    lastName: str = Field(min_length=1, max_length=50)
    address: Optional[Address] = None    # ← Deeply nested model
    avatar: Optional[ImageUrl] = None    # ← Complex nested validation

class Address(BaseModel):
    street: str = Field(min_length=1, max_length=100)
    city: str = Field(min_length=1, max_length=50)
    country: str = Field(regex=r"^[A-Z]{2}$")  # ← Deep validation
```

#### 4. Python TypedDict (Lightweight Type Hints)

```python
class UserRequired(TypedDict):
    id: str
    email: str
    profile: UserProfile              # ← Nested TypedDict reference

class User(UserRequired, total=False):
    preferences: UserPreferences      # ← Optional nested TypedDict
    roles: List[UserRole]             # ← List of nested types

class UserProfileRequired(TypedDict):
    firstName: str
    lastName: str

class UserProfile(UserProfileRequired, total=False):
    address: Address                  # ← Deeply nested TypedDict
    avatar: ImageUrl                  # ← Complex nested structure

class AddressRequired(TypedDict):
    street: str
    city: str
    country: str                      # ← Clean type definitions

class Address(AddressRequired, total=False):
    state: Optional[str]
    postalCode: Optional[str]
```

#### 5. C# Classes (System.Text.Json Serialization)

```csharp
namespace GeneratedApiModels;

public class User
{
    [JsonPropertyName("id")]
    public Guid Id { get; set; }
    
    [JsonPropertyName("email")]
    public string Email { get; set; }
    
    [JsonPropertyName("profile")]
    public UserProfile Profile { get; set; }          // ← Nested class reference
    
    [JsonPropertyName("preferences")]
    public UserPreferences Preferences { get; set; }   // ← Optional nested class
    
    [JsonPropertyName("roles")]
    public List<UserRole> Roles { get; set; }         // ← List of nested enums
}

public class UserProfile
{
    [JsonPropertyName("firstName")]
    public string FirstName { get; set; }              // ← camelCase → PascalCase
    
    [JsonPropertyName("lastName")]
    public string LastName { get; set; }
    
    [JsonPropertyName("address")]
    public Address Address { get; set; }               // ← Deeply nested class
    
    [JsonPropertyName("avatar")]
    public ImageUrl Avatar { get; set; }               // ← Complex nested class
}

public class UserPreferences
{
    [JsonPropertyName("notifications")]
    public NotificationSettings Notifications { get; set; }  // ← Nested settings class
    
    [JsonPropertyName("language")]
    public string Language { get; set; }               // ← Enum as string
}

public class Address
{
    [JsonPropertyName("street")]
    public string Street { get; set; }
    
    [JsonPropertyName("city")]
    public string City { get; set; }
    
    [JsonPropertyName("country")]
    public string Country { get; set; }                // ← Clean C# properties
}

public enum UserRole
{
    [JsonPropertyName("customer")]
    Customer,                                           // ← JSON-friendly enum values
    
    [JsonPropertyName("admin")]
    Admin,
    
    [JsonPropertyName("moderator")]
    Moderator
}
```

#### 6. API Endpoints (Type-Safe Nested Objects)

```typescript
export type ApiEndpoints = {
  "POST /users": {
    request: CreateUserRequest;    // ← Request with nested objects
    response: User;               // ← Response with nested objects
  };
  "GET /users/{userId}": {
    params: { userId: string };
    response: User;               // ← Complex nested response
  };
};

// Usage with full type safety for nested objects:
const newUser = await api.call('POST /users', {
  body: {
    email: 'john@example.com',
    password: 'SecurePass123!',
    profile: {                    // ← Nested object in request
      firstName: 'John',
      lastName: 'Doe',
      address: {                  // ← Deeply nested object
        street: '123 Main St',
        city: 'Boston',
        country: 'US'
      }
    },
    preferences: {                // ← Optional nested object
      language: 'en',
      notifications: {            // ← Deeply nested settings
        email: true,
        push: false
      }
    }
  }
});
// newUser.profile.address.city is fully typed! ✅
```

#### 7. Angular Services (Nested Object Support)

```typescript
@Injectable({ providedIn: 'root' })
export class UserService {
  createUser(userData: CreateUserRequest): Observable<User> {
    // userData.profile.address.street is fully typed ✅
    // userData.preferences.notifications.email is fully typed ✅
    return this.http.post<User>('/users', userData);
  }

  getUser(userId: string): Observable<User> {
    return this.http.get<User>(`/users/${userId}`);
    // Response: user.profile.address.city is fully typed ✅
  }
}
```

### Key Nested Object Features

✅ **Deep Nesting**: Objects can be nested to any depth  
✅ **Cross-References**: `$ref` links between schemas work perfectly  
✅ **Array Nesting**: Arrays of nested objects are fully supported  
✅ **Optional Nesting**: Optional nested objects with proper null handling  
✅ **Validation Preservation**: All nested validation rules are maintained  
✅ **Circular References**: Handled safely (e.g., Product containing ProductSnapshot)  
✅ **Type Safety**: Full IntelliSense and compile-time checking for nested properties  

### Testing Nested Objects

```bash
# Test with simple nested objects
dtolator --from-openapi simple-sample.json --typescript

# Test with complex deep nesting
dtolator --from-openapi full-sample.json --zod

# Generate complete type-safe setup with nesting
dtolator --from-openapi full-sample.json --angular -o ./output-dir

# Test JSON to TypeScript with nested objects
dtolator --from-json test-data-complex.json --typescript

# Test JSON Schema with nested structures
dtolator --from-json-schema complex-schema.json --zod
```

The generated code maintains complete type safety and validation for all nested structures, making it easy to work with complex API responses and requests in your applications. 