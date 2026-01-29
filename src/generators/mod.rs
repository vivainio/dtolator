pub mod angular;
pub mod common;
pub mod dotnet;
pub mod endpoints;
pub mod import_generator;
pub mod json_schema;
pub mod pydantic;
pub mod python_dict;
pub mod rust_serde;
pub mod typescript;
pub mod zod;
pub mod zod_schema;

use crate::openapi::OpenApiSchema;
use anyhow::Result;

pub trait Generator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String>;
}
