# Rust Test Suite for dtolator

The dtolator project now includes a comprehensive Rust-based test suite that replaces the previous Python `test-suite.py`. All tests run within the Rust ecosystem and maintain the same output validation against `output-samples/`.

## Quick Start

### Running All Tests
The simplest way to run the test suite:

```bash
cargo test --test integration_tests
```

This will automatically:
1. Build the latest dtolator binary (release mode)
2. Run all 22 test cases
3. Compare outputs with expected files in `output-samples/`
4. Report results with detailed diffs if any tests fail

Alternatively, use the convenient batch script on Windows:

```cmd
run-tests.bat
```

### Refreshing Expected Output Files
To regenerate all expected outputs in `output-samples/` (useful after intentional code improvements):

```bash
DTOLATOR_TEST_REFRESH=1 cargo test --test integration_tests
```

Or use the batch script on Windows:

```cmd
run-tests.bat --refresh
```

## Test Coverage

The Rust test suite includes **22 comprehensive test cases** covering all supported output formats:

### Angular Services
- Angular Full Sample (with Zod validation)
- Angular Simple Sample (with Zod validation)
- Angular Nested Test (without Zod)
- Comprehensive Nested Test (with Zod)
- Angular Promises with Zod
- Angular Promises without Zod

### TypeScript & Zod
- Nested Test (TypeScript + Zod)
- JSON Simple TypeScript
- JSON Simple Zod
- JSON Complex TypeScript
- From JSON Schema TypeScript
- From JSON Schema Zod

### Python Output
- Pydantic Test
- Python TypedDict Test
- Python TypedDict Full Test
- JSON Simple Pydantic
- JSON Complex Pydantic
- From JSON Schema Pydantic

### .NET & JSON Schema
- DotNet Test (C# classes)
- JSON Simple JSON Schema
- JSON Complex JSON Schema
- OpenAPI JSON Schema

## Test Structure

The test suite is implemented in `tests/integration_tests.rs` with the following components:

### TestCase struct
Defines a single test case with:
- `name`: Display name for the test
- `input_file`: Input file (OpenAPI JSON, plain JSON, or JSON Schema)
- `command_args`: Command-line arguments for dtolator
- `expected_dir`: Expected output directory in `output-samples/`

### TestSuite struct
Main test runner that:
1. Builds the dtolator binary (unless `--no-build` is specified)
2. Runs each test case against the binary
3. Compares generated output with expected files in `output-samples/`
4. Optionally refreshes expected output files

### Key Functions
- `build_project()`: Compiles dtolator using cargo
- `compare_files()`: Compares individual files with diff output
- `compare_directories()`: Recursively compares output directories
- `run_single_test()`: Executes a single test case
- `run_all_tests()`: Runs the entire test suite

## Output Format

### Success
When all tests pass:
```
============================================================
                    Test Results Summary
============================================================
Total tests: 22
Passed: 22
Failed: 0

ALL TESTS PASSED!
```

### Failures
When tests fail, detailed information is provided:
```
============================================================
                    Test Results Summary
============================================================
Total tests: 22
Passed: 20
Failed: 2

FAILURE DETAILS:
   • Angular Full Sample: Output differs - 3 errors
   • JSON Simple TypeScript: File differs: dto.ts
```

## Dependencies

Test suite dependencies are isolated in `[dev-dependencies]` of `Cargo.toml`:
- `walkdir` - Directory traversal for file comparison
- `similar` - Text diffing for file comparisons
- `colored` - Colored console output
- `tempfile` - Temporary directory management

These dependencies are **only compiled when running tests**, not included in the main binary.

## Key Differences from Python Suite

### Benefits of Rust Implementation
1. **Unified Workflow** - Uses standard `cargo test` command, no separate Python script
2. **Always Tests Fresh Code** - Automatically rebuilds and tests the latest dtolator binary
3. **Faster Execution** - Native binary compilation and execution
4. **No External Dependencies** - Test runner is built into the Rust ecosystem
5. **Better Integration** - Seamlessly integrated with Rust toolchain and CI/CD
6. **Consistent Environment** - Same toolchain as the main binary

### File Organization
- **Python version**: `test-suite.py` in project root (deprecated)
- **Rust version**: `tests/integration_tests.rs` in tests directory
- **Helper script**: `run-tests.bat` for Windows convenience
- **Output samples**: Both versions use the same `output-samples/` directory

### Feature Parity
The Rust suite maintains all functionality from the Python version:
- ✅ 22 test cases across all output formats
- ✅ Directory comparison with diff output
- ✅ Refresh mode to update expected outputs
- ✅ Colored console output
- ✅ Detailed error reporting
- ✅ Automatic build integration (always tests fresh code)
- ✅ Environment variable configuration
- ❌ TypeScript type checking (removed - can be added separately if needed)

## Integration with CI/CD

The test suite integrates seamlessly with CI/CD pipelines:

```bash
# Run tests and exit with appropriate code
cargo test --test integration_tests
```

The test will exit with:
- Code `0` if all tests pass
- Code `1` if any test fails

## Troubleshooting

### Binary Not Found Error
If you get "dtolator binary not found", rebuild the project:
```bash
cargo build --release
cargo test --test integration_tests
```

### Permission Errors on Windows
If you encounter permission issues, ensure the target directory isn't locked:
```cmd
cargo clean
cargo test --test integration_tests
```

### Output Comparison Mismatches
If tests fail due to output differences:
1. Review the diff output carefully
2. If changes are intentional, refresh outputs: `run-tests.bat --refresh`
3. Verify the changes before committing

## Future Enhancements

Potential improvements to the test suite:
- [ ] TypeScript type checking integration
- [ ] Performance benchmarking
- [ ] Parallel test execution
- [ ] Test filtering by name pattern
- [ ] Coverage reporting
- [ ] HTML test report generation

