# dtolator

**dtolator** (Data Type Translator) is a Rust command-line tool that converts OpenAPI schema JSON files to either Zod schema definitions or TypeScript interfaces.

## Features

- ✅ Convert OpenAPI 3.x schemas to Zod schemas with validation
- ✅ Convert OpenAPI 3.x schemas to TypeScript interfaces
- ✅ Support for complex types (objects, arrays, enums, unions)
- ✅ Support for OpenAPI composition keywords (`allOf`, `oneOf`, `anyOf`)
- ✅ Support for validation constraints (min/max, length, patterns, formats)
- ✅ Support for nullable types
- ✅ Support for schema references (`$ref`)
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

Output to stdout:
```bash
dtolator -i schema.json --typescript
```

### Command Line Options

```
Convert OpenAPI schema JSON files to Zod schema definitions or TypeScript interfaces

Usage: dtolator [OPTIONS] --input <INPUT>

Options:
  -i, --input <INPUT>      Input OpenAPI schema JSON file
  -o, --output <OUTPUT>    Output file path
  -t, --typescript         Generate TypeScript interfaces instead of Zod schemas
  -p, --pretty             Pretty print the output
  -h, --help               Print help
  -V, --version            Print version
```

## Examples

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

## Testing

Test the application with the provided example:

```bash
# Build the project
cargo build --release

# Test with Zod output
./target/release/dtolator -i example.json

# Test with TypeScript output  
./target/release/dtolator -i example.json --typescript
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details. 