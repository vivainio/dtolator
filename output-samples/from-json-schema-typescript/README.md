# From JSON Schema TypeScript

This directory contains the expected output for generating TypeScript interfaces from a JSON Schema file.

## Input
- **Source**: `output-samples/json-simple-json-schema/schema.json` - A JSON Schema (Draft 2020-12) with user profile and address schemas
- **Command**: `dtolator --from-json-schema output-samples/json-simple-json-schema/schema.json --typescript --pretty`

## Generated Files
- `dto.ts` - TypeScript interfaces representing all schemas defined in the JSON Schema

## Features Demonstrated
- ✅ JSON Schema parsing with comment stripping
- ✅ `$defs` resolution and conversion to TypeScript interfaces
- ✅ Reference handling (`$ref` → interface relationships)
- ✅ Type mapping from JSON Schema to TypeScript:
  - `"type": "string"` → `string`
  - `"type": "integer"` → `number`
  - `"type": "boolean"` → `boolean`
  - `"type": ["string", "null"]` → `string | null`
  - `"type": "array"` → `T[]`
  - `"type": "object"` → interface definition
- ✅ Optional property handling (properties not in `required` array)
- ✅ Nested object relationships
- ✅ Validation constraint awareness (preserved in schema but not enforced in TS)

## JSON Schema Input Features
The input JSON Schema demonstrates:
- **Schema definitions** in `$defs` section
- **Root schema** with properties and references
- **References** between schemas using `$ref: "#/$defs/TypeName"`
- **Mixed types** like `["null", "null"]` (nullable fields)
- **Required fields** specification
- **Type constraints** like `minLength`, `pattern` (metadata preserved)

## TypeScript Output
The generated TypeScript interfaces provide:
- **Clean interface definitions** for all schema types
- **Optional properties** using `?:` syntax
- **Union types** for nullable fields (`type | null`)
- **Nested references** resolved to proper interface types
- **Export statements** for all interfaces

## Use Cases
This feature enables:
1. **JSON Schema → TypeScript workflow** for existing JSON Schema files
2. **Cross-ecosystem compatibility** between JSON Schema and TypeScript tools
3. **Schema validation integration** with JSON Schema validators
4. **Documentation generation** from JSON Schema specifications
5. **Code generation** from standardized JSON Schema definitions

## Integration Notes
- Input JSON Schema files can contain JavaScript-style comments (`/* ... */`) which are automatically stripped
- Supports JSON Schema Draft 2020-12 format
- All existing dtolator generators work with `--from-json-schema` input
- Maintains full compatibility with validation constraints and metadata 