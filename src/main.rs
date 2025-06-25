use clap::Parser;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;

mod openapi;
mod generators;

use openapi::OpenApiSchema;
use generators::{zod::ZodGenerator, typescript::TypeScriptGenerator, endpoints::EndpointsGenerator, angular::AngularGenerator, pydantic::PydanticGenerator, Generator};

#[derive(Parser)]
#[command(name = "dtolator")]
#[command(about = "Convert OpenAPI schema JSON files to Zod schema definitions or TypeScript interfaces")]
#[command(version = "0.1.0")]
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
    
    /// Generate API endpoint types from OpenAPI paths
    #[arg(short, long)]
    endpoints: bool,
    
    /// Pretty print the output
    #[arg(short, long)]
    pretty: bool,
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
                generate_angular_services(&schema, &output_dir, cli.pretty, cli.zod)?;
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
                let ts_final = if cli.pretty { format_output(&ts_output) } else { ts_output };
                
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
                let ts_final = if cli.pretty { format_output(&ts_output) } else { ts_output };
                
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
                let generator = AngularGenerator::new();
                generator.generate(&schema)?
            } else if cli.pydantic {
                let generator = PydanticGenerator::new();
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

fn generate_angular_services(schema: &OpenApiSchema, output_dir: &PathBuf, pretty: bool, with_zod: bool) -> Result<()> {
    let angular_generator = AngularGenerator::new().with_zod_validation(with_zod);
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
        
        // Generate TypeScript interfaces that import from schema.ts
        let ts_output = generate_typescript_with_imports(schema)?;
        let ts_final = if pretty { format_output(&ts_output) } else { ts_output };
        
        fs::write(&dto_path, ts_final)
            .with_context(|| format!("Failed to write dto.ts file: {}", dto_path.display()))?;
    } else {
        // Generate only TypeScript interfaces
        let ts_generator = TypeScriptGenerator::new();
        let dto_output = ts_generator.generate(schema)?;
        let dto_final = if pretty { format_output(&dto_output) } else { dto_output };
        
        fs::write(&dto_path, dto_final)
            .with_context(|| format!("Failed to write dto.ts file: {}", dto_path.display()))?;
    }
    
    // Generate subs-to-url utility function
    let subs_to_url_content = generate_subs_to_url_func();
    let subs_path = output_dir.join("subs-to-url.func.ts");
    fs::write(&subs_path, subs_to_url_content)
        .with_context(|| format!("Failed to write subs-to-url.func.ts file: {}", subs_path.display()))?;
    
    // Parse and split the Angular generator output into individual service files
    let mut files_generated = vec![dto_path.display().to_string(), subs_path.display().to_string()];
    
    // Split by the FILE markers and match content to files
    let mut current_content = String::new();
    let mut current_file = String::new();
    let mut in_file_section = false;
    
    for line in output.lines() {
        if line.starts_with("// FILE: ") {
            // If we were collecting content for a previous file, write it now
            if !current_file.is_empty() && !current_content.is_empty() {
                let service_path = output_dir.join(&current_file);
                let final_content = if pretty { format_output(&current_content) } else { current_content.clone() };
                fs::write(&service_path, final_content)
                    .with_context(|| format!("Failed to write {} file: {}", current_file, service_path.display()))?;
                files_generated.push(service_path.display().to_string());
            }
            
            // Start collecting for the new file
            current_file = line[9..].to_string(); // Remove "// FILE: "
            current_content.clear();
            in_file_section = true;
        } else if in_file_section {
            // If we haven't hit a FILE marker yet, this line belongs to the current file
            if !current_content.is_empty() {
                current_content.push('\n');
            }
            current_content.push_str(line);
        }
    }
    
    // Write the last file if there is one
    if !current_file.is_empty() && !current_content.is_empty() {
        let service_path = output_dir.join(&current_file);
        let final_content = if pretty { format_output(&current_content) } else { current_content };
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
                    let final_content = if pretty { format_output(first_service_content) } else { first_service_content.to_string() };
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

fn generate_typescript_with_imports(schema: &OpenApiSchema) -> Result<String> {
    let mut output = String::new();
    
    // Add header comment
    output.push_str("// Generated TypeScript interfaces from OpenAPI schema\n");
    output.push_str("// Do not modify this file manually\n\n");
    
    // Import all schema types from schema.ts
    if let Some(components) = &schema.components {
        if let Some(schemas) = &components.schemas {
            if !schemas.is_empty() {
                let type_names: Vec<String> = schemas.keys().cloned().collect();
                
                // Import both schema constants and types
                let mut imports = Vec::new();
                for name in &type_names {
                    imports.push(format!("{}Schema", name)); // Import schema constant
                    imports.push(format!("type {}", name));  // Import type
                }
                
                output.push_str(&format!("import {{ {} }} from './schema';\n\n", imports.join(", ")));
                
                // Re-export only the types for convenience
                for name in &type_names {
                    output.push_str(&format!("export type {{ {} }};\n", name));
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

import { environment } from '@env/environment';

export function subsToUrl(
  url: string,
  params?: { [key: string]: string | number | boolean | null | undefined },
  queryParams?: { [key: string]: string | number | boolean | null | undefined }
): string {
  if (params) {
    for (const key in params) {
      if (params.hasOwnProperty(key)) {
        const regex = new RegExp(':' + key + '($|/)');
        url = url.replace(regex, params[key] + '$1');
      }
    }
  }
  
  if (queryParams) {
    const qs = Object.keys(queryParams)
      .filter((key) => queryParams[key] !== null && queryParams[key] !== undefined)
      .map((key) => {
        const value = encodeURIComponent(queryParams[key]!);
        return `${key}=${value}`;
      })
      .join('&');
      
    if (qs.length > 0) {
      url += '?' + qs;
    }
  }

  const injectedConfig = (window as any).API_CONFIG;
  if (injectedConfig) {
    return injectedConfig.BACKEND_API_URL + url;
  }

  return environment.apiUrl + url;
}
"#.to_string()
}

fn format_output(output: &str) -> String {
    // Basic formatting - could be enhanced with a proper formatter
    output.to_string()
} 