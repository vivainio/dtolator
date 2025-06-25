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

### Real-world Workflow

For a complete type-safe API setup, generate all three outputs:

```bash
# 1. Generate TypeScript interfaces for data types
dtolator -i your-api.json --typescript -o src/types/api-types.ts

# 2. Generate API endpoint definitions
dtolator -i your-api.json --endpoints -o src/types/api-endpoints.ts

# 3. Generate Zod schemas for runtime validation
dtolator -i your-api.json -o src/schemas/api-schemas.ts
```

Then use them in your application:

```typescript
// Import generated types
import { User, Product } from './types/api-types';
import { ApiEndpoints } from './types/api-endpoints';
import { User as UserSchema } from './schemas/api-schemas';

// Type-safe API client
async function getUser(id: string): Promise<User> {
  const response = await fetch(`/api/users/${id}`);
  const data = await response.json();
  
  // Runtime validation with Zod
  return UserSchema.parse(data);
}

// Type-safe API call using endpoint types
class ApiClient {
  async call<K extends keyof ApiEndpoints>(
    endpoint: K,
    options: /* endpoint-specific options */
  ): Promise</* endpoint-specific response */> {
    // Implementation with full type safety
  }
}
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details. 