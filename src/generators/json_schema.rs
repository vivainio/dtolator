use crate::generators::Generator;
use crate::generators::common;
use crate::openapi::{OpenApiSchema, Schema, is_schema_nullable, schema_type_str};
use anyhow::Result;
use serde_json::{Value, json};

pub struct JsonSchemaGenerator {
    indent_level: usize,
}

impl Default for JsonSchemaGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonSchemaGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn schema_to_json_schema(&self, schema: &Schema) -> Result<Value> {
        match schema {
            Schema::Reference { reference } => {
                let type_name = reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                Ok(json!({
                    "$ref": format!("#/$defs/{}", type_name)
                }))
            }
            Schema::Object {
                schema_type,
                properties,
                required,
                items,
                enum_values,
                format,
                description,
                nullable,
                all_of,
                one_of,
                any_of,
                minimum,
                maximum,
                min_length,
                max_length,
                pattern,
                ..
            } => {
                let mut json_schema = serde_json::Map::new();

                // Handle composition schemas
                if let Some(all_of_schemas) = all_of {
                    let mut all_of_json = Vec::new();
                    for s in all_of_schemas {
                        all_of_json.push(self.schema_to_json_schema(s)?);
                    }
                    json_schema.insert("allOf".to_string(), Value::Array(all_of_json));
                    if let Some(desc) = description {
                        json_schema.insert("description".to_string(), Value::String(desc.clone()));
                    }
                    return Ok(Value::Object(json_schema));
                }

                if let Some(one_of_schemas) = one_of {
                    let mut one_of_json = Vec::new();
                    for s in one_of_schemas {
                        one_of_json.push(self.schema_to_json_schema(s)?);
                    }
                    json_schema.insert("oneOf".to_string(), Value::Array(one_of_json));
                    if let Some(desc) = description {
                        json_schema.insert("description".to_string(), Value::String(desc.clone()));
                    }
                    return Ok(Value::Object(json_schema));
                }

                if let Some(any_of_schemas) = any_of {
                    let mut any_of_json = Vec::new();
                    for s in any_of_schemas {
                        any_of_json.push(self.schema_to_json_schema(s)?);
                    }
                    json_schema.insert("anyOf".to_string(), Value::Array(any_of_json));
                    if let Some(desc) = description {
                        json_schema.insert("description".to_string(), Value::String(desc.clone()));
                    }
                    return Ok(Value::Object(json_schema));
                }

                // Handle enum values
                if let Some(enum_vals) = enum_values {
                    json_schema.insert("enum".to_string(), Value::Array(enum_vals.clone()));
                    if let Some(desc) = description {
                        json_schema.insert("description".to_string(), Value::String(desc.clone()));
                    }
                    return Ok(Value::Object(json_schema));
                }

                // Handle type
                if let Some(type_str) = schema_type_str(schema_type) {
                    json_schema.insert("type".to_string(), Value::String(type_str.to_string()));
                }

                // Handle object properties
                if let Some(props) = properties {
                    let mut properties_json = serde_json::Map::new();
                    for (key, prop_schema) in props {
                        properties_json
                            .insert(key.clone(), self.schema_to_json_schema(prop_schema)?);
                    }
                    json_schema.insert("properties".to_string(), Value::Object(properties_json));

                    if let Some(req) = required
                        && !req.is_empty()
                    {
                        json_schema.insert(
                            "required".to_string(),
                            Value::Array(req.iter().map(|r| Value::String(r.clone())).collect()),
                        );
                    }

                    json_schema.insert("additionalProperties".to_string(), Value::Bool(false));
                }

                // Handle array items
                if let Some(items_schema) = items {
                    json_schema.insert(
                        "items".to_string(),
                        self.schema_to_json_schema(items_schema)?,
                    );
                }

                // Handle format
                if let Some(fmt) = format {
                    json_schema.insert("format".to_string(), Value::String(fmt.clone()));
                }

                // Handle description
                if let Some(desc) = description {
                    json_schema.insert("description".to_string(), Value::String(desc.clone()));
                }

                // Handle nullable
                if is_schema_nullable(nullable, schema_type)
                    && let Some(existing_type) = json_schema.get("type")
                {
                    json_schema.insert("type".to_string(), json!([existing_type.clone(), "null"]));
                }

                // Handle number constraints
                if let Some(min) = minimum {
                    json_schema.insert("minimum".to_string(), json!(min));
                }
                if let Some(max) = maximum {
                    json_schema.insert("maximum".to_string(), json!(max));
                }

                // Handle string constraints
                if let Some(min_len) = min_length {
                    json_schema.insert("minLength".to_string(), json!(min_len));
                }
                if let Some(max_len) = max_length {
                    json_schema.insert("maxLength".to_string(), json!(max_len));
                }
                if let Some(pat) = pattern {
                    json_schema.insert("pattern".to_string(), Value::String(pat.clone()));
                }

                Ok(Value::Object(json_schema))
            }
        }
    }

    fn generate_full_schema(&self, schema: &OpenApiSchema) -> Result<Value> {
        let mut json_schema = serde_json::Map::new();

        // JSON Schema metadata
        json_schema.insert(
            "$schema".to_string(),
            Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
        );

        // Add title and description from OpenAPI info
        if let Some(info) = &schema.info {
            json_schema.insert("title".to_string(), Value::String(info.title.clone()));
            if let Some(desc) = &info.description {
                json_schema.insert("description".to_string(), Value::String(desc.clone()));
            }
        }

        // Process schemas
        if let Some(components) = &schema.components
            && let Some(schemas) = &components.schemas
            && !schemas.is_empty()
        {
            // Create $defs section for reusable schemas
            let mut defs = serde_json::Map::new();

            // Sort schemas by dependencies
            let sorted_names = common::topological_sort(schemas)?;

            for name in sorted_names {
                if let Some(schema_def) = schemas.get(&name) {
                    defs.insert(name.clone(), self.schema_to_json_schema(schema_def)?);
                }
            }

            json_schema.insert("$defs".to_string(), Value::Object(defs));

            // If there's a root schema, make it the main schema
            if let Some(root_schema) = schemas.get("Root") {
                let root_json = self.schema_to_json_schema(root_schema)?;
                if let Value::Object(root_obj) = root_json {
                    for (key, value) in root_obj {
                        if key != "$ref" {
                            json_schema.insert(key, value);
                        }
                    }
                }
            } else {
                // If no root schema, reference the first schema
                if let Some(first_name) = schemas.keys().next() {
                    json_schema.insert(
                        "$ref".to_string(),
                        Value::String(format!("#/$defs/{first_name}")),
                    );
                }
            }
        }

        Ok(Value::Object(json_schema))
    }
}

impl Generator for JsonSchemaGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, _command: &str) -> Result<String> {
        let json_schema = self.generate_full_schema(schema)?;
        // JSON doesn't support comments, so we output valid JSON only
        let output = serde_json::to_string_pretty(&json_schema)?;
        Ok(output)
    }
}

impl Clone for JsonSchemaGenerator {
    fn clone(&self) -> Self {
        Self {
            indent_level: self.indent_level,
        }
    }
}
