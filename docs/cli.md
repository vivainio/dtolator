# Command-Line Help for `dtolator`

This document contains the help content for the `dtolator` command-line program.

**Command Overview:**

* [`dtolator`↴](#dtolator)
* [`dtolator peek`↴](#dtolator-peek)

## `dtolator`

Convert OpenAPI schema JSON files to Zod schema definitions or TypeScript interfaces

**Usage:** `dtolator [OPTIONS] [COMMAND]`

###### **Subcommands:**

* `peek` — Quick-look at an OpenAPI spec: prints minimal endpoint + type summary to stdout

###### **Options:**

* `--from-openapi <FROM_OPENAPI>` — Input OpenAPI schema JSON file
* `--from-json <FROM_JSON>` — Input plain JSON file (for generating DTOs like quicktype.io)
* `--from-json-schema <FROM_JSON_SCHEMA>` — Input JSON Schema file (for generating DTOs from JSON Schema)
* `--root <ROOT>` — Name for the root class/interface when using --json (default: Root)

  Default value: `Root`
* `-o`, `--output <OUTPUT>` — Output directory path (if specified, writes dto.ts and optionally schema.ts files)
* `--output-file <OUTPUT_FILE>` — Write the combined output to a single file (mutually exclusive with --output; --angular emits multiple files, so use --output with a directory there instead)
* `-t`, `--typescript` — Generate TypeScript interfaces instead of Zod schemas (when not using output directory)
* `-z`, `--zod` — Generate Zod schemas (creates schema.ts and makes dto.ts import from it)
* `-a`, `--angular` — Generate Angular API services (creates multiple service files and utilities)
* `--pydantic` — Generate Pydantic BaseModel classes for Python
* `--pydantic-version <VERSION>` — Pydantic version to target (1 or 2, default: 1). Implies --pydantic when specified

  Possible values: `1`, `2`

* `--python-dict` — Generate Python TypedDict definitions
* `--dotnet` — Generate C# classes with System.Text.Json serialization
* `--json-schema` — Generate JSON Schema output
* `-e`, `--endpoints` — Generate API endpoint types from OpenAPI paths
* `--rust-serde` — Generate Rust structs with Serde serialization/deserialization
* `--markdown` — Generate Markdown API documentation
* `--markdown-minimal` — Generate minimal Markdown (endpoints + types only, no tables or prose)
* `--promises` — Generate promises using lastValueFrom instead of Observables (only works with --angular)
* `--debug` — Enable debug output
* `--hide-version` — Hide version from generated output headers (use 'dtolator' instead of 'dtolator==VERSION')
* `--skip-file <SKIP_FILE>` — Skip writing specific file(s) to output directory (can be used multiple times)
* `--base-url-mode <BASE_URL>` — Base URL generation mode: 'global' (default), 'argument', or 'none' (relative URLs, route only)

  Default value: `global`

  Possible values:
  - `global`:
    Use global variable for API URL (default: API_URL, customizable via --api-url-variable)
  - `argument`:
    Pass baseUrl as mandatory first parameter
  - `none`:
    Use only the route, with no base URL prefix (relative URLs)

* `--api-url-variable <API_URL_VARIABLE>` — Name of the global variable used for the API base URL (only with --base-url-mode global)

  Default value: `API_URL`
* `--ignore-operation-id` — Ignore the operationId when generating Angular method names; derive names from the summary instead
* `--delete-old` — Delete obsolete files from the output directory after generation



## `dtolator peek`

Quick-look at an OpenAPI spec: prints minimal endpoint + type summary to stdout

**Usage:** `dtolator peek <FILE>`

###### **Arguments:**

* `<FILE>` — OpenAPI JSON file to peek at



<hr/>

<small><i>
    This document was generated automatically by
    <a href="https://crates.io/crates/clap-markdown"><code>clap-markdown</code></a>.
</i></small>
