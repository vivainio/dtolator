use anyhow::{Context, Result};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

mod generators;
mod openapi;

use generators::{
    angular::AngularGenerator, dotnet::DotNetGenerator, endpoints::EndpointsGenerator,
    json_schema::JsonSchemaGenerator, pydantic::PydanticGenerator,
    python_dict::PythonDictGenerator, typescript::TypeScriptGenerator, zod::ZodGenerator,
    Generator,
};
use indexmap::IndexMap;
use openapi::{Components, Info, OpenApiSchema, Schema};

// Type aliases to reduce complexity
type StructHashMap =
    std::collections::HashMap<String, (String, Vec<String>, Option<String>, Vec<String>)>;
type JsonToPlaceholderMap = std::collections::HashMap<String, String>;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input OpenAPI schema JSON file
    #[arg(long)]
    from_openapi: Option<PathBuf>,

    /// Input plain JSON file (for generating DTOs like quicktype.io)
    #[arg(long)]
    from_json: Option<PathBuf>,

    /// Input JSON Schema file (for generating DTOs from JSON Schema)
    #[arg(long)]
    from_json_schema: Option<PathBuf>,

    /// Name for the root class/interface when using --json (default: Root)
    #[arg(long, default_value = "Root")]
    root: String,

    /// Output directory path (if specified, writes dto.ts and optionally schema.ts files)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Generate TypeScript interfaces instead of Zod schemas (when not using output directory)
    #[arg(short, long)]
    typescript: bool,

    /// Generate Zod schemas (creates schema.ts and makes dto.ts import from it)
    #[arg(short, long)]
    zod: bool,

    /// Generate Angular API services (creates multiple service files and utilities)
    #[arg(short, long)]
    angular: bool,

    /// Generate Pydantic BaseModel classes for Python
    #[arg(long)]
    pydantic: bool,

    /// Generate Python TypedDict definitions
    #[arg(long)]
    python_dict: bool,

    /// Generate C# classes with System.Text.Json serialization
    #[arg(long)]
    dotnet: bool,

    /// Generate JSON Schema output
    #[arg(long)]
    json_schema: bool,

    /// Generate API endpoint types from OpenAPI paths
    #[arg(short, long)]
    endpoints: bool,

    /// Generate promises using lastValueFrom instead of Observables (only works with --angular)
    #[arg(long)]
    promises: bool,

    /// Enable debug output
    #[arg(long)]
    debug: bool,
}

impl Cli {
    fn build_command_string(&self) -> String {
        let mut parts = vec!["dtolator".to_string()];

        if let Some(openapi_path) = &self.from_openapi {
            let filename = openapi_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");
            parts.push(format!("--from-openapi {filename}"));
        }

        if let Some(json_path) = &self.from_json {
            let filename = json_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");
            parts.push(format!("--from-json {filename}"));
        }

        if let Some(json_schema_path) = &self.from_json_schema {
            let filename = json_schema_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");
            parts.push(format!("--from-json-schema {filename}"));
        }

        // Skip output directory in command string as it's usually a temp directory
        // and makes tests non-deterministic

        if self.typescript {
            parts.push("--typescript".to_string());
        }

        if self.zod {
            parts.push("--zod".to_string());
        }

        if self.angular {
            parts.push("--angular".to_string());
        }

        if self.pydantic {
            parts.push("--pydantic".to_string());
        }

        if self.python_dict {
            parts.push("--python-dict".to_string());
        }

        if self.dotnet {
            parts.push("--dotnet".to_string());
        }

        if self.json_schema {
            parts.push("--json-schema".to_string());
        }

        if self.endpoints {
            parts.push("--endpoints".to_string());
        }

        if self.promises {
            parts.push("--promises".to_string());
        }

        if self.debug {
            parts.push("--debug".to_string());
        }

        parts.join(" ")
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let command_string = cli.build_command_string();

    // Validate that exactly one input type is provided
    let input_count = [&cli.from_openapi, &cli.from_json, &cli.from_json_schema]
        .iter()
        .filter(|x| x.is_some())
        .count();

    if input_count == 0 {
        return Err(anyhow::anyhow!("Please specify exactly one input type: --from-openapi, --from-json, or --from-json-schema"));
    }

    if input_count > 1 {
        return Err(anyhow::anyhow!("Please specify only one input type: --from-openapi, --from-json, or --from-json-schema"));
    }

    // Validate that Angular generation is only used with OpenAPI input
    if cli.angular && cli.from_openapi.is_none() {
        return Err(anyhow::anyhow!("--angular flag requires --from-openapi input. Angular services need API endpoint information that is only available in OpenAPI specifications."));
    }

    // Validate that endpoints generation is only used with OpenAPI input
    if cli.endpoints && cli.from_openapi.is_none() {
        return Err(anyhow::anyhow!("--endpoints flag requires --from-openapi input. API endpoint types need path information that is only available in OpenAPI specifications."));
    }

    // Validate that promises flag is only used with Angular
    if cli.promises && !cli.angular {
        return Err(anyhow::anyhow!("--promises flag can only be used with --angular. Use --angular --promises to generate Angular services with Promise-based methods."));
    }

    // Read and parse the input file
    let schema = if let Some(openapi_path) = &cli.from_openapi {
        // Read and parse OpenAPI schema
        let input_content = std::fs::read_to_string(openapi_path)
            .with_context(|| format!("Failed to read OpenAPI file: {}", openapi_path.display()))?;

        serde_json::from_str::<OpenApiSchema>(&input_content)
            .with_context(|| "Failed to parse OpenAPI schema JSON")?
    } else if let Some(json_path) = &cli.from_json {
        // Read and parse plain JSON, then convert to OpenAPI schema
        let input_content = std::fs::read_to_string(json_path)
            .with_context(|| format!("Failed to read JSON file: {}", json_path.display()))?;

        json_to_openapi_schema_with_root(serde_json::from_str(&input_content)?, &cli.root)?
    } else if let Some(json_schema_path) = &cli.from_json_schema {
        // Read and parse JSON Schema, then convert to OpenAPI schema
        let input_content = std::fs::read_to_string(json_schema_path).with_context(|| {
            format!(
                "Failed to read JSON Schema file: {}",
                json_schema_path.display()
            )
        })?;

        // Strip JavaScript-style comments that might be in generated JSON Schema files
        let cleaned_content = strip_json_comments(&input_content);

        json_schema_to_openapi_schema(serde_json::from_str(&cleaned_content)?, &cli.root)?
    } else {
        unreachable!() // We validated above that exactly one of them is Some
    };

    match cli.output {
        Some(output_dir) => {
            // Output directory specified - generate files
            fs::create_dir_all(&output_dir).with_context(|| {
                format!(
                    "Failed to create output directory: {}",
                    output_dir.display()
                )
            })?;

            if cli.angular {
                // Generate Angular services with multiple files
                generate_angular_services(
                    &schema,
                    &output_dir,
                    cli.zod,
                    cli.debug,
                    cli.promises,
                    &command_string,
                )?;
            } else if cli.pydantic {
                // Generate Pydantic models to a Python file
                let pydantic_generator = PydanticGenerator::new();
                let pydantic_output =
                    pydantic_generator.generate_with_command(&schema, &command_string)?;

                let models_path = output_dir.join("models.py");
                fs::write(&models_path, pydantic_output).with_context(|| {
                    format!("Failed to write models.py file: {}", models_path.display())
                })?;

                println!("Generated files:");
                println!("  - {}", models_path.display());
            } else if cli.python_dict {
                // Generate Python TypedDict definitions to a Python file
                let python_dict_generator = PythonDictGenerator::new();
                let python_dict_output =
                    python_dict_generator.generate_with_command(&schema, &command_string)?;

                let typed_dicts_path = output_dir.join("typed_dicts.py");
                fs::write(&typed_dicts_path, python_dict_output).with_context(|| {
                    format!(
                        "Failed to write typed_dicts.py file: {}",
                        typed_dicts_path.display()
                    )
                })?;

                println!("Generated files:");
                println!("  - {}", typed_dicts_path.display());
            } else if cli.dotnet {
                // Generate C# classes to a C# file
                let dotnet_generator = DotNetGenerator::new();
                let dotnet_output =
                    dotnet_generator.generate_with_command(&schema, &command_string)?;

                let models_path = output_dir.join("Models.cs");
                fs::write(&models_path, dotnet_output).with_context(|| {
                    format!("Failed to write Models.cs file: {}", models_path.display())
                })?;

                println!("Generated files:");
                println!("  - {}", models_path.display());
            } else if cli.json_schema {
                // Generate JSON Schema to a JSON file
                let json_schema_generator = JsonSchemaGenerator::new();
                let json_schema_output =
                    json_schema_generator.generate_with_command(&schema, &command_string)?;

                let schema_path = output_dir.join("schema.json");
                fs::write(&schema_path, json_schema_output).with_context(|| {
                    format!(
                        "Failed to write schema.json file: {}",
                        schema_path.display()
                    )
                })?;

                println!("Generated files:");
                println!("  - {}", schema_path.display());
            } else if cli.zod {
                // Generate both dto.ts (with imports) and schema.ts

                // Generate Zod schemas first
                let zod_generator = ZodGenerator::new();
                let zod_output = zod_generator.generate_with_command(&schema, &command_string)?;

                let schema_path = output_dir.join("schema.ts");
                fs::write(&schema_path, zod_output).with_context(|| {
                    format!("Failed to write schema.ts file: {}", schema_path.display())
                })?;

                // Generate TypeScript interfaces that import from schema.ts
                let ts_generator = TypeScriptGenerator::new();
                let ts_output = ts_generator.generate_with_imports(&schema, &command_string)?;

                let dto_path = output_dir.join("dto.ts");
                fs::write(&dto_path, ts_output).with_context(|| {
                    format!("Failed to write dto.ts file: {}", dto_path.display())
                })?;

                println!("Generated files:");
                println!("  - {}", dto_path.display());
                println!("  - {}", schema_path.display());
            } else {
                // Generate only dto.ts with TypeScript interfaces
                let ts_generator = TypeScriptGenerator::new();
                let ts_output = ts_generator.generate_with_command(&schema, &command_string)?;

                let dto_path = output_dir.join("dto.ts");
                fs::write(&dto_path, ts_output).with_context(|| {
                    format!("Failed to write dto.ts file: {}", dto_path.display())
                })?;

                println!("Generated files:");
                println!("  - {}", dto_path.display());
            }
        }
        None => {
            // No output directory - use original single-output behavior with stdout
            let output = if cli.endpoints {
                let generator = EndpointsGenerator::new();
                generator.generate_with_command(&schema, &command_string)?
            } else if cli.typescript {
                let generator = TypeScriptGenerator::new();
                generator.generate_with_command(&schema, &command_string)?
            } else if cli.angular {
                let generator = AngularGenerator::new().with_promises(cli.promises);
                generator.generate_with_command(&schema, &command_string)?
            } else if cli.pydantic {
                let generator = PydanticGenerator::new();
                generator.generate_with_command(&schema, &command_string)?
            } else if cli.python_dict {
                let generator = PythonDictGenerator::new();
                generator.generate_with_command(&schema, &command_string)?
            } else if cli.dotnet {
                let generator = DotNetGenerator::new();
                generator.generate_with_command(&schema, &command_string)?
            } else if cli.json_schema {
                let generator = JsonSchemaGenerator::new();
                generator.generate_with_command(&schema, &command_string)?
            } else {
                let generator = ZodGenerator::new();
                generator.generate_with_command(&schema, &command_string)?
            };

            println!("{output}");
        }
    }

    Ok(())
}

fn generate_angular_services(
    schema: &OpenApiSchema,
    output_dir: &Path,
    with_zod: bool,
    debug: bool,
    promises: bool,
    command_string: &str,
) -> Result<()> {
    let angular_generator = AngularGenerator::new()
        .with_zod_validation(with_zod)
        .with_debug(debug)
        .with_promises(promises);
    let output = angular_generator.generate_with_command(schema, command_string)?;

    // Also generate DTOs and utility function
    let dto_path = output_dir.join("dto.ts");

    if with_zod {
        // Generate Zod schemas first
        let zod_generator = ZodGenerator::new();
        let zod_output = zod_generator.generate_with_command(schema, command_string)?;

        let schema_path = output_dir.join("schema.ts");
        fs::write(&schema_path, zod_output).with_context(|| {
            format!("Failed to write schema.ts file: {}", schema_path.display())
        })?;

        // Generate TypeScript interfaces that re-export from schema.ts
        let ts_generator = TypeScriptGenerator::new();
        let ts_output = ts_generator.generate_with_imports(schema, command_string)?;

        fs::write(&dto_path, ts_output)
            .with_context(|| format!("Failed to write dto.ts file: {}", dto_path.display()))?;
    } else {
        // Generate only TypeScript interfaces
        let ts_generator = TypeScriptGenerator::new();
        let mut dto_output = ts_generator.generate_with_command(schema, command_string)?;

        // Add query parameter types for Angular services
        let angular_generator = AngularGenerator::new();
        let query_param_types = angular_generator.generate_query_param_types(schema)?;
        if !query_param_types.trim().is_empty() {
            dto_output.push('\n');
            dto_output.push_str(&query_param_types);
        }

        fs::write(&dto_path, dto_output)
            .with_context(|| format!("Failed to write dto.ts file: {}", dto_path.display()))?;
    }

    // Generate fill-url utility function
    let angular_generator = AngularGenerator::new();
    let fill_url_content = angular_generator.generate_fill_url_func(command_string);
    let fill_url_path = output_dir.join("fill-url.ts");
    fs::write(&fill_url_path, fill_url_content).with_context(|| {
        format!(
            "Failed to write fill-url.ts file: {}",
            fill_url_path.display()
        )
    })?;

    // Parse and split the Angular generator output into individual service files
    let mut files_generated = vec![
        dto_path.display().to_string(),
        fill_url_path.display().to_string(),
    ];

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

                let service_path = output_dir.join(&current_file);
                fs::write(&service_path, &current_content).with_context(|| {
                    format!(
                        "Failed to write {} file: {}",
                        current_file,
                        service_path.display()
                    )
                })?;
                files_generated.push(service_path.display().to_string());
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
    if !current_file.is_empty() && !current_content.is_empty() {
        let service_path = output_dir.join(&current_file);
        fs::write(&service_path, &current_content).with_context(|| {
            format!(
                "Failed to write {} file: {}",
                current_file,
                service_path.display()
            )
        })?;
        files_generated.push(service_path.display().to_string());
    }

    // Special case: extract the service content before the first FILE marker
    let parts: Vec<&str> = output.split("// FILE: ").collect();
    if !parts.is_empty() {
        let first_service_content = parts[0].trim();
        if !first_service_content.is_empty() && parts.len() > 1 {
            // Extract filename from the first FILE marker
            if let Some(first_marker) = parts.get(1) {
                if let Some(newline_pos) = first_marker.find('\n') {
                    let first_file_name = &first_marker[..newline_pos];
                    let service_path = output_dir.join(first_file_name);
                    fs::write(&service_path, first_service_content).with_context(|| {
                        format!(
                            "Failed to write {} file: {}",
                            first_file_name,
                            service_path.display()
                        )
                    })?;

                    // Add to files_generated if not already there
                    let file_path_str = service_path.display().to_string();
                    if !files_generated.contains(&file_path_str) {
                        files_generated.push(file_path_str);
                    }
                }
            }
        }
    }

    println!("Generated Angular API files:");
    for file in files_generated {
        println!("  - {file}");
    }

    Ok(())
}

#[allow(dead_code)]
fn longest_common_suffix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    let revs: Vec<Vec<char>> = strings.iter().map(|s| s.chars().rev().collect()).collect();
    let mut suffix = Vec::new();
    for i in 0..revs[0].len() {
        let c = revs[0][i];
        if revs
            .iter()
            .all(|r| r.len() > i && r[i].eq_ignore_ascii_case(&c))
        {
            suffix.push(c);
        } else {
            break;
        }
    }
    suffix.into_iter().rev().collect()
}

fn capitalize_first_letter(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

#[allow(dead_code)]
fn update_refs(schemas: &mut IndexMap<String, Schema>, old_names: &[String], new_name: &str) {
    fn update_schema_refs(schema: &mut Schema, old_names: &[String], new_name: &str) {
        match schema {
            Schema::Reference { reference } => {
                for old in old_names {
                    let old_ref = format!("#/components/schemas/{old}");
                    if reference == &old_ref {
                        *reference = format!("#/components/schemas/{new_name}");
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
                    for (_k, v) in props.iter_mut() {
                        update_schema_refs(v, old_names, new_name);
                    }
                }
                if let Some(item) = items {
                    update_schema_refs(item, old_names, new_name);
                }
                if let Some(schemas) = all_of {
                    for s in schemas.iter_mut() {
                        update_schema_refs(s, old_names, new_name);
                    }
                }
                if let Some(schemas) = one_of {
                    for s in schemas.iter_mut() {
                        update_schema_refs(s, old_names, new_name);
                    }
                }
                if let Some(schemas) = any_of {
                    for s in schemas.iter_mut() {
                        update_schema_refs(s, old_names, new_name);
                    }
                }
            }
        }
    }
    for (_k, schema) in schemas.iter_mut() {
        update_schema_refs(schema, old_names, new_name);
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
        Schema::Reference { reference } => {
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
        openapi: Some("3.0.0".to_string()),
        info: Some(Info {
            title: "Generated from JSON".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Schema generated from plain JSON input".to_string()),
        }),
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
        openapi: Some("3.0.0".to_string()),
        info: Some(Info {
            title,
            version: "1.0.0".to_string(),
            description: Some(description),
        }),
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
        Some(_) => None, // More complex additional properties schema - skip for now
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
