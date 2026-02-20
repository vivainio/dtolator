# dtolator

Code generator that converts OpenAPI schemas, plain JSON, and JSON Schema files into typed code for multiple languages and frameworks.

## Supported outputs

| Flag | Output |
|------|--------|
| *(default)* | Zod schemas (stdout) |
| `--typescript` / `-t` | TypeScript interfaces |
| `--zod` / `-z` | Zod schemas + TypeScript DTOs |
| `--angular` / `-a` | Angular API services |
| `--pydantic` | Python Pydantic models |
| `--python-dict` | Python TypedDict definitions |
| `--dotnet` | C# classes (System.Text.Json) |
| `--json-schema` | JSON Schema |
| `--endpoints` / `-e` | API endpoint types |
| `--rust-serde` | Rust structs with Serde |

## Installation

Download the latest executable from the [GitHub Releases page](https://github.com/vivainio/dtolator/releases) and place it on your `PATH`.

Or install via Cargo:

```bash
cargo install --git https://github.com/vivainio/dtolator
```

Or build from source:

```bash
git clone https://github.com/vivainio/dtolator
cd dtolator
cargo build --release
```

## Usage

```bash
# Zod schemas to stdout (default)
dtolator --from-openapi schema.json

# TypeScript interfaces to stdout
dtolator --from-openapi schema.json --typescript

# Write to directory (generates dto.ts + schema.ts)
dtolator --from-openapi schema.json --zod -o ./output

# Angular services with Zod validation and promises
dtolator --from-openapi schema.json -o ./src/app/api --angular --zod --promises

# From plain JSON (like quicktype)
dtolator --from-json data.json --typescript --root MyType

# From JSON Schema
dtolator --from-json-schema schema.json --pydantic
```

### Input types

Exactly one required:

- `--from-openapi <FILE>` — OpenAPI 3.x specification (richest: includes endpoints, params, validation)
- `--from-json <FILE>` — Plain JSON data (schema inferred automatically)
- `--from-json-schema <FILE>` — JSON Schema definition

### Output options

- `-o, --output <DIR>` — Write files to directory instead of stdout
- `--delete-old` — After generation, delete files in the output directory that weren't just generated
- `--skip-file <NAME>` — Skip writing a specific file (repeatable)
- `--hide-version` — Omit version from generated file headers
- `--debug` — Verbose debug output

### Angular-specific options

- `--promises` — Use `lastValueFrom` / `Promise` instead of `Observable`
- `--base-url-mode <MODE>` — `global` (default) or `argument`
- `--api-url-variable <NAME>` — Global variable name for API URL (default: `API_URL`)

See [ANGULAR.md](ANGULAR.md) for detailed Angular integration docs.

## OpenAPI features supported

- Basic types: string, number, integer, boolean, array, object
- Enums, `$ref` references, nullable/optional properties
- Composition: `allOf`, `oneOf`, `anyOf`
- Validation: minLength, maxLength, minimum, maximum, pattern, format
- Paths: path params, query params, header params, request bodies, response types
- Map/dictionary types via `additionalProperties`

## Incremental writes

Files are only written when content actually changes (with a fast-path file-size check), so bundler watchers won't trigger unnecessary rebuilds.

## Testing

```bash
cargo test                           # Run all tests
python run-tests.py                  # Integration test suite
python run-tests.py --refresh        # Update expected outputs after intentional changes
```

See [ADVANCED.md](ADVANCED.md) for endpoint generation details.

## License

MIT
