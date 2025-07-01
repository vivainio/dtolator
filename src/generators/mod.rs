pub mod angular;
pub mod dotnet;
pub mod endpoints;
pub mod json_schema;
pub mod pydantic;
pub mod python_dict;
pub mod typescript;
pub mod zod;

use crate::openapi::OpenApiSchema;
use anyhow::Result;

pub trait Generator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String>;
}
