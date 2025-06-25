# dtolator

**dtolator** (Data Type Translator) is a Rust command-line tool that converts OpenAPI schema JSON files to either Zod schema definitions, TypeScript interfaces, or API endpoint types.

## Features

- ✅ Convert OpenAPI 3.x schemas to Zod schemas with validation
- ✅ Convert OpenAPI 3.x schemas to TypeScript interfaces
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

Generate Zod schemas (default):
```bash
dtolator -i schema.json -o output.ts
```

Generate TypeScript interfaces:
```bash
dtolator -i schema.json -o output.ts --typescript
```

Generate API endpoint types:
```bash
dtolator -i schema.json -o endpoints.ts --endpoints
```

Output to stdout:
```bash
dtolator -i schema.json --typescript
```

### Command Line Options

```
Convert OpenAPI schema JSON files to Zod schema definitions, TypeScript interfaces, or API endpoint types

Usage: dtolator [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>      Input OpenAPI schema JSON file
  -o, --output <OUTPUT>    Output file path
  -t, --typescript         Generate TypeScript interfaces instead of Zod schemas
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

### 2. React Hooks Integration

```typescript
import { useState, useEffect } from 'react';

function useApiCall<K extends keyof ApiEndpoints>(
  endpoint: K,
  options: Parameters<TypedApiClient['call']>[1],
  enabled = true
) {
  const [data, setData] = useState<ExtractEndpointResponse<K> | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const execute = async () => {
    setLoading(true);
    setError(null);
    try {
      const api = new TypedApiClient('https://api.example.com/v2');
      const result = await api.call(endpoint, options);
      setData(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (enabled) execute();
  }, [enabled]);

  return { data, loading, error, refetch: execute };
}

// React component using the hook
function UserProfile({ userId }: { userId: string }) {
  // Fully typed hook - knows this returns User data
  const { data: user, loading, error } = useApiCall(
    'GET /users/{userId}',
    { params: { userId } }
  );

  if (loading) return <div>Loading...</div>;
  if (error) return <div>Error: {error}</div>;
  if (!user) return <div>No user found</div>;

  // TypeScript knows user.profile, user.email, etc. exist
  return (
    <div>
      <h1>{user.profile.firstName} {user.profile.lastName}</h1>
      <p>Email: {user.email}</p>
    </div>
  );
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