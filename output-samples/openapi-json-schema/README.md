# OpenAPI JSON Schema

This directory contains the expected output for generating JSON Schema from an OpenAPI specification.

## Input
- **Source**: `simple-sample.json` - An OpenAPI 3.0.3 specification with user management endpoints
- **Command**: `dtolator --openapi simple-sample.json --json-schema --pretty`

## Generated Files
- `schema.json` - JSON Schema (Draft 2020-12) representation of all schemas defined in the OpenAPI specification

## Features Demonstrated
- ✅ OpenAPI schema component extraction
- ✅ Reference resolution between schemas (`$ref` handling)
- ✅ Type system conversion from OpenAPI to JSON Schema
- ✅ Comprehensive schema definitions for API data models
- ✅ Proper metadata extraction from OpenAPI info section
- ✅ Complex object relationships and dependencies

## JSON Schema Output
The generated schema converts OpenAPI component schemas to JSON Schema format:
- All `components.schemas` definitions converted to `$defs` 
- OpenAPI-specific extensions mapped to JSON Schema equivalents
- Proper reference linking maintained
- API data models as reusable schema definitions
- Full validation capabilities for API request/response data

This test case demonstrates the generator's ability to bridge OpenAPI specifications with JSON Schema tooling ecosystems, enabling validation, documentation generation, and code generation workflows. 