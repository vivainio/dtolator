use clap::Parser;
use anyhow::{Context, Result};
use std::path::PathBuf;

mod openapi;
mod generators;

use openapi::OpenApiSchema;
use generators::{zod::ZodGenerator, typescript::TypeScriptGenerator, Generator};

#[derive(Parser)]
#[command(name = "dtolator")]
#[command(about = "Convert OpenAPI schema JSON files to Zod schema definitions or TypeScript interfaces")]
#[command(version = "0.1.0")]
struct Cli {
    /// Input OpenAPI schema JSON file
    #[arg(short, long)]
    input: PathBuf,
    
    /// Output file path
    #[arg(short, long)]
    output: Option<PathBuf>,
    
    /// Generate TypeScript interfaces instead of Zod schemas
    #[arg(short, long)]
    typescript: bool,
    
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
    
    // Generate output based on the selected format
    let output = if cli.typescript {
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
    
    // Write to output file or stdout
    match cli.output {
        Some(output_path) => {
            std::fs::write(&output_path, final_output)
                .with_context(|| format!("Failed to write to output file: {}", output_path.display()))?;
            println!("Output written to: {}", output_path.display());
        }
        None => {
            println!("{}", final_output);
        }
    }
    
    Ok(())
}

fn format_output(output: &str) -> String {
    // Basic formatting - could be enhanced with a proper formatter
    output.to_string()
} 