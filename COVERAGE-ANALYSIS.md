# Code Coverage Analysis for dtolator

## Overview

This document outlines what **IS** and **IS NOT** covered by the current test suite, helping identify where additional tests would be most valuable.

## Currently Well-Tested Areas ✅

### Input Processing
- ✅ OpenAPI 3.x schema parsing
- ✅ Plain JSON to OpenAPI schema conversion
- ✅ JSON Schema to OpenAPI schema conversion
- ✅ JavaScript-style comment stripping from JSON Schema
- ✅ Schema reference resolution (`$ref` handling)
- ✅ Properties, required fields, and schema composition

### Generator Types
- ✅ TypeScript interface generation (basic)
- ✅ Zod schema generation
- ✅ Angular service generation
- ✅ Pydantic model generation
- ✅ Python TypedDict generation
- ✅ C# .NET model generation
- ✅ JSON Schema generation
- ✅ Endpoint type generation

### Angular Features
- ✅ Observable-based API method generation
- ✅ Promise-based API method generation (with `--promises`)
- ✅ Query parameter type generation
- ✅ Header parameter type generation
- ✅ Multiple service file generation
- ✅ Utility function generation (fill-url)
- ✅ TypeScript/Zod integration

### Output Modes
- ✅ Directory-based output (multiple files)
- ✅ Stdout output (single file)
- ✅ Nested directory structures
- ✅ File naming and organization

### Complex Schema Features
- ✅ Nested objects and arrays
- ✅ Schema references and deduplication
- ✅ allOf/oneOf/anyOf support
- ✅ Enum values
- ✅ Array item type inference
- ✅ Naming conventions for complex types

## Likely Coverage Gaps ❌

### Error Handling & Validation
- ❌ **Invalid input files**
  - Non-JSON files
  - Malformed JSON
  - Invalid OpenAPI structure
  - Missing required OpenAPI fields
  - Invalid schema types

- ❌ **Invalid references**
  - Broken `$ref` links
  - Circular references in some contexts
  - Self-referencing schemas

- ❌ **Edge case handling**
  - What happens with undefined behavior
  - Fallback for unsupported combinations

### JSON Schema Edge Cases
- ❌ Complex `additionalProperties` schemas
- ❌ Multiple type arrays beyond ["type", "null"]
- ❌ Schema composition with missing refs
- ❌ Very deeply nested `$defs`

### Generator Features
- ❌ **Less common options**
  - Some flag combinations not tested
  - Debug mode output
  - Hide version functionality

- ❌ **Type mapping edge cases**
  - Unusual format specifications
  - Non-standard type combinations
  - Complex pattern validations

- ❌ **Output edge cases**
  - Very long identifiers
  - Special characters in names
  - Unicode and internationalization
  - Path edge cases

### TypeScript/Zod Specific
- ❌ Complex discriminated unions
- ❌ Recursive type definitions in edge cases
- ❌ Very deep nesting (performance/limits)
- ❌ Rare validation patterns

### Angular Specific
- ❌ Complex request/response transformations
- ❌ Unusual HTTP status codes
- ❌ File upload/download handling
- ❌ Special content types

### Python Generators
- ❌ Advanced Pydantic validators
- ❌ Complex TypedDict edge cases
- ❌ Import ordering and formatting

### C# Generator
- ❌ Complex nullable reference types
- ❌ Advanced JSON serialization options
- ❌ Unusual property naming patterns

## Test Coverage by Percentage

### Estimated Coverage Areas

| Area | Coverage | Priority |
|------|----------|----------|
| Input parsing | 80-90% | ✅ Good |
| Angular generation | 85-95% | ✅ Good |
| TypeScript generation | 75-85% | ⚠️ Needs work |
| Zod generation | 75-85% | ⚠️ Needs work |
| Pydantic generation | 70-80% | ⚠️ Needs work |
| Python Dict generation | 70-80% | ⚠️ Needs work |
| .NET generation | 60-70% | ⚠️ Needs work |
| JSON Schema generation | 60-70% | ⚠️ Needs work |
| Error handling | 20-30% | 🔴 Critical gap |
| Edge cases | 30-40% | 🔴 Critical gap |

## Suggested Test Additions

### Priority 1: Error Handling (Most Important)
These tests would catch regressions and improve robustness:

```rust
// Invalid input tests
TestCase { input_file: "invalid.json", ... }
TestCase { input_file: "broken-openapi.json", ... }

// Circular reference handling
TestCase { input_file: "circular-refs.json", ... }

// Empty/minimal schemas
TestCase { input_file: "empty-schema.json", ... }
TestCase { input_file: "minimal-schema.json", ... }
```

### Priority 2: Edge Cases
These tests improve code quality:

```rust
// Deep nesting limits
TestCase { input_file: "deeply-nested.json", ... }

// Special characters
TestCase { input_file: "special-chars.json", ... }

// Unusual combinations
TestCase { 
    command_args: vec!["--typescript", "--json-schema"], 
    ... 
}
```

### Priority 3: Feature Completeness
These tests ensure all features work:

```rust
// Less common generators
TestCase { 
    command_args: vec!["--endpoints", "--from-openapi"],
    ...
}

// Rare option combinations
TestCase {
    command_args: vec!["--debug", "--hide-version"],
    ...
}
```

## How to Measure Improvement

1. **Baseline**: Run `run-tests.bat --coverage --html` now
2. **Add tests**: Create new test cases in `tests/integration_tests.rs`
3. **Refresh**: Run `run-tests.bat --refresh` to update outputs
4. **Measure**: Run `run-tests.bat --coverage --html` again
5. **Compare**: Check if coverage percentage increased

## Coverage Report Navigation

When you run `run-tests.bat --coverage --html`:

1. **index.html**: Shows overall coverage
2. **Source files**: Click to see line-by-line coverage
3. **Red lines**: Not executed during tests
4. **Green lines**: Executed during tests

Focus on red lines in:
- `src/lib.rs` - Main library logic
- `src/generators/*.rs` - Generation code
- `src/openapi.rs` - Schema processing

## Next Steps

1. **Generate baseline**: `run-tests.bat --coverage --html`
2. **Review report**: Open `coverage/index.html`
3. **Identify gaps**: Look for red lines in critical code
4. **Add tests**: Create focused test cases
5. **Measure impact**: Check coverage improvement

See [COVERAGE.md](COVERAGE.md) for detailed instructions.

