# JSON Simple JSON Schema

This directory contains the expected output for generating JSON Schema from a simple JSON file.

## Input
- **Source**: `test-data-simple.json` - A simple JSON object with basic data types
- **Command**: `dtolator --json test-data-simple.json --json-schema --pretty`

## Generated Files
- `schema.json` - JSON Schema (Draft 2020-12) representation of the input JSON structure

## Features Demonstrated
- ✅ Basic type conversion (string, integer, boolean, null)
- ✅ Nested object handling with proper `$defs` and `$ref` usage
- ✅ Array type with item type definitions  
- ✅ Required field detection
- ✅ Proper JSON Schema metadata (`$schema`, `title`, `description`)
- ✅ `additionalProperties: false` for strict validation

## JSON Schema Output
The generated schema follows JSON Schema Draft 2020-12 specification and includes:
- Root level object schema with all properties
- Reusable definitions in `$defs` section
- Proper reference linking with `$ref`
- Type constraints and validation rules
- Descriptive metadata for tooling integration 