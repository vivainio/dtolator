# Angular API Generation Documentation

## Overview

This document outlines the practices and conventions for automatically generating TypeScript interfaces, DTOs, and Angular services from OpenAPI specifications.

## Using the dtolator Tool

### Command Line Usage

The dtolator tool now supports Angular API generation with the `--angular` flag:

```bash
# Generate Angular API services in a directory
dtolator --input openapi.json --output ./src/app/api --angular

# Generate Angular API services with Zod validation
dtolator --input openapi.json --output ./src/app/api --angular --zod

# Generate to stdout (single combined output)  
dtolator --input openapi.json --angular
```

### Generated Files Structure

When using `--angular` with an output directory, the following files are generated:

```
api-output/
├── dto.ts                    # TypeScript interfaces and enums
├── subs-to-url.func.ts      # URL building utility function
├── users-api.service.ts     # Service for Users tag
├── products-api.service.ts  # Service for Products tag (if exists)
└── index.ts                 # Barrel exports
```

When using `--angular --zod` with an output directory, additional validation is included:

```
api-output/
├── dto.ts                    # TypeScript type re-exports from schema.ts
├── schema.ts                 # Zod validation schemas with runtime validation
├── subs-to-url.func.ts      # URL building utility function
├── users-api.service.ts     # Service with built-in Zod validation
├── products-api.service.ts  # Service with built-in Zod validation
└── index.ts                 # Barrel exports
```

## Generated Angular Services

### Service Example

Given this OpenAPI specification:

```json
{
  "paths": {
    "/users": {
      "get": {
        "tags": ["Users"],
        "summary": "List All Users",
        "responses": {
          "200": {
            "content": {
              "application/json": {
                "schema": { "type": "array", "items": { "$ref": "#/components/schemas/User" } }
              }
            }
          }
        }
      },
      "post": {
        "tags": ["Users"], 
        "summary": "Create New User",
        "requestBody": {
          "content": {
            "application/json": {
              "schema": { "$ref": "#/components/schemas/CreateUserRequest" }
            }
          }
        },
        "responses": {
          "201": {
            "content": {
              "application/json": {
                "schema": { "$ref": "#/components/schemas/ApiResponse" }
              }
            }
          }
        }
      }
    },
    "/users/{userId}": {
      "get": {
        "tags": ["Users"],
        "summary": "Get User By ID", 
        "parameters": [
          {
            "name": "userId",
            "in": "path",
            "required": true,
            "schema": { "type": "integer" }
          }
        ],
        "responses": {
          "200": {
            "content": {
              "application/json": {
                "schema": { "$ref": "#/components/schemas/User" }
              }
            }
          }
        }
      }
    }
  }
}
```

The tool generates this Angular service:

```typescript
// users-api.service.ts
// Generated Angular service from OpenAPI schema
// Do not modify this file manually

import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { subsToUrl } from './subs-to-url.func';
import { ApiResponse, CreateUserRequest, User } from './dto';

@Injectable({ providedIn: 'root' })
export class UsersApiService {
  constructor(private http: HttpClient) {}

  listAllUsers(): Observable<unknown[]> {
    const url = subsToUrl('/users', {}, {});
    return this.http.get<unknown[]>(url);
  }

  createNewUser(dto: CreateUserRequest): Observable<ApiResponse> {
    const url = subsToUrl('/users', {}, {});
    return this.http.post<ApiResponse>(url, dto);
  }

  getUserByID(userId: number): Observable<User> {
    const url = subsToUrl('/users/{userId}', { userId: userId }, {});
    return this.http.get<User>(url);
  }
}
```

### With Zod Validation

When using the `--zod` flag, the generated service includes runtime validation:

```typescript
// users-api.service.ts
// Generated Angular service from OpenAPI schema with Zod validation
// Do not modify this file manually

import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { map } from 'rxjs/operators';
import { subsToUrl } from './subs-to-url.func';
import { ApiResponseSchema, ApiResponse, CreateUserRequestSchema, CreateUserRequest, UserSchema, User } from './dto';

@Injectable({ providedIn: 'root' })
export class UsersApiService {
  constructor(private http: HttpClient) {}

  listAllUsers(): Observable<unknown[]> {
    const url = subsToUrl('/users', {}, {});
    return this.http.get<unknown[]>(url)
      .pipe(
        map(response => z.array(UserSchema).parse(response))
      );
  }

  createNewUser(dto: CreateUserRequest): Observable<ApiResponse> {
    // Validate request body with Zod
    const validatedDto = CreateUserRequestSchema.parse(dto);

    const url = subsToUrl('/users', {}, {});
    return this.http.post<ApiResponse>(url, validatedDto)
      .pipe(
        map(response => ApiResponseSchema.parse(response))
      );
  }

  getUserByID(userId: number): Observable<User> {
    const url = subsToUrl('/users/{userId}', { userId: userId }, {});
    return this.http.get<User>(url)
      .pipe(
        map(response => UserSchema.parse(response))
      );
  }
}
```

### URL Building Utility

The generated `subs-to-url.func.ts` provides a utility function for building URLs with parameters:

```typescript
// subs-to-url.func.ts
// Generated utility function for URL building
// Do not modify this file manually

import { environment } from '@env/environment';

export function subsToUrl(
  url: string,
  params?: { [key: string]: string | number | boolean | null | undefined },
  queryParams?: { [key: string]: string | number | boolean | null | undefined }
): string {
  // Substitutes path parameters like /users/{userId} -> /users/123
  if (params) {
    for (const key in params) {
      if (params.hasOwnProperty(key)) {
        const regex = new RegExp(':' + key + '($|/)');
        url = url.replace(regex, params[key] + '$1');
      }
    }
  }
  
  // Adds query parameters like ?page=1&limit=10
  if (queryParams) {
    const qs = Object.keys(queryParams)
      .filter((key) => queryParams[key] !== null && queryParams[key] !== undefined)
      .map((key) => {
        const value = encodeURIComponent(queryParams[key]!);
        return `${key}=${value}`;
      })
      .join('&');
      
    if (qs.length > 0) {
      url += '?' + qs;
    }
  }

  // Supports both environment config and runtime injection
  const injectedConfig = (window as any).API_CONFIG;
  if (injectedConfig) {
    return injectedConfig.BACKEND_API_URL + url;
  }

  return environment.apiUrl + url;
}
```

### TypeScript DTOs

The generated `dto.ts` contains strongly-typed interfaces:

```typescript
// dto.ts
// Generated TypeScript interfaces from OpenAPI schema
// Do not modify this file manually

export interface User {
  id: number;
  email: string;
  name: string;
  age?: number;
  isActive?: boolean;
  tags?: string[];
  status?: "active" | "inactive" | "pending";
}

export interface CreateUserRequest {
  email: string;
  name: string;
  age?: number | null;
}

export interface ApiResponse {
  success: boolean;
  message?: string;
  data?: User;
}
```

### Index File

The `index.ts` provides convenient barrel exports:

```typescript
// index.ts  
// Generated index file for Angular services
// Do not modify this file manually

export * from './dto';
export * from './subs-to-url.func';
export * from './users-api.service';
```

## Service Usage in Angular Components

### Component Example

```typescript
import { Component, OnInit } from '@angular/core';
import { UsersApiService, User, CreateUserRequest } from '../api';

@Component({
  selector: 'app-user-list',
  template: `
    <div *ngFor="let user of users">
      <h3>{{ user.name }}</h3>
      <p>{{ user.email }}</p>
      <button (click)="loadUser(user.id)">View Details</button>
    </div>
    
    <button (click)="createUser()">Create New User</button>
  `
})
export class UserListComponent implements OnInit {
  users: User[] = [];

  constructor(private usersApi: UsersApiService) {}

  ngOnInit() {
    this.loadUsers();
  }

  loadUsers() {
    this.usersApi.listAllUsers().subscribe({
      next: (users) => this.users = users,
      error: (err) => console.error('Failed to load users:', err)
    });
  }

  loadUser(userId: number) {
    this.usersApi.getUserByID(userId).subscribe({
      next: (user) => console.log('User details:', user),
      error: (err) => console.error('Failed to load user:', err)
    });
  }

  createUser() {
    const newUser: CreateUserRequest = {
      email: 'john@example.com',
      name: 'John Doe',
      age: 30
    };

    this.usersApi.createNewUser(newUser).subscribe({
      next: (response) => {
        console.log('User created:', response);
        this.loadUsers(); // Refresh the list
      },
      error: (err) => console.error('Failed to create user:', err)
    });
  }
}
```

### Environment Configuration

Make sure your `environment.ts` includes the API URL:

```typescript
// environment.ts
export const environment = {
  production: false,
  apiUrl: 'https://api.example.com/v1'
};
```

## Current Functionality

### Generated Files Structure
```
gen-dir/
├── dto.ts                    # TypeScript interfaces and enums
├── subs-to-url.func.ts      # URL building utility function
├── users-api.service.ts     # Service for Users tag
├── products-api.service.ts  # Service for Products tag
└── index.ts                 # Barrel exports
```

## Service Generation Logic

### 1. Service Grouping
Services are grouped by the **first tag** in the OpenAPI specification:
- Each endpoint's `tags[0]` determines which service it belongs to
- Endpoints without tags are **skipped**
- All endpoints with the same tag go into the same service class

### 2. Service Naming Convention
- **Service Name**: `{Tag}ApiService` (e.g., `UsersApiService`)
- **File Name**: `{tag}-api.service.ts` (e.g., `users-api.service.ts`)
- **Class Name**: `{Tag}ApiService` with proper PascalCase

### 3. Method Naming Convention
Method names are derived from the OpenAPI `summary` field:
1. Take the `summary` value
2. Remove all spaces: `"Get User By ID"` → `"GetUserByID"`
3. Convert to camelCase: `"GetUserByID"` → `"getUserByID"`

## OpenAPI Requirements

### Essential Fields for Proper Generation

#### 1. Tags (Required)
```json
{
  "paths": {
    "/users": {
      "get": {
        "tags": ["Users"],  // First tag determines service grouping
        "summary": "Get All Users"
      }
    }
  }
}
```

#### 2. Summary Fields (Required)
```json
{
  "summary": "Get User Profile By ID"  // Becomes method name: getUserProfileByID()
}
```

#### 3. Response Schemas (Required)
```json
{
  "responses": {
    "200": {
      "content": {
        "application/json": {
          "schema": {
            "$ref": "#/components/schemas/UserDTO"  // Must reference a DTO
          }
        }
      }
    }
  }
}
```

#### 4. Schema Naming Convention
- **DTOs**: Must end with `DTO` (e.g., `UserDTO`, `CreateUserRequestDTO`)
- **Enums**: Must end with `Enum` (e.g., `UserStatusEnum`)
- Other schemas are skipped

## Best Practices for OpenAPI Specifications

### 1. Meaningful Summary Fields
❌ **Bad**: Generic summaries
```json
"summary": "Get users"
"summary": "Create user" 
"summary": "List products"
```

✅ **Good**: Descriptive summaries
```json
"summary": "Get All Users With Pagination"
"summary": "Create New User Account"
"summary": "Search Products With Filters"
```

### 2. Consistent Tagging
❌ **Bad**: Inconsistent or missing tags
```json
// No tags - endpoint will be skipped
"get": { "summary": "Get Users" }

// Inconsistent casing
"tags": ["users", "Users", "user"]
```

✅ **Good**: Consistent tag naming
```json
"tags": ["Users"]     // All user-related endpoints
"tags": ["Products"]  // All product-related endpoints
"tags": ["Orders"]    // All order-related endpoints
```

### 3. Proper DTO Structure
```json
{
  "components": {
    "schemas": {
      "UserDTO": {
        "type": "object",
        "required": ["id", "email"],
        "properties": {
          "id": { "type": "string" },
          "email": { "type": "string", "format": "email" }
        }
      }
    }
  }
}
```

## Generated Service Example

Given this OpenAPI specification:
```json
{
  "paths": {
    "/users": {
      "get": {
        "tags": ["Users"],
        "summary": "Get All Users With Pagination",
        "responses": {
          "200": {
            "content": {
              "application/json": {
                "schema": { "$ref": "#/components/schemas/UserListDTO" }
              }
            }
          }
        }
      }
    }
  }
}
```

Generates this Angular service:
```typescript
// users-api.service.ts
import { HttpClient } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable } from 'rxjs';
import { subsToUrl } from './subs-to-url.func';
import { UserListDTO } from './dto';

@Injectable({ providedIn: 'root' })
export class UsersApiService {
  constructor(private http: HttpClient) {}

  getAllUsersWithPagination(): Observable<UserListDTO> {
    const url = subsToUrl('/users', {}, {});
    return this.http.get<UserListDTO>(url);
  }
}
```

## Alternative Naming Convention

### Simplified Structure
```
angular-apis/
├── user-api.ts         # UserApi class
├── product-api.ts      # ProductApi class  
├── order-api.ts        # OrderApi class
└── index.ts            # Barrel exports
```

### Simplified Naming Convention
- **File Names**: `{tag}-api.ts` (e.g., `user-api.ts`, `product-api.ts`)
- **Class Names**: `{Tag}Api` (e.g., `UserApi`, `ProductApi`)
- **Method Names**: Same camelCase convention from summary

### Example Generated Service (Alternative)
```typescript
// user-api.ts
import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

@Injectable({ providedIn: 'root' })
export class UserApi {
  constructor(private http: HttpClient) {}

  getAllUsersWithPagination(): Observable<UserListDTO> {
    return this.http.get<UserListDTO>('/api/users');
  }

  createNewUserAccount(dto: CreateUserRequestDTO): Observable<UserDTO> {
    return this.http.post<UserDTO>('/api/users', dto);
  }
}
```

## Current Limitations

1. **Tag Dependency**: Endpoints without tags are completely skipped
2. **First Tag Only**: Only the first tag is used for grouping
3. **DTO Naming**: Only schemas ending with "DTO" or "Enum" are processed
4. **Response Assumption**: Assumes 200 responses always exist
5. **Fixed File Structure**: No flexibility in generated file organization

## Recommendations

1. **Always include tags** in your OpenAPI specification
2. **Use descriptive summaries** that translate to good method names
3. **Follow DTO naming conventions** with proper suffixes
4. **Group related endpoints** under the same tag
5. **Test generated code** with sample OpenAPI specs before production use

## Error Handling

The generation process will:
- **Skip endpoints** without tags (with console warning)
- **Continue processing** if individual endpoints fail
- **Report errors** for missing response schemas
- **Validate DTO naming** and skip non-conforming schemas 