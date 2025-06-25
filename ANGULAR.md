# Angular API Generation Documentation

## Overview

This document outlines the practices and conventions for automatically generating TypeScript interfaces, DTOs, and Angular services from OpenAPI specifications.

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