# Angular API with Zod Validation - Generated Sample

This directory contains a complete Angular API client generated from the `full-sample.json` OpenAPI specification using the dtolator tool with Zod validation enabled.

## Command Used

```bash
cargo run -- --input full-sample.json --output output-samples/angular --angular --zod
```

## Generated Files

### 📄 `schema.ts` (6,763 bytes)
Contains Zod validation schemas with comprehensive validation rules:
- **Runtime validation** for all data types
- **Type inference** using `z.infer<typeof Schema>`
- **Complex validation rules** including regex patterns, min/max constraints, and format validation
- **Schema constants** with "Schema" suffix (e.g., `UserSchema`, `CreateUserRequestSchema`)

### 📄 `dto.ts` (1,546 bytes)
TypeScript type definitions that import from `schema.ts`:
- **Type-only imports** for clean separation
- **Re-exports** of all types for convenient usage
- **Automatic type inference** from Zod schemas

### 📄 `default-api.service.ts` (2,240 bytes)
Angular service with **built-in Zod validation**:
- **Request validation**: Validates request bodies before sending HTTP requests
- **Response validation**: Validates API responses using RxJS `map` operator
- **Type-safe methods** with proper TypeScript typing
- **Injectable service** ready for dependency injection

### 📄 `subs-to-url.func.ts` (1,084 bytes)
URL building utility function:
- **Path parameter substitution**
- **Query parameter serialization**
- **Environment configuration support**
- **Runtime API configuration injection**

### 📄 `index.ts` (185 bytes)
Barrel exports for convenient imports:
- Exports all types, schemas, services, and utilities
- Single import point for the entire API client

## Key Features with Zod Validation

### 1. Request Body Validation
```typescript
createNewUserAccount(dto: CreateUserRequest): Observable<User> {
  // Validate request body with Zod
  const validatedDto = CreateUserRequestSchema.parse(dto);

  const url = subsToUrl('/users', {}, {});
  return this.http.post<User>(url, validatedDto)
    .pipe(
      map(response => UserSchema.parse(response))
    );
}
```

### 2. Response Validation
```typescript
getAllUsersWithPagination(queryParams?: { page?: number, limit?: number }): Observable<UserListResponse> {
  const url = subsToUrl('/users', {}, queryParams || {});
  return this.http.get<UserListResponse>(url)
    .pipe(
      map(response => UserListResponseSchema.parse(response))
    );
}
```

### 3. Comprehensive Schema Validation
```typescript
export const UserSchema = z.object({
  id: z.string().uuid(),
  email: z.string().email(),
  profile: UserProfileSchema,
  preferences: UserPreferencesSchema.optional(),
  createdAt: z.string().datetime().optional(),
  updatedAt: z.string().datetime().optional(),
  isActive: z.boolean().optional(),
  roles: z.array(UserRoleSchema).optional()
});
```

## Usage in Angular Components

### Basic Service Injection
```typescript
import { Component, OnInit } from '@angular/core';
import { DefaultApiService, User, CreateUserRequest } from './api';

@Component({
  selector: 'app-user-management',
  template: `
    <div *ngFor="let user of users">
      <h3>{{ user.profile.firstName }} {{ user.profile.lastName }}</h3>
      <p>{{ user.email }}</p>
    </div>
  `
})
export class UserManagementComponent implements OnInit {
  users: User[] = [];

  constructor(private apiService: DefaultApiService) {}

  ngOnInit() {
    this.loadUsers();
  }

  loadUsers() {
    this.apiService.getAllUsersWithPagination({ page: 1, limit: 10 })
      .subscribe({
        next: (response) => {
          this.users = response.data; // Already validated by Zod
        },
        error: (error) => {
          // Zod validation errors or HTTP errors
          console.error('Failed to load users:', error);
        }
      });
  }
}
```

## Dependencies Required

Add these to your Angular project:

```json
{
  "dependencies": {
    "@angular/common": "^17.0.0",
    "@angular/core": "^17.0.0",
    "rxjs": "^7.0.0",
    "zod": "^3.22.0"
  }
}
```

## Benefits of Zod Validation

1. **Runtime Type Safety**: Validates data at runtime, not just compile time
2. **API Contract Enforcement**: Ensures API responses match expected schemas
3. **Better Error Messages**: Detailed validation errors for debugging
4. **Data Transformation**: Automatic type coercion and formatting
5. **Schema Evolution**: Easy to update validation rules as APIs evolve

## Generated API Methods

| Method | Endpoint | Validation |
|--------|----------|------------|
| `getAllUsersWithPagination()` | `GET /users` | Response validated with `UserListResponseSchema` |
| `createNewUserAccount()` | `POST /users` | Request: `CreateUserRequestSchema`, Response: `UserSchema` |
| `getUserProfileByID()` | `GET /users/{userId}` | Response validated with `UserSchema` |
| `searchProductsWithFilters()` | `GET /products` | Response validated with `ProductListResponseSchema` |
| `createNewOrder()` | `POST /orders` | Request: `CreateOrderRequestSchema`, Response: `OrderSchema` |

This generated API client provides a complete, type-safe, and validated interface to your OpenAPI backend with full runtime validation using Zod schemas. 