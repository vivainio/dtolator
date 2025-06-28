# JSON Complex JSON Schema

This directory contains the expected output for generating JSON Schema from a complex JSON file with nested structures.

## Input
- **Source**: `test-data-complex.json` - A complex JSON object with deeply nested structures, arrays of objects, and mixed data types
- **Command**: `dtolator --json test-data-complex.json --json-schema --pretty`

## Generated Files
- `schema.json` - JSON Schema (Draft 2020-12) representation of the complex JSON structure

## Features Demonstrated
- ✅ Complex nested object hierarchies
- ✅ Arrays containing objects with structured schemas
- ✅ Mixed nullable and non-nullable fields
- ✅ Multiple levels of object composition
- ✅ Proper dependency resolution and reference ordering
- ✅ Comprehensive `$defs` section with many reusable schemas
- ✅ Realistic data modeling scenarios (users, organizations, settings)

## JSON Schema Output
The generated schema demonstrates advanced JSON Schema features:
- Deep nesting with proper reference resolution
- Multiple object definitions in `$defs` section
- Complex array handling with object item types
- Mixed data types including nullable fields
- Comprehensive validation structure for enterprise-level data

This test case validates the generator's ability to handle real-world complex JSON structures that might be found in API responses or configuration files. 