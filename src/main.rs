use clap::Parser;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::fs;

mod openapi;
mod generators;

use openapi::OpenApiSchema;
use generators::{zod::ZodGenerator, typescript::TypeScriptGenerator, endpoints::EndpointsGenerator, Generator};

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
            
            if cli.zod {
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

fn format_output(output: &str) -> String {
    // Basic formatting - could be enhanced with a proper formatter
    output.to_string()
} 