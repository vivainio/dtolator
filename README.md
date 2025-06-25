# dtolator

**dtolator** (Data Type Translator) is a Rust command-line tool that converts OpenAPI schema JSON files to Zod schema definitions, TypeScript interfaces, Pydantic BaseModel classes, or API endpoint types.

## Features

- ✅ Convert OpenAPI 3.x schemas to Zod schemas with validation
- ✅ Convert OpenAPI 3.x schemas to TypeScript interfaces
- ✅ Convert OpenAPI 3.x schemas to Pydantic BaseModel classes
- ✅ Generate API endpoint types for type-safe client development
- ✅ Support for complex types (objects, arrays, enums, unions)
- ✅ Support for OpenAPI composition keywords (`allOf`, `oneOf`, `anyOf`)
- ✅ Support for validation constraints (min/max, length, patterns, formats)
- ✅ Support for nullable types
- ✅ Support for schema references (`$ref`)
- ✅ Extract path parameters, query parameters, and request/response types
- ✅ Command-line interface with flexible output options

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
dtolator -i schema.json
```

Generate TypeScript interfaces to stdout:
```bash
dtolator -i schema.json --typescript
```

Generate API endpoint types to stdout:
```bash
dtolator -i schema.json --endpoints
```

Generate Angular API services to stdout:
```bash
dtolator -i schema.json --angular
```

Generate TypeScript interfaces to directory:
```bash
dtolator -i schema.json -o ./output
```

Generate Zod schemas + TypeScript interfaces to directory:
```bash
dtolator -i schema.json -o ./output --zod
```

Generate Angular API services to directory:
```bash
dtolator -i schema.json -o ./output --angular
```

Generate Angular API services with Zod validation to directory:
```bash
dtolator -i schema.json -o ./output --angular --zod
```

Generate Pydantic models to stdout:
```bash
dtolator -i schema.json --pydantic
```

Generate Pydantic models to directory:
```bash
dtolator -i schema.json -o ./output --pydantic
```

### Command Line Options

```
Convert OpenAPI schema JSON files to Zod schema definitions or TypeScript interfaces

Usage: dtolator [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>      Input OpenAPI schema JSON file
  -o, --output <OUTPUT>    Output directory path (if specified, writes dto.ts and optionally schema.ts files)
  -t, --typescript         Generate TypeScript interfaces instead of Zod schemas (when not using output directory)
  -z, --zod                Generate Zod schemas (creates schema.ts and makes dto.ts import from it)
  -a, --angular            Generate Angular API services (creates multiple service files and utilities)
      --pydantic           Generate Pydantic BaseModel classes for Python
  -e, --endpoints          Generate API endpoint types from OpenAPI paths
  -p, --pretty             Pretty print the output
  -h, --help               Print help
  -V, --version            Print version
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
./target/release/dtolator -i simple-sample.json

# Test with TypeScript interfaces
./target/release/dtolator -i simple-sample.json --typescript

# Test with API endpoints generation
./target/release/dtolator -i full-sample.json --endpoints

# Generate complete type-safe API client setup
./target/release/dtolator -i full-sample.json --typescript -o types.ts
./target/release/dtolator -i full-sample.json --endpoints -o api-endpoints.ts
./target/release/dtolator -i full-sample.json -o schemas.ts  # Zod schemas
```

### Complete Project Setup

For a production-ready type-safe API setup, generate all three outputs:

```bash
# 1. Generate TypeScript interfaces for data types
dtolator -i your-api.json --typescript -o src/types/api-types.ts

# 2. Generate API endpoint definitions  
dtolator -i your-api.json --endpoints -o src/types/api-endpoints.ts

# 3. Generate Zod schemas for runtime validation
dtolator -i your-api.json -o src/schemas/api-schemas.ts
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

#### 4. API Endpoints (Type-Safe Nested Objects)

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

#### 5. Angular Services (Nested Object Support)

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
dtolator -i simple-sample.json --typescript

# Test with complex deep nesting
dtolator -i full-sample.json --zod

# Generate complete type-safe setup with nesting
dtolator -i full-sample.json --angular -o ./output-dir
```

The generated code maintains complete type safety and validation for all nested structures, making it easy to work with complex API responses and requests in your applications. 