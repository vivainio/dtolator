pub mod zod;
pub mod typescript;

use anyhow::Result;
use crate::openapi::OpenApiSchema;

pub trait Generator {
    fn generate(&self, schema: &OpenApiSchema) -> Result<String>;
} 