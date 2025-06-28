# JSON Simple JSON Schema Output Sample

## Overview
This directory contains a JSON Schema generated from plain JSON data.

- **Command**: `dtolator --json test-data-simple.json --json-schema`

## Generated Files
- `schema.json` - JSON Schema (Draft 2020-12) representation of the JSON data structure

## Features Demonstrated
- Plain JSON to JSON Schema conversion
- Automatic type inference from JSON values
- Schema generation with proper constraints
- Root object schema definition

## Input
- **Source**: `test-data-simple.json` - A simple JSON object with basic data types

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