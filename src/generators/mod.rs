pub mod zod;
pub mod typescript;
pub mod endpoints;
pub mod angular;
pub mod pydantic;
pub mod python_dict;
pub mod dotnet;
pub mod json_schema;

use anyhow::Result;
use crate::openapi::OpenApiSchema;

pub trait Generator {
    fn generate(&self, schema: &OpenApiSchema) -> Result<String>;
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String>;
} 