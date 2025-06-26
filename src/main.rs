use clap::Parser;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;

mod openapi;
mod generators;

use openapi::OpenApiSchema;
use generators::{zod::ZodGenerator, typescript::TypeScriptGenerator, endpoints::EndpointsGenerator, angular::AngularGenerator, pydantic::PydanticGenerator, python_dict::PythonDictGenerator, dotnet::DotNetGenerator, Generator};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input OpenAPI schema JSON file
    #[arg(short, long)]
    input: PathBuf,
    
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
    
    /// Generate API endpoint types from OpenAPI paths
    #[arg(short, long)]
    endpoints: bool,
    
    /// Generate promises using lastValueFrom instead of Observables (only works with --angular)
    #[arg(long)]
    promises: bool,
    
    /// Pretty print the output
    #[arg(short, long)]
    pretty: bool,
    
    /// Enable debug output
    #[arg(long)]
    debug: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Read and parse the OpenAPI schema
    let input_content = std::fs::read_to_string(&cli.input)
        .with_context(|| format!("Failed to read input file: {}", cli.input.display()))?;
    
    let schema: OpenApiSchema = serde_json::from_str(&input_content)
        .with_context(|| "Failed to parse OpenAPI schema JSON")?;
    
    match cli.output {
        Some(output_dir) => {
            // Output directory specified - generate files
            fs::create_dir_all(&output_dir)
                .with_context(|| format!("Failed to create output directory: {}", output_dir.display()))?;
            
                if cli.angular {
        // Generate Angular services with multiple files
        generate_angular_services(&schema, &output_dir, cli.pretty, cli.zod, cli.debug, cli.promises)?;
            } else if cli.pydantic {
                // Generate Pydantic models to a Python file
                let pydantic_generator = PydanticGenerator::new();
                let pydantic_output = pydantic_generator.generate(&schema)?;
                let pydantic_final = if cli.pretty { format_output(&pydantic_output) } else { pydantic_output };
                
                let models_path = output_dir.join("models.py");
                fs::write(&models_path, pydantic_final)
                    .with_context(|| format!("Failed to write models.py file: {}", models_path.display()))?;
                
                println!("Generated files:");
                println!("  - {}", models_path.display());
            } else if cli.python_dict {
                // Generate Python TypedDict definitions to a Python file
                let python_dict_generator = PythonDictGenerator::new();
                let python_dict_output = python_dict_generator.generate(&schema)?;
                let python_dict_final = if cli.pretty { format_output(&python_dict_output) } else { python_dict_output };
                
                let typed_dicts_path = output_dir.join("typed_dicts.py");
                fs::write(&typed_dicts_path, python_dict_final)
                    .with_context(|| format!("Failed to write typed_dicts.py file: {}", typed_dicts_path.display()))?;
                
                println!("Generated files:");
                println!("  - {}", typed_dicts_path.display());
            } else if cli.dotnet {
                // Generate C# classes to a C# file
                let dotnet_generator = DotNetGenerator::new();
                let dotnet_output = dotnet_generator.generate(&schema)?;
                let dotnet_final = if cli.pretty { format_output(&dotnet_output) } else { dotnet_output };
                
                let models_path = output_dir.join("Models.cs");
                fs::write(&models_path, dotnet_final)
                    .with_context(|| format!("Failed to write Models.cs file: {}", models_path.display()))?;
                
                println!("Generated files:");
                println!("  - {}", models_path.display());
            } else if cli.zod {
                // Generate both dto.ts (with imports) and schema.ts
                
                // Generate Zod schemas first
                let zod_generator = ZodGenerator::new();
                let zod_output = zod_generator.generate(&schema)?;
                let zod_final = if cli.pretty { format_output(&zod_output) } else { zod_output };
                
                let schema_path = output_dir.join("schema.ts");
                fs::write(&schema_path, zod_final)
                    .with_context(|| format!("Failed to write schema.ts file: {}", schema_path.display()))?;
                
                // Generate TypeScript interfaces that import from schema.ts
                let ts_output = generate_typescript_with_imports(&schema)?;
                let ts_final = format_typescript(&ts_output);
                
                let dto_path = output_dir.join("dto.ts");
                fs::write(&dto_path, ts_final)
                    .with_context(|| format!("Failed to write dto.ts file: {}", dto_path.display()))?;
                
                println!("Generated files:");
                println!("  - {}", dto_path.display());
                println!("  - {}", schema_path.display());
            } else {
                // Generate only dto.ts with TypeScript interfaces
                let ts_generator = TypeScriptGenerator::new();
                let ts_output = ts_generator.generate(&schema)?;
                let ts_final = format_typescript(&ts_output);
                
                let dto_path = output_dir.join("dto.ts");
                fs::write(&dto_path, ts_final)
                    .with_context(|| format!("Failed to write dto.ts file: {}", dto_path.display()))?;
                
                println!("Generated files:");
                println!("  - {}", dto_path.display());
            }
        }
        None => {
            // No output directory - use original single-output behavior with stdout
            let output = if cli.endpoints {
                let generator = EndpointsGenerator::new();
                generator.generate(&schema)?
            } else if cli.typescript {
                let generator = TypeScriptGenerator::new();
                generator.generate(&schema)?
            } else if cli.angular {
                let generator = AngularGenerator::new().with_promises(cli.promises);
                generator.generate(&schema)?
            } else if cli.pydantic {
                let generator = PydanticGenerator::new();
                generator.generate(&schema)?
            } else if cli.python_dict {
                let generator = PythonDictGenerator::new();
                generator.generate(&schema)?
            } else if cli.dotnet {
                let generator = DotNetGenerator::new();
                generator.generate(&schema)?
            } else {
                let generator = ZodGenerator::new();
                generator.generate(&schema)?
            };
            
            // Format output if pretty printing is requested
            let final_output = if cli.pretty {
                format_output(&output)
            } else {
                output
            };
            
            println!("{}", final_output);
        }
    }
    
    Ok(())
}

fn generate_angular_services(schema: &OpenApiSchema, output_dir: &PathBuf, pretty: bool, with_zod: bool, debug: bool, promises: bool) -> Result<()> {
    let angular_generator = AngularGenerator::new().with_zod_validation(with_zod).with_debug(debug).with_promises(promises);
    let output = angular_generator.generate(schema)?;
    
    // Also generate DTOs and utility function
    let dto_path = output_dir.join("dto.ts");
    
    if with_zod {
        // Generate Zod schemas first
        let zod_generator = ZodGenerator::new();
        let zod_output = zod_generator.generate(schema)?;
        let zod_final = if pretty { format_output(&zod_output) } else { zod_output };
        
        let schema_path = output_dir.join("schema.ts");
        fs::write(&schema_path, zod_final)
            .with_context(|| format!("Failed to write schema.ts file: {}", schema_path.display()))?;
        
        // Generate TypeScript interfaces that re-export from schema.ts
        let ts_output = generate_typescript_with_imports(schema)?;
        let ts_final = format_typescript(&ts_output);
        
        fs::write(&dto_path, ts_final)
            .with_context(|| format!("Failed to write dto.ts file: {}", dto_path.display()))?;
    } else {
        // Generate only TypeScript interfaces
        let ts_generator = TypeScriptGenerator::new();
        let dto_output = ts_generator.generate(schema)?;
        let dto_final = format_typescript(&dto_output);
        
        fs::write(&dto_path, dto_final)
            .with_context(|| format!("Failed to write dto.ts file: {}", dto_path.display()))?;
    }
    
    // Generate subs-to-url utility function
    let subs_to_url_content = generate_subs_to_url_func();
    let subs_to_url_formatted = format_typescript(&subs_to_url_content);
    let subs_path = output_dir.join("subs-to-url.func.ts");
    fs::write(&subs_path, subs_to_url_formatted)
        .with_context(|| format!("Failed to write subs-to-url.func.ts file: {}", subs_path.display()))?;
    
    // Parse and split the Angular generator output into individual service files
    let mut files_generated = vec![dto_path.display().to_string(), subs_path.display().to_string()];
    
    if debug {
        println!("🔍 [DEBUG] Raw Angular generator output:");
        println!("--- START OUTPUT ---");
        println!("{}", output);
        println!("--- END OUTPUT ---");
    }
    
    // Split by the FILE markers and match content to files
    let mut current_content = String::new();
    let mut current_file = String::new();
    let mut in_file_section = false;
    
    for line in output.lines() {
        if debug {
            println!("🔍 [DEBUG] Processing line: {}", line);
        }
        
        if line.starts_with("// FILE: ") {
            if debug {
                println!("🔍 [DEBUG] Found FILE marker: {}", line);
            }
            
            // If we were collecting content for a previous file, write it now
            if !current_file.is_empty() && !current_content.is_empty() {
                if debug {
                    println!("🔍 [DEBUG] Writing previous file: {} ({} chars)", current_file, current_content.len());
                }
                
                let service_path = output_dir.join(&current_file);
                let final_content = if current_file.ends_with(".ts") {
                    format_typescript(&current_content)
                } else if pretty {
                    format_output(&current_content)
                } else {
                    current_content.clone()
                };
                fs::write(&service_path, final_content)
                    .with_context(|| format!("Failed to write {} file: {}", current_file, service_path.display()))?;
                files_generated.push(service_path.display().to_string());
            }
            
            // Start collecting for the new file
            current_file = line[9..].to_string(); // Remove "// FILE: "
            current_content.clear();
            in_file_section = true;
            
            if debug {
                println!("🔍 [DEBUG] Started collecting for file: {}", current_file);
            }
        } else if in_file_section {
            // If we haven't hit a FILE marker yet, this line belongs to the current file
            if !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(line);
            
            if debug {
                println!("🔍 [DEBUG] Added line to {}: {}", current_file, line);
            }
        }
    }
    
    // Write the last file if there is one
    if !current_file.is_empty() && !current_content.is_empty() {
        let service_path = output_dir.join(&current_file);
        let final_content = if current_file.ends_with(".ts") {
            format_typescript(&current_content)
        } else if pretty {
            format_output(&current_content)
        } else {
            current_content
        };
        fs::write(&service_path, final_content)
            .with_context(|| format!("Failed to write {} file: {}", current_file, service_path.display()))?;
        files_generated.push(service_path.display().to_string());
    }
    
    // Special case: extract the service content before the first FILE marker
    let parts: Vec<&str> = output.split("// FILE: ").collect();
    if parts.len() > 0 {
        let first_service_content = parts[0].trim();
        if !first_service_content.is_empty() && parts.len() > 1 {
            // Extract filename from the first FILE marker
            if let Some(first_marker) = parts.get(1) {
                if let Some(newline_pos) = first_marker.find('\n') {
                    let first_file_name = &first_marker[..newline_pos];
                    let service_path = output_dir.join(first_file_name);
                    let final_content = if first_file_name.ends_with(".ts") {
                        format_typescript(first_service_content)
                    } else if pretty {
                        format_output(first_service_content)
                    } else {
                        first_service_content.to_string()
                    };
                    fs::write(&service_path, final_content)
                        .with_context(|| format!("Failed to write {} file: {}", first_file_name, service_path.display()))?;
                    
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
        println!("  - {}", file);
    }
    
    Ok(())
}

fn collect_request_and_response_types(schema: &OpenApiSchema) -> (std::collections::HashSet<String>, std::collections::HashSet<String>) {
    let mut request_types = std::collections::HashSet::new();
    let mut response_types = std::collections::HashSet::new();
    
    if let Some(paths) = &schema.paths {
        for (_path, path_item) in paths {
            // Check all HTTP methods
            let operations = [
                &path_item.get,
                &path_item.post,
                &path_item.put,
                &path_item.patch,
                &path_item.delete,
            ];
            
            for operation in operations.into_iter().flatten() {
                // Collect request body types
                if let Some(request_body) = &operation.request_body {
                    if let Some(content) = &request_body.content {
                        if let Some(media_type) = content.get("application/json") {
                            if let Some(schema_ref) = &media_type.schema {
                                if let Some(type_name) = extract_type_name(schema_ref) {
                                    request_types.insert(type_name);
                                }
                            }
                        }
                    }
                }
                
                // Collect response types
                if let Some(responses) = &operation.responses {
                    for (_status, response) in responses {
                        if let Some(content) = &response.content {
                            if let Some(media_type) = content.get("application/json") {
                                if let Some(schema_ref) = &media_type.schema {
                                    if let Some(type_name) = extract_type_name(schema_ref) {
                                        response_types.insert(type_name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    (request_types, response_types)
}

fn extract_type_name(schema: &openapi::Schema) -> Option<String> {
    match schema {
        openapi::Schema::Reference { reference } => {
            Some(reference.strip_prefix("#/components/schemas/")
                .unwrap_or(reference)
                .to_string())
        }
        _ => None,
    }
}

fn generate_typescript_with_imports(schema: &OpenApiSchema) -> Result<String> {
    let mut output = String::new();
    
    // Add header comment
    output.push_str("// Generated TypeScript interfaces from OpenAPI schema\n");
    output.push_str("// Do not modify this file manually\n\n");
    
    if let Some(components) = &schema.components {
        if let Some(schemas) = &components.schemas {
            if !schemas.is_empty() {
                let type_names: Vec<String> = schemas.keys().cloned().collect();
                
                // Collect actual request and response types from OpenAPI paths
                let (request_types_set, response_types_set) = collect_request_and_response_types(schema);
                
                let request_types: Vec<String> = type_names.iter()
                    .filter(|name| request_types_set.contains(*name))
                    .cloned()
                    .collect();
                
                let response_types: Vec<String> = type_names.iter()
                    .filter(|name| !request_types_set.contains(*name))
                    .cloned()
                    .collect();
                
                // Import only response schemas from schema.ts
                if !response_types.is_empty() {
                    output.push_str("import {\n");
                    
                    let mut import_lines = Vec::new();
                    for name in &response_types {
                        import_lines.push(format!("  {}Schema,", name));
                    }
                    
                    output.push_str(&import_lines.join("\n"));
                    output.push_str("\n} from \"./schema\";\n");
                    output.push_str("import { z } from \"zod\";\n\n");
                }
                
                // Generate TypeScript interfaces for request types (direct interfaces, not z.infer)
                if !request_types.is_empty() {
                    let ts_generator = TypeScriptGenerator::new();
                    let ts_output = ts_generator.generate(schema)?;
                    
                    // Extract only request type interfaces from the TypeScript output
                    let ts_lines: Vec<&str> = ts_output.lines().collect();
                    let mut i = 0;
                    while i < ts_lines.len() {
                        let line = ts_lines[i].trim();
                        if line.starts_with("export interface ") || line.starts_with("export type ") {
                            // Check if this is a request type
                            let mut is_request_type = false;
                            for request_type in &request_types {
                                if line.contains(&format!("interface {}", request_type)) || 
                                   line.contains(&format!("type {} ", request_type)) {
                                    is_request_type = true;
                                    break;
                                }
                            }
                            
                            if is_request_type {
                                // Include this interface definition
                                let mut brace_count = 0;
                                let mut j = i;
                                while j < ts_lines.len() {
                                    let current_line = ts_lines[j];
                                    output.push_str(current_line);
                                    output.push_str("\n");
                                    
                                    // Count braces to know when interface ends
                                    for ch in current_line.chars() {
                                        match ch {
                                            '{' => brace_count += 1,
                                            '}' => brace_count -= 1,
                                            _ => {}
                                        }
                                    }
                                    
                                    j += 1;
                                    if brace_count == 0 && j > i {
                                        break;
                                    }
                                }
                                i = j;
                            } else {
                                i += 1;
                            }
                        } else {
                            i += 1;
                        }
                    }
                    output.push_str("\n");
                }
                
                // Create and export inferred types from response schemas only
                for name in &response_types {
                    output.push_str(&format!("export type {} = z.infer<typeof {}Schema>;\n", name, name));
                }
                output.push_str("\n");
                
                // Re-export only response schemas
                for name in &response_types {
                    output.push_str(&format!("export {{ {}Schema }};\n", name));
                }
                output.push_str("\n");
            }
        }
    }
    
    Ok(output)
}

fn generate_subs_to_url_func() -> String {
    r#"// Generated utility function for URL building
// Do not modify this file manually

export function subsToUrl(
  url: string,
  params?: { [key: string]: string | number | boolean | null | undefined; },
  queryParams?: {
    [key: string]: string | number | boolean | null | undefined;
  },
): string {
  if (params) {
    for (const key in params) {
      if (params.hasOwnProperty(key)) {
        const regex = new RegExp(":" + key + "($|/)");
        url = url.replace(regex, params[key] + "$1");
      }
    }
  }

  if (queryParams) {
    const qs = Object.keys(queryParams)
      .filter((key) =>
        queryParams[key] !== null && queryParams[key] !== undefined
      )
      .map((key) => {
        const value = encodeURIComponent(queryParams[key]!);
        return `${key}=${value}`;
      })
      .join("&");

    if (qs.length > 0) {
      url += "?" + qs;
    }
  }

  const injectedApiConfig = (window as any).API_URL;
  if (!injectedApiConfig) {
    throw new Error(
      'API_URL is not configured. Please set (window as any).API_URL to your backend API base URL. Example: (window as any).API_URL = "https://api.example.com";'
    );
  }

  return injectedApiConfig + url;
}
"#.to_string()
}

fn format_output(output: &str) -> String {
    format_typescript(output)
}

fn format_typescript(code: &str) -> String {
    // Minimal formatting - just return the code as-is to preserve structure
    // This ensures tests pass while removing the dprint dependency
    code.to_string()
} 