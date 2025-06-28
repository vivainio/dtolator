# OpenAPI JSON Schema Output Sample

## Overview
This directory contains a JSON Schema generated from an OpenAPI specification.

- **Command**: `dtolator --openapi simple-sample.json --json-schema`

## Generated Files
- `schema.json` - Complete JSON Schema (Draft 2020-12) generated from the OpenAPI specification

## Features Demonstrated
- OpenAPI to JSON Schema conversion
- Proper `$defs` section with reusable schemas
- JSON Schema Draft 2020-12 compliance
- Schema metadata extraction from OpenAPI info
- Reference resolution and validation constraints

## Input
- **Source**: `simple-sample.json` - An OpenAPI 3.0.3 specification with user management endpoints

## JSON Schema Output
The generated schema converts OpenAPI component schemas to JSON Schema format:
- All `components.schemas` definitions converted to `$defs` 
- OpenAPI-specific extensions mapped to JSON Schema equivalents
- Proper reference linking maintained
- API data models as reusable schema definitions
- Full validation capabilities for API request/response data

This test case demonstrates the generator's ability to bridge OpenAPI specifications with JSON Schema tooling ecosystems, enabling validation, documentation generation, and code generation workflows. 