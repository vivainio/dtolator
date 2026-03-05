use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub mod generators;
pub mod openapi;

pub use generators::pydantic::PydanticVersion;
use generators::{
    Generator, angular::AngularGenerator, dotnet::DotNetGenerator, endpoints::EndpointsGenerator,
    json_schema::JsonSchemaGenerator, pydantic::PydanticGenerator,
    python_dict::PythonDictGenerator, rust_serde::RustSerdeGenerator,
    typescript::TypeScriptGenerator, zod::ZodGenerator,
};
use indexmap::IndexMap;
use openapi::{AdditionalProperties, Components, Info, OpenApiSchema, Schema};

// Type aliases to reduce complexity
type StructHashMap =
    std::collections::HashMap<String, (String, Vec<String>, Option<String>, Vec<String>)>;
type JsonToPlaceholderMap = std::collections::HashMap<String, String>;

/// High-level input type for generation
#[derive(Debug, Clone, Copy)]
pub enum InputType {
    OpenApi,
    Json,
    JsonSchema,
}

/// High-level generator type for generation
#[derive(Debug, Clone, Copy)]
pub enum GeneratorType {
    TypeScript,
    Angular,
    Zod,
    Pydantic,
    PythonDict,
    DotNet,
    JsonSchema,
    Endpoints,
    RustSerde,
}

/// Base URL mode for Angular API generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum BaseUrlMode {
    /// Use global variable for API URL (default: API_URL, customizable via --api-url-variable)
    Global,
    /// Pass baseUrl as mandatory first parameter
    Argument,
}

impl BaseUrlMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            BaseUrlMode::Global => "global",
            BaseUrlMode::Argument => "argument",
        }
    }
}

/// Options for library-based generation
#[derive(Debug, Clone)]
pub struct GenerateOptions {
    pub input_type: InputType,
    pub input_path: PathBuf,
    pub output_dir: PathBuf,
    pub generator_type: GeneratorType,
    pub pydantic_version: PydanticVersion,
    pub with_zod: bool,
    pub with_promises: bool,
    pub hide_version: bool,
    pub root_name: String,
    pub debug: bool,
    pub skip_files: Vec<String>,
    pub base_url_mode: BaseUrlMode,
    pub api_url_variable: String,
    pub delete_old: bool,
}

impl GenerateOptions {
    pub fn build_command_string(&self) -> String {
        let version = env!("BUILD_VERSION");
        let command_name = if self.hide_version {
            "dtolator".to_string()
        } else {
            format!("dtolator=={version}")
        };

        let mut parts = vec![command_name];

        if let Some(filename) = self.input_path.file_name().and_then(|name| name.to_str()) {
            match self.input_type {
                InputType::OpenApi => parts.push(format!("--from-openapi {filename}")),
                InputType::Json => parts.push(format!("--from-json {filename}")),
                InputType::JsonSchema => parts.push(format!("--from-json-schema {filename}")),
            }
        }

        // For Angular, put --zod before --angular; for others, main generator first
        if self.with_zod && matches!(self.generator_type, GeneratorType::Angular) {
            parts.push("--zod".to_string());
        }

        // Main generator types
        match self.generator_type {
            GeneratorType::TypeScript => {
                parts.push("--typescript".to_string());
            }
            GeneratorType::Angular => {
                parts.push("--angular".to_string());
            }
            GeneratorType::Pydantic => {
                parts.push("--pydantic".to_string());
                if self.pydantic_version == PydanticVersion::V2 {
                    parts.push("--pydantic-version 2".to_string());
                }
            }
            GeneratorType::PythonDict => {
                parts.push("--python-dict".to_string());
            }
            GeneratorType::DotNet => {
                parts.push("--dotnet".to_string());
            }
            GeneratorType::JsonSchema => {
                parts.push("--json-schema".to_string());
            }
            GeneratorType::Endpoints => {
                parts.push("--endpoints".to_string());
            }
            GeneratorType::Zod => {
                parts.push("--zod".to_string());
            }
            GeneratorType::RustSerde => {
                parts.push("--rust-serde".to_string());
            }
        }

        // For non-Angular, put --zod after main generator
        if self.with_zod
            && !matches!(
                self.generator_type,
                GeneratorType::Angular | GeneratorType::Zod
            )
        {
            parts.push("--zod".to_string());
        }

        if self.with_promises {
            parts.push("--promises".to_string());
        }

        if self.debug {
            parts.push("--debug".to_string());
        }

        if self.base_url_mode != BaseUrlMode::Global {
            parts.push(format!("--base-url-mode {}", self.base_url_mode.as_str()));
        }

        if self.api_url_variable != "API_URL" {
            parts.push(format!("--api-url-variable {}", self.api_url_variable));
        }

        parts.join(" ")
    }
}

/// Library entry point for generation. This mirrors the directory-output behavior of the CLI.
pub fn generate(options: GenerateOptions) -> Result<()> {
    // Read and parse the input file into an OpenApiSchema
    let schema = match options.input_type {
        InputType::OpenApi => {
            let input_content =
                std::fs::read_to_string(&options.input_path).with_context(|| {
                    format!(
                        "Failed to read OpenAPI file: {}",
                        options.input_path.display()
                    )
                })?;

            serde_json::from_str::<OpenApiSchema>(&input_content)
                .with_context(|| "Failed to parse OpenAPI schema JSON")?
        }
        InputType::Json => {
            let input_content =
                std::fs::read_to_string(&options.input_path).with_context(|| {
                    format!("Failed to read JSON file: {}", options.input_path.display())
                })?;

            json_to_openapi_schema_with_root(
                serde_json::from_str(&input_content)?,
                &options.root_name,
            )?
        }
        InputType::JsonSchema => {
            let input_content =
                std::fs::read_to_string(&options.input_path).with_context(|| {
                    format!(
                        "Failed to read JSON Schema file: {}",
                        options.input_path.display()
                    )
                })?;

            let cleaned_content = strip_json_comments(&input_content);
            json_schema_to_openapi_schema(
                serde_json::from_str(&cleaned_content)?,
                &options.root_name,
            )?
        }
    };

    // Extract inline request body schemas and add them to components
    let schema = extract_inline_request_schemas(schema)?;

    // Build the command string used in generated file headers
    let command_string = options.build_command_string();

    // Ensure output directory exists
    fs::create_dir_all(&options.output_dir).with_context(|| {
        format!(
            "Failed to create output directory: {}",
            options.output_dir.display()
        )
    })?;

    let mut written_files: Vec<String> = Vec::new();

    match options.generator_type {
        GeneratorType::Angular => {
            written_files = generate_angular_services(
                &schema,
                &options.output_dir,
                options.with_zod,
                options.debug,
                options.with_promises,
                &command_string,
                &options.skip_files,
                options.base_url_mode,
                &options.api_url_variable,
            )?;
        }
        GeneratorType::Pydantic => {
            let pydantic_generator = PydanticGenerator::new(options.pydantic_version);
            let pydantic_output =
                pydantic_generator.generate_with_command(&schema, &command_string)?;
            let models_path = options.output_dir.join("models.py");
            write_if_changed(&models_path, &pydantic_output)?;
            written_files.push("models.py".to_string());
        }
        GeneratorType::PythonDict => {
            let python_dict_generator = PythonDictGenerator::new();
            let python_dict_output =
                python_dict_generator.generate_with_command(&schema, &command_string)?;

            let typed_dicts_path = options.output_dir.join("typed_dicts.py");
            write_if_changed(&typed_dicts_path, &python_dict_output)?;
            written_files.push("typed_dicts.py".to_string());
        }
        GeneratorType::DotNet => {
            let dotnet_generator = DotNetGenerator::new();
            let dotnet_output = dotnet_generator.generate_with_command(&schema, &command_string)?;

            let models_path = options.output_dir.join("Models.cs");
            write_if_changed(&models_path, &dotnet_output)?;
            written_files.push("Models.cs".to_string());
        }
        GeneratorType::JsonSchema => {
            let json_schema_generator = JsonSchemaGenerator::new();
            let json_schema_output =
                json_schema_generator.generate_with_command(&schema, &command_string)?;

            let schema_path = options.output_dir.join("schema.json");
            write_if_changed(&schema_path, &json_schema_output)?;
            written_files.push("schema.json".to_string());
        }
        GeneratorType::Zod => {
            // Generate schema.ts (Zod schemas + inferred types) and dto.ts (query/header param types)
            let zod_generator = ZodGenerator::new();
            let zod_output = zod_generator.generate_with_command(&schema, &command_string)?;

            let schema_path = options.output_dir.join("schema.ts");
            write_if_changed(&schema_path, &zod_output)?;
            written_files.push("schema.ts".to_string());

            let ts_generator = TypeScriptGenerator::new();
            let ts_output = ts_generator.generate_with_imports(&schema, &command_string)?;

            let dto_path = options.output_dir.join("dto.ts");
            write_if_changed(&dto_path, &ts_output)?;
            written_files.push("dto.ts".to_string());
        }
        GeneratorType::TypeScript | GeneratorType::Endpoints => {
            if matches!(options.generator_type, GeneratorType::Endpoints) {
                let generator = EndpointsGenerator::new();
                let output = generator.generate_with_command(&schema, &command_string)?;
                let endpoints_path = options.output_dir.join("endpoints.ts");
                write_if_changed(&endpoints_path, &output)?;
                written_files.push("endpoints.ts".to_string());
            } else if options.with_zod {
                // TypeScript + Zod (same behavior as CLI with --typescript --zod)
                let zod_generator = ZodGenerator::new();
                let zod_output = zod_generator.generate_with_command(&schema, &command_string)?;

                let schema_path = options.output_dir.join("schema.ts");
                write_if_changed(&schema_path, &zod_output)?;
                written_files.push("schema.ts".to_string());

                let ts_generator = TypeScriptGenerator::new();
                let ts_output = ts_generator.generate_with_imports(&schema, &command_string)?;

                let dto_path = options.output_dir.join("dto.ts");
                write_if_changed(&dto_path, &ts_output)?;
                written_files.push("dto.ts".to_string());
            } else {
                // TypeScript only
                let ts_generator = TypeScriptGenerator::new();
                let ts_output = ts_generator.generate_with_command(&schema, &command_string)?;

                let dto_path = options.output_dir.join("dto.ts");
                write_if_changed(&dto_path, &ts_output)?;
                written_files.push("dto.ts".to_string());
            }
        }
        GeneratorType::RustSerde => {
            let rust_generator = RustSerdeGenerator::new();
            let rust_output = rust_generator.generate_with_command(&schema, &command_string)?;

            let models_path = options.output_dir.join("models.rs");
            write_if_changed(&models_path, &rust_output)?;
            written_files.push("models.rs".to_string());
        }
    }

    if options.delete_old {
        delete_obsolete_files(&options.output_dir, &written_files)?;
    }

    // Angular already prints its own message via generate_angular_services
    if !matches!(options.generator_type, GeneratorType::Angular) {
        println!("Generated files:");
        for file in &written_files {
            println!("  - {}", options.output_dir.join(file).display());
        }
    }

    Ok(())
}

/// CLI definition (moved from main.rs) so it can be reused and tested.
#[derive(Parser)]
#[command(author, version = env!("BUILD_VERSION"), about, long_about = None)]
pub struct Cli {
    /// Input OpenAPI schema JSON file
    #[arg(long)]
    pub from_openapi: Option<PathBuf>,

    /// Input plain JSON file (for generating DTOs like quicktype.io)
    #[arg(long)]
    pub from_json: Option<PathBuf>,

    /// Input JSON Schema file (for generating DTOs from JSON Schema)
    #[arg(long)]
    pub from_json_schema: Option<PathBuf>,

    /// Name for the root class/interface when using --json (default: Root)
    #[arg(long, default_value = "Root")]
    pub root: String,

    /// Output directory path (if specified, writes dto.ts and optionally schema.ts files)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Generate TypeScript interfaces instead of Zod schemas (when not using output directory)
    #[arg(short, long)]
    pub typescript: bool,

    /// Generate Zod schemas (creates schema.ts and makes dto.ts import from it)
    #[arg(short, long)]
    pub zod: bool,

    /// Generate Angular API services (creates multiple service files and utilities)
    #[arg(short, long)]
    pub angular: bool,

    /// Generate Pydantic BaseModel classes for Python
    #[arg(long)]
    pub pydantic: bool,

    /// Pydantic version to target (1 or 2, default: 1). Implies --pydantic when specified.
    #[arg(long = "pydantic-version", value_name = "VERSION")]
    pub pydantic_version: Option<PydanticVersion>,

    /// Generate Python TypedDict definitions
    #[arg(long)]
    pub python_dict: bool,

    /// Generate C# classes with System.Text.Json serialization
    #[arg(long)]
    pub dotnet: bool,

    /// Generate JSON Schema output
    #[arg(long)]
    pub json_schema: bool,

    /// Generate API endpoint types from OpenAPI paths
    #[arg(short, long)]
    pub endpoints: bool,

    /// Generate Rust structs with Serde serialization/deserialization
    #[arg(long)]
    pub rust_serde: bool,

    /// Generate promises using lastValueFrom instead of Observables (only works with --angular)
    #[arg(long)]
    pub promises: bool,

    /// Enable debug output
    #[arg(long)]
    pub debug: bool,

    /// Hide version from generated output headers (use 'dtolator' instead of 'dtolator==VERSION')
    #[arg(long)]
    pub hide_version: bool,

    /// Skip writing specific file(s) to output directory (can be used multiple times)
    #[arg(long)]
    pub skip_file: Vec<String>,

    /// Base URL generation mode: 'global' (default) or 'argument'
    #[arg(long = "base-url-mode", default_value = "global")]
    pub base_url: BaseUrlMode,

    /// Name of the global variable used for the API base URL (only with --base-url-mode global)
    #[arg(long = "api-url-variable", default_value = "API_URL")]
    pub api_url_variable: String,

    /// Delete obsolete files from the output directory after generation
    #[arg(long)]
    pub delete_old: bool,
}

impl Cli {
    fn to_generate_options(&self, output_dir: PathBuf) -> GenerateOptions {
        let (input_type, input_path) = if let Some(path) = &self.from_openapi {
            (InputType::OpenApi, path.clone())
        } else if let Some(path) = &self.from_json {
            (InputType::Json, path.clone())
        } else if let Some(path) = &self.from_json_schema {
            (InputType::JsonSchema, path.clone())
        } else {
            unreachable!("validated that exactly one input is provided")
        };

        let generator_type = if self.angular {
            GeneratorType::Angular
        } else if self.pydantic || self.pydantic_version.is_some() {
            GeneratorType::Pydantic
        } else if self.python_dict {
            GeneratorType::PythonDict
        } else if self.dotnet {
            GeneratorType::DotNet
        } else if self.json_schema {
            GeneratorType::JsonSchema
        } else if self.endpoints {
            GeneratorType::Endpoints
        } else if self.rust_serde {
            GeneratorType::RustSerde
        } else if self.zod && !self.typescript {
            GeneratorType::Zod
        } else {
            GeneratorType::TypeScript
        };

        GenerateOptions {
            input_type,
            input_path,
            output_dir,
            generator_type,
            pydantic_version: self.pydantic_version.unwrap_or(PydanticVersion::V1),
            with_zod: self.zod,
            with_promises: self.promises,
            hide_version: self.hide_version,
            root_name: self.root.clone(),
            debug: self.debug,
            skip_files: self.skip_file.clone(),
            base_url_mode: self.base_url,
            api_url_variable: self.api_url_variable.clone(),
            delete_old: self.delete_old,
        }
    }
}

/// Run the CLI using a custom iterator of arguments (for testing).
pub fn run_cli_with_args<I, S>(args: I) -> Result<()>
where
    I: IntoIterator<Item = S>,
    S: Into<std::ffi::OsString> + Clone,
{
    let cli = Cli::parse_from(args);

    // Validate that exactly one input type is provided
    let input_count = [&cli.from_openapi, &cli.from_json, &cli.from_json_schema]
        .iter()
        .filter(|x| x.is_some())
        .count();

    if input_count == 0 {
        return Err(anyhow::anyhow!(
            "Please specify exactly one input type: --from-openapi, --from-json, or --from-json-schema"
        ));
    }

    if input_count > 1 {
        return Err(anyhow::anyhow!(
            "Please specify only one input type: --from-openapi, --from-json, or --from-json-schema"
        ));
    }

    // Validate that Angular generation is only used with OpenAPI input
    if cli.angular && cli.from_openapi.is_none() {
        return Err(anyhow::anyhow!(
            "--angular flag requires --from-openapi input. Angular services need API endpoint information that is only available in OpenAPI specifications."
        ));
    }

    // Validate that endpoints generation is only used with OpenAPI input
    if cli.endpoints && cli.from_openapi.is_none() {
        return Err(anyhow::anyhow!(
            "--endpoints flag requires --from-openapi input. API endpoint types need path information that is only available in OpenAPI specifications."
        ));
    }

    // Validate that promises flag is only used with Angular
    if cli.promises && !cli.angular {
        return Err(anyhow::anyhow!(
            "--promises flag can only be used with --angular. Use --angular --promises to generate Angular services with Promise-based methods."
        ));
    }

    match &cli.output {
        Some(output_dir) => {
            // Output directory specified — delegate to generate()
            let options = cli.to_generate_options(output_dir.clone());
            generate(options)?;
        }
        None => {
            // No output directory — parse schema and print single output to stdout
            let schema = if let Some(openapi_path) = &cli.from_openapi {
                let input_content = std::fs::read_to_string(openapi_path).with_context(|| {
                    format!("Failed to read OpenAPI file: {}", openapi_path.display())
                })?;
                serde_json::from_str::<OpenApiSchema>(&input_content)
                    .with_context(|| "Failed to parse OpenAPI schema JSON")?
            } else if let Some(json_path) = &cli.from_json {
                let input_content = std::fs::read_to_string(json_path).with_context(|| {
                    format!("Failed to read JSON file: {}", json_path.display())
                })?;
                json_to_openapi_schema_with_root(serde_json::from_str(&input_content)?, &cli.root)?
            } else if let Some(json_schema_path) = &cli.from_json_schema {
                let input_content =
                    std::fs::read_to_string(json_schema_path).with_context(|| {
                        format!(
                            "Failed to read JSON Schema file: {}",
                            json_schema_path.display()
                        )
                    })?;
                let cleaned_content = strip_json_comments(&input_content);
                json_schema_to_openapi_schema(serde_json::from_str(&cleaned_content)?, &cli.root)?
            } else {
                unreachable!()
            };

            let schema = extract_inline_request_schemas(schema)?;
            let options = cli.to_generate_options(PathBuf::new());
            let command_string = options.build_command_string();

            let output =
                match options.generator_type {
                    GeneratorType::Endpoints => {
                        EndpointsGenerator::new().generate_with_command(&schema, &command_string)?
                    }
                    GeneratorType::Angular => AngularGenerator::new()
                        .with_zod_validation(options.with_zod)
                        .with_promises(options.with_promises)
                        .with_base_url_mode(options.base_url_mode)
                        .with_api_url_variable(options.api_url_variable.clone())
                        .generate_with_command(&schema, &command_string)?,
                    GeneratorType::Pydantic => PydanticGenerator::new(options.pydantic_version)
                        .generate_with_command(&schema, &command_string)?,
                    GeneratorType::PythonDict => PythonDictGenerator::new()
                        .generate_with_command(&schema, &command_string)?,
                    GeneratorType::DotNet => {
                        DotNetGenerator::new().generate_with_command(&schema, &command_string)?
                    }
                    GeneratorType::JsonSchema => JsonSchemaGenerator::new()
                        .generate_with_command(&schema, &command_string)?,
                    GeneratorType::RustSerde => {
                        RustSerdeGenerator::new().generate_with_command(&schema, &command_string)?
                    }
                    GeneratorType::TypeScript => TypeScriptGenerator::new()
                        .generate_with_command(&schema, &command_string)?,
                    GeneratorType::Zod => {
                        ZodGenerator::new().generate_with_command(&schema, &command_string)?
                    }
                };

            println!("{output}");
        }
    }

    Ok(())
}

/// Run the CLI using the real process arguments.
pub fn run_cli() -> Result<()> {
    run_cli_with_args(std::env::args_os())
}

#[allow(clippy::too_many_arguments)]
fn generate_angular_services(
    schema: &OpenApiSchema,
    output_dir: &Path,
    with_zod: bool,
    debug: bool,
    promises: bool,
    command_string: &str,
    skip_files: &[String],
    base_url_mode: BaseUrlMode,
    api_url_variable: &str,
) -> Result<Vec<String>> {
    let angular_generator = AngularGenerator::new()
        .with_zod_validation(with_zod)
        .with_debug(debug)
        .with_promises(promises)
        .with_base_url_mode(base_url_mode)
        .with_api_url_variable(api_url_variable.to_string());
    let output = angular_generator.generate_with_command(schema, command_string)?;

    // Also generate DTOs and utility function
    let dto_path = output_dir.join("dto.ts");
    let mut files_generated: Vec<String> = Vec::new();
    if with_zod {
        // Generate Zod schemas first
        let zod_generator = ZodGenerator::new();
        let zod_output = zod_generator.generate_with_command(schema, command_string)?;

        let schema_path = output_dir.join("schema.ts");
        if !skip_files.contains(&"schema.ts".to_string()) {
            write_if_changed(&schema_path, &zod_output)?;
            files_generated.push("schema.ts".to_string());
        }

        // Generate dto.ts with query/header param types (request body types live in schema.ts)
        let ts_generator = TypeScriptGenerator::new();
        let mut ts_output = ts_generator.generate_with_imports(schema, command_string)?;

        let header_param_types = ts_generator.generate_header_param_types(schema)?;
        if !header_param_types.trim().is_empty() {
            ts_output.push('\n');
            ts_output.push_str(&header_param_types);
        }

        if ts_output.contains("export ") && !skip_files.contains(&"dto.ts".to_string()) {
            write_if_changed(&dto_path, &ts_output)?;
            files_generated.push("dto.ts".to_string());
        }
    } else {
        // Generate only TypeScript interfaces
        let ts_generator = TypeScriptGenerator::new();
        let mut dto_output = ts_generator.generate_with_command(schema, command_string)?;

        let query_param_types = ts_generator.generate_query_param_types(schema)?;
        if !query_param_types.trim().is_empty() {
            dto_output.push('\n');
            dto_output.push_str(&query_param_types);
        }

        let header_param_types = ts_generator.generate_header_param_types(schema)?;
        if !header_param_types.trim().is_empty() {
            dto_output.push('\n');
            dto_output.push_str(&header_param_types);
        }

        if !skip_files.contains(&"dto.ts".to_string()) {
            write_if_changed(&dto_path, &dto_output)?;
            files_generated.push("dto.ts".to_string());
        }
    }

    if debug {
        println!("🔍 [DEBUG] Raw Angular generator output:");
        println!("--- START OUTPUT ---");
        println!("{output}");
        println!("--- END OUTPUT ---");
    }

    // Split by the FILE markers and match content to files
    let mut current_content = String::new();
    let mut current_file = String::new();
    let mut in_file_section = false;

    for line in output.lines() {
        if debug {
            println!("🔍 [DEBUG] Processing line: {line}");
        }

        if let Some(stripped) = line.strip_prefix("// FILE: ") {
            if debug {
                println!("🔍 [DEBUG] Found FILE marker: {line}");
            }

            // If we were collecting content for a previous file, write it now
            if !current_file.is_empty() && !current_content.is_empty() {
                if debug {
                    println!(
                        "🔍 [DEBUG] Writing previous file: {} ({} chars)",
                        current_file,
                        current_content.len()
                    );
                }

                if !skip_files.contains(&current_file) {
                    let service_path = output_dir.join(&current_file);
                    write_if_changed(&service_path, &current_content)?;
                    files_generated.push(current_file.clone());
                }
            }

            // Start collecting for the new file
            current_file = stripped.to_string();
            current_content.clear();
            in_file_section = true;

            if debug {
                println!("🔍 [DEBUG] Started collecting for file: {current_file}");
            }
        } else if in_file_section {
            // If we haven't hit a FILE marker yet, this line belongs to the current file
            if !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(line);

            if debug {
                println!("🔍 [DEBUG] Added line to {current_file}: {line}");
            }
        }
    }

    // Write the last file if there is one
    if !current_file.is_empty()
        && !current_content.is_empty()
        && !skip_files.contains(&current_file)
    {
        let service_path = output_dir.join(&current_file);
        write_if_changed(&service_path, &current_content)?;
        files_generated.push(current_file.clone());
    }

    // Special case: extract the service content before the first FILE marker
    let parts: Vec<&str> = output.split("// FILE: ").collect();
    if !parts.is_empty() {
        let first_service_content = parts[0].trim();
        // Extract filename from the first FILE marker
        if !first_service_content.is_empty()
            && parts.len() > 1
            && let Some(first_marker) = parts.get(1)
            && let Some(newline_pos) = first_marker.find('\n')
        {
            let first_file_name = &first_marker[..newline_pos];
            if !skip_files.contains(&first_file_name.to_string()) {
                let service_path = output_dir.join(first_file_name);
                write_if_changed(&service_path, first_service_content)?;

                // Add to files_generated if not already there
                if !files_generated.contains(&first_file_name.to_string()) {
                    files_generated.push(first_file_name.to_string());
                }
            }
        }
    }

    println!("Generated Angular API files:");
    for file in &files_generated {
        println!("  - {}", output_dir.join(file).display());
    }

    Ok(files_generated)
}

/// Write `contents` to `path` only if the file doesn't already exist with identical content.
fn write_if_changed(path: &Path, contents: &str) -> Result<bool> {
    if let Ok(metadata) = fs::metadata(path)
        && metadata.len() == contents.len() as u64
        && let Ok(existing) = fs::read_to_string(path)
        && existing == contents
    {
        return Ok(false);
    }
    fs::write(path, contents)
        .with_context(|| format!("Failed to write file: {}", path.display()))?;
    Ok(true)
}

/// Delete files in `dir` that are not in `keep` (filenames only, not paths).
fn delete_obsolete_files(dir: &Path, keep: &[String]) -> Result<()> {
    for entry in fs::read_dir(dir)
        .with_context(|| format!("Failed to read output directory: {}", dir.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        if let Some(name) = entry.file_name().to_str()
            && !keep.contains(&name.to_string())
        {
            fs::remove_file(entry.path()).with_context(|| {
                format!("Failed to delete obsolete file: {}", entry.path().display())
            })?;
            println!("Deleted obsolete file: {}", entry.path().display());
        }
    }
    Ok(())
}

fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn json_value_to_schema_pass1(
    value: &serde_json::Value,
    schemas: &mut IndexMap<String, Schema>,
    current_name: &str,
    struct_hashes: &mut StructHashMap,
    json_to_placeholder: &mut JsonToPlaceholderMap,
    parent_key: Option<&str>,
) -> Result<Schema> {
    match value {
        serde_json::Value::Null => Ok(Schema::null()),
        serde_json::Value::Bool(_) => Ok(Schema::boolean()),
        serde_json::Value::Number(n) => {
            if n.is_i64() || n.is_u64() {
                Ok(Schema::integer())
            } else {
                Ok(Schema::number())
            }
        }
        serde_json::Value::String(_) => Ok(Schema::string()),
        serde_json::Value::Array(arr) => {
            if arr.is_empty() {
                Ok(Schema::array(
                    Schema::object().schema_type("object").build(),
                ))
            } else {
                // Create a meaningful name for the array item type
                let item_name = if let Some(parent_key) = parent_key {
                    // Use parent key to create meaningful type name
                    let mut name = capitalize_first_letter(parent_key);
                    // Remove plural 's' to get singular item name
                    if name.ends_with("s") && name.len() > 1 {
                        name.pop();
                    }
                    // Handle special cases
                    match name.as_str() {
                        "Item" => format!("{current_name}Item"),
                        "Data" => format!("{current_name}DataItem"),
                        _ => name,
                    }
                } else {
                    format!("{current_name}Item")
                };

                let item_schema = json_value_to_schema_pass1(
                    &arr[0],
                    schemas,
                    &item_name,
                    struct_hashes,
                    json_to_placeholder,
                    Some(&item_name.to_lowercase()),
                )?;
                Ok(Schema::array(item_schema))
            }
        }
        serde_json::Value::Object(obj) => {
            if obj.is_empty() {
                return Ok(Schema::object().schema_type("object").build());
            }

            // Use JSON content for deduplication
            let serialized = serde_json::to_string(obj)?;

            if let Some(placeholder) = json_to_placeholder.get(&serialized) {
                // Same structure, reuse existing schema
                return Ok(Schema::reference(format!(
                    "#/components/schemas/{placeholder}"
                )));
            }

            // Generate properties
            let mut properties = IndexMap::new();
            let mut required = Vec::new();

            for (key, value) in obj {
                // Create meaningful names for nested objects
                let property_name = match value {
                    serde_json::Value::Object(_) => {
                        // For nested objects, create a meaningful type name
                        let base_name = capitalize_first_letter(key);
                        // If the key itself is meaningful, use it; otherwise derive from parent
                        if base_name.len() > 2 && !base_name.ends_with("s") {
                            base_name
                        } else {
                            format!("{current_name}{base_name}")
                        }
                    }
                    serde_json::Value::Array(arr) => {
                        // For arrays containing objects, create a meaningful type name for container
                        if !arr.is_empty() && matches!(arr[0], serde_json::Value::Object(_)) {
                            // This is just for the array property context, actual item naming happens in array processing
                            current_name.to_string()
                        } else {
                            current_name.to_string()
                        }
                    }
                    _ => capitalize_first_letter(key),
                };

                let property_schema = json_value_to_schema_pass1(
                    value,
                    schemas,
                    &property_name,
                    struct_hashes,
                    json_to_placeholder,
                    Some(key),
                )?;
                properties.insert(key.clone(), property_schema);
                required.push(key.clone());
            }

            let placeholder_name = current_name.to_string();
            json_to_placeholder.insert(serialized.clone(), placeholder_name.clone());
            struct_hashes.insert(
                serialized.clone(),
                (current_name.to_string(), required.clone(), None, vec![]),
            );

            let mut builder = Schema::object()
                .schema_type("object")
                .properties(properties);

            if !required.is_empty() {
                builder = builder.required(required);
            }

            let schema = builder.build();

            schemas.insert(current_name.to_string(), schema.clone());

            // Only return a reference if this is not the root level object
            if current_name != "Root" && parent_key.is_some() {
                Ok(Schema::reference(format!(
                    "#/components/schemas/{current_name}"
                )))
            } else {
                Ok(schema)
            }
        }
    }
}

fn resolve_placeholders(
    schema: &mut Schema,
    hash_to_final_name: &std::collections::HashMap<String, String>,
) {
    match schema {
        Schema::Reference { reference, .. } => {
            for (placeholder, final_name) in hash_to_final_name {
                let placeholder_ref = format!("#/components/schemas/{placeholder}");
                if reference == &placeholder_ref {
                    *reference = format!("#/components/schemas/{final_name}");
                }
            }
        }
        Schema::Object {
            properties,
            items,
            all_of,
            one_of,
            any_of,
            ..
        } => {
            if let Some(props) = properties {
                for (_key, prop_schema) in props.iter_mut() {
                    resolve_placeholders(prop_schema, hash_to_final_name);
                }
            }
            if let Some(item_schema) = items {
                resolve_placeholders(item_schema, hash_to_final_name);
            }
            if let Some(schemas) = all_of {
                for s in schemas.iter_mut() {
                    resolve_placeholders(s, hash_to_final_name);
                }
            }
            if let Some(schemas) = one_of {
                for s in schemas.iter_mut() {
                    resolve_placeholders(s, hash_to_final_name);
                }
            }
            if let Some(schemas) = any_of {
                for s in schemas.iter_mut() {
                    resolve_placeholders(s, hash_to_final_name);
                }
            }
        }
    }
}

fn json_to_openapi_schema_with_root(
    json_value: serde_json::Value,
    root_name: &str,
) -> Result<OpenApiSchema> {
    let mut schemas = IndexMap::new();
    let mut struct_hashes: StructHashMap = std::collections::HashMap::new();
    let mut json_to_placeholder: JsonToPlaceholderMap = std::collections::HashMap::new();
    let root_schema = json_value_to_schema_pass1(
        &json_value,
        &mut schemas,
        root_name,
        &mut struct_hashes,
        &mut json_to_placeholder,
        None,
    )?;
    schemas.insert(root_name.to_string(), root_schema);
    let json_to_final_name: std::collections::HashMap<String, String> = json_to_placeholder
        .iter()
        .map(|(json_key, placeholder)| {
            let (final_name, _, _, _) = struct_hashes.get(json_key).unwrap();
            (placeholder.clone(), final_name.clone())
        })
        .collect();
    for (_k, schema) in schemas.iter_mut() {
        resolve_placeholders(schema, &json_to_final_name);
    }
    Ok(OpenApiSchema {
        openapi: "3.0.0".to_string(),
        info: Info {
            title: "Generated from JSON".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Schema generated from plain JSON input".to_string()),
        },
        components: Some(Components {
            schemas: Some(schemas),
        }),
        paths: None,
    })
}

// JSON Schema structures for parsing
#[derive(Debug, Clone, Deserialize, Serialize)]
struct JsonSchemaDefinition {
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    #[serde(rename = "$defs")]
    pub defs: Option<IndexMap<String, JsonSchemaObject>>,
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
    #[serde(flatten)]
    pub root: JsonSchemaObject,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct JsonSchemaObject {
    #[serde(rename = "type")]
    pub schema_type: Option<JsonSchemaType>,
    pub properties: Option<IndexMap<String, JsonSchemaObject>>,
    pub required: Option<Vec<String>>,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<serde_json::Value>,
    pub items: Option<Box<JsonSchemaObject>>,
    #[serde(rename = "enum")]
    pub enum_values: Option<Vec<serde_json::Value>>,
    pub format: Option<String>,
    pub description: Option<String>,
    pub title: Option<String>,
    pub example: Option<serde_json::Value>,
    #[serde(rename = "allOf")]
    pub all_of: Option<Vec<JsonSchemaObject>>,
    #[serde(rename = "oneOf")]
    pub one_of: Option<Vec<JsonSchemaObject>>,
    #[serde(rename = "anyOf")]
    pub any_of: Option<Vec<JsonSchemaObject>>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    #[serde(rename = "minLength")]
    pub min_length: Option<usize>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum JsonSchemaType {
    Single(String),
    Multiple(Vec<String>),
}

fn json_schema_to_openapi_schema(
    json_schema: JsonSchemaDefinition,
    root_name: &str,
) -> Result<OpenApiSchema> {
    let mut schemas = IndexMap::new();

    // Process $defs first if they exist
    if let Some(defs) = &json_schema.defs {
        for (name, schema_obj) in defs {
            let openapi_schema = json_schema_object_to_openapi_schema(schema_obj)?;
            schemas.insert(name.clone(), openapi_schema);
        }
    }

    // Process the root schema
    let root_schema = if let Some(ref_path) = &json_schema.reference {
        // Root is a reference to a def
        Schema::Reference {
            reference: convert_json_schema_ref(ref_path)?,
            description: None,
        }
    } else {
        // Root is an inline schema
        json_schema_object_to_openapi_schema(&json_schema.root)?
    };

    // Add root schema if it's not just a reference
    if matches!(root_schema, Schema::Object { .. }) {
        schemas.insert(root_name.to_string(), root_schema);
    } else if let Schema::Reference { .. } = root_schema {
        // If root is a reference, we still need to add it under the root name for consistency
        schemas.insert(root_name.to_string(), root_schema);
    }

    // Extract metadata from JSON Schema
    let title = json_schema
        .root
        .title
        .clone()
        .unwrap_or_else(|| "Generated from JSON Schema".to_string());
    let description = json_schema
        .root
        .description
        .clone()
        .unwrap_or_else(|| "Schema generated from JSON Schema input".to_string());

    Ok(OpenApiSchema {
        openapi: "3.0.0".to_string(),
        info: Info {
            title,
            version: "1.0.0".to_string(),
            description: Some(description),
        },
        components: Some(Components {
            schemas: Some(schemas),
        }),
        paths: None,
    })
}

fn json_schema_object_to_openapi_schema(json_schema: &JsonSchemaObject) -> Result<Schema> {
    // Handle references first
    if let Some(ref_path) = &json_schema.reference {
        return Ok(Schema::reference(convert_json_schema_ref(ref_path)?));
    }

    // Handle composition schemas
    if let Some(all_of) = &json_schema.all_of {
        let schemas: Result<Vec<Schema>> = all_of
            .iter()
            .map(json_schema_object_to_openapi_schema)
            .collect();

        let mut builder = Schema::object().all_of(schemas?);

        if let Some(desc) = &json_schema.description {
            builder = builder.description(desc);
        }
        if let Some(example) = &json_schema.example {
            builder = builder.example(example.clone());
        }
        if let Some(min) = json_schema.minimum {
            builder = builder.minimum(min);
        }
        if let Some(max) = json_schema.maximum {
            builder = builder.maximum(max);
        }
        if let Some(min_len) = json_schema.min_length {
            builder = builder.min_length(min_len);
        }
        if let Some(max_len) = json_schema.max_length {
            builder = builder.max_length(max_len);
        }
        if let Some(pattern) = &json_schema.pattern {
            builder = builder.pattern(pattern);
        }

        return Ok(builder.build());
    }

    if let Some(one_of) = &json_schema.one_of {
        let schemas: Result<Vec<Schema>> = one_of
            .iter()
            .map(json_schema_object_to_openapi_schema)
            .collect();

        let mut builder = Schema::object().one_of(schemas?);

        if let Some(desc) = &json_schema.description {
            builder = builder.description(desc);
        }
        if let Some(example) = &json_schema.example {
            builder = builder.example(example.clone());
        }
        if let Some(min) = json_schema.minimum {
            builder = builder.minimum(min);
        }
        if let Some(max) = json_schema.maximum {
            builder = builder.maximum(max);
        }
        if let Some(min_len) = json_schema.min_length {
            builder = builder.min_length(min_len);
        }
        if let Some(max_len) = json_schema.max_length {
            builder = builder.max_length(max_len);
        }
        if let Some(pattern) = &json_schema.pattern {
            builder = builder.pattern(pattern);
        }

        return Ok(builder.build());
    }

    if let Some(any_of) = &json_schema.any_of {
        let schemas: Result<Vec<Schema>> = any_of
            .iter()
            .map(json_schema_object_to_openapi_schema)
            .collect();

        let mut builder = Schema::object().any_of(schemas?);

        if let Some(desc) = &json_schema.description {
            builder = builder.description(desc);
        }
        if let Some(example) = &json_schema.example {
            builder = builder.example(example.clone());
        }
        if let Some(min) = json_schema.minimum {
            builder = builder.minimum(min);
        }
        if let Some(max) = json_schema.maximum {
            builder = builder.maximum(max);
        }
        if let Some(min_len) = json_schema.min_length {
            builder = builder.min_length(min_len);
        }
        if let Some(max_len) = json_schema.max_length {
            builder = builder.max_length(max_len);
        }
        if let Some(pattern) = &json_schema.pattern {
            builder = builder.pattern(pattern);
        }

        return Ok(builder.build());
    }

    // Handle regular schema types
    let (schema_type, nullable) = match &json_schema.schema_type {
        Some(JsonSchemaType::Single(type_str)) => (Some(type_str.clone()), false),
        Some(JsonSchemaType::Multiple(types)) => {
            // Handle union types like ["string", "null"]
            let non_null_types: Vec<&String> = types.iter().filter(|t| *t != "null").collect();
            let has_null = types.iter().any(|t| t == "null");

            if non_null_types.len() == 1 {
                (Some(non_null_types[0].clone()), has_null)
            } else if non_null_types.is_empty() {
                (Some("null".to_string()), true)
            } else {
                // Multiple non-null types - this is more complex, for now just take the first
                (Some(non_null_types[0].clone()), has_null)
            }
        }
        None => (None, false),
    };

    // Convert properties
    let properties = if let Some(props) = &json_schema.properties {
        let mut openapi_props = IndexMap::new();
        for (key, prop_schema) in props {
            openapi_props.insert(
                key.clone(),
                json_schema_object_to_openapi_schema(prop_schema)?,
            );
        }
        Some(openapi_props)
    } else {
        None
    };

    // Handle array items
    let items = if let Some(items_schema) = &json_schema.items {
        Some(Box::new(json_schema_object_to_openapi_schema(
            items_schema,
        )?))
    } else {
        None
    };

    // Convert additional properties
    let additional_properties = match &json_schema.additional_properties {
        Some(serde_json::Value::Bool(false)) => None, // Strict mode in JSON Schema
        Some(serde_json::Value::Bool(true)) => None,  // Allow any additional properties
        Some(value) if value.is_object() => {
            let ap: JsonSchemaObject = serde_json::from_value(value.clone())?;
            Some(AdditionalProperties::Schema(Box::new(
                json_schema_object_to_openapi_schema(&ap)?,
            )))
        }
        Some(_) => None,
        None => None,
    };

    let mut builder = Schema::object();

    if let Some(schema_type) = schema_type {
        builder = builder.schema_type(schema_type);
    }
    if let Some(properties) = properties {
        builder = builder.properties(properties);
    }
    if let Some(required) = json_schema.required.clone() {
        builder = builder.required(required);
    }
    if let Some(additional_properties) = additional_properties {
        builder = builder.additional_properties(additional_properties);
    }
    if let Some(items) = items {
        builder = builder.items(items);
    }
    if let Some(enum_values) = json_schema.enum_values.clone() {
        builder = builder.enum_values(enum_values);
    }
    if let Some(format) = json_schema.format.clone() {
        builder = builder.format(format);
    }
    if let Some(description) = json_schema.description.clone() {
        builder = builder.description(description);
    }
    if let Some(example) = json_schema.example.clone() {
        builder = builder.example(example);
    }
    if let Some(minimum) = json_schema.minimum {
        builder = builder.minimum(minimum);
    }
    if let Some(maximum) = json_schema.maximum {
        builder = builder.maximum(maximum);
    }
    if let Some(min_length) = json_schema.min_length {
        builder = builder.min_length(min_length);
    }
    if let Some(max_length) = json_schema.max_length {
        builder = builder.max_length(max_length);
    }
    if let Some(pattern) = json_schema.pattern.clone() {
        builder = builder.pattern(pattern);
    }
    if nullable {
        builder = builder.nullable(true);
    }

    Ok(builder.build())
}

fn convert_json_schema_ref(json_schema_ref: &str) -> Result<String> {
    // Convert JSON Schema $ref format to OpenAPI format
    // JSON Schema: "#/$defs/MyType" -> OpenAPI: "#/components/schemas/MyType"
    if let Some(def_name) = json_schema_ref.strip_prefix("#/$defs/") {
        Ok(format!("#/components/schemas/{def_name}"))
    } else if let Some(def_name) = json_schema_ref.strip_prefix("#/definitions/") {
        // Also support older JSON Schema format
        Ok(format!("#/components/schemas/{def_name}"))
    } else {
        // Pass through other reference formats
        Ok(json_schema_ref.to_string())
    }
}

fn strip_json_comments(content: &str) -> String {
    // Remove JavaScript-style comments from JSON content
    // This handles /* ... */ style comments that might be in generated JSON Schema files
    let mut result = String::new();
    let mut chars = content.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '/' && chars.peek() == Some(&'*') {
            // Start of /* comment - skip until we find */
            chars.next(); // consume the *
            let mut prev_was_star = false;
            for comment_ch in chars.by_ref() {
                if prev_was_star && comment_ch == '/' {
                    break;
                }
                prev_was_star = comment_ch == '*';
            }
            // Skip any trailing whitespace/newlines after comment
            while let Some(&whitespace_ch) = chars.peek() {
                if whitespace_ch.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
        } else {
            // Regular character, keep it
            result.push(ch);
        }
    }

    result
}

/// Extract inline request body schemas (especially multipart/form-data) and add them to components.schemas
fn extract_inline_request_schemas(mut schema: OpenApiSchema) -> Result<OpenApiSchema> {
    let mut new_schemas = IndexMap::new();

    if let Some(paths) = &mut schema.paths {
        for (_path, path_item) in paths {
            let operations = [
                &mut path_item.get,
                &mut path_item.post,
                &mut path_item.put,
                &mut path_item.delete,
                &mut path_item.patch,
            ];

            for operation_opt in operations {
                if let Some(operation) = operation_opt
                    && let Some(request_body) = &mut operation.request_body
                {
                    // Check for multipart/form-data or any other content type with inline schema
                    for (_content_type, media_type) in request_body.content.iter_mut() {
                        // Only extract if it's an inline schema (not a reference)
                        if let Some(inline_schema) = &media_type.schema
                            && matches!(inline_schema, Schema::Object { .. })
                        {
                            // Generate a DTO name, preferring operationId over summary
                            let dto_name =
                                generate_dto_name(&operation.operation_id, &operation.summary);

                            // Clone the schema and add it to new_schemas
                            new_schemas.insert(dto_name.clone(), inline_schema.clone());

                            // Replace the inline schema with a reference
                            media_type.schema = Some(Schema::Reference {
                                reference: format!("#/components/schemas/{}", dto_name),
                                description: None,
                            });
                        }
                    }
                }
            }
        }
    }

    // Add the extracted schemas to components.schemas
    if !new_schemas.is_empty() {
        if let Some(components) = &mut schema.components {
            if let Some(existing_schemas) = &mut components.schemas {
                for (name, inline_schema) in new_schemas {
                    // Only add if not already present
                    if !existing_schemas.contains_key(&name) {
                        existing_schemas.insert(name, inline_schema);
                    }
                }
            } else {
                components.schemas = Some(new_schemas);
            }
        } else {
            schema.components = Some(Components {
                schemas: Some(new_schemas),
            });
        }
    }

    Ok(schema)
}

/// Generate a DTO name from an operation, preferring operationId over summary.
fn generate_dto_name(operation_id: &Option<String>, summary: &Option<String>) -> String {
    if let Some(id) = operation_id {
        // operationId is typically camelCase; capitalize the first letter for PascalCase
        let pascal = {
            let mut chars = id.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        };
        format!("{}Dto", pascal)
    } else if let Some(summary) = summary {
        format!("{}Dto", generators::common::summary_to_pascal_case(summary))
    } else {
        "UnknownDto".to_string()
    }
}
