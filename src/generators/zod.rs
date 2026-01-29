use crate::generators::Generator;
use crate::generators::common;
use crate::generators::import_generator::ImportGenerator;
use crate::generators::zod_schema::{NumberConstraints, StringConstraints, ZodValue};
use crate::openapi::{OpenApiSchema, Schema};
use anyhow::Result;

pub struct ZodGenerator {
    indent_level: usize,
}

impl Default for ZodGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl ZodGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }

    fn generate_schema(&self, name: &str, schema: &Schema) -> Result<String> {
        let mut output = String::new();

        let schema_name = format!("{name}Schema");
        output.push_str(&format!("{}export const {} = ", self.indent(), schema_name));
        let zod_value = self.schema_to_zod(schema)?;
        output.push_str(&format!("{zod_value}"));
        output.push_str(";\n\n");

        output.push_str(&format!(
            "{}export type {} = z.infer<typeof {}>;\n\n",
            self.indent(),
            name,
            schema_name
        ));

        Ok(output)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn schema_to_zod(&self, schema: &Schema) -> Result<ZodValue> {
        match schema {
            Schema::Reference { reference } => {
                let ref_name = reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                Ok(ZodValue::Reference(ref_name.to_string()))
            }
            Schema::Object {
                schema_type,
                properties,
                required,
                items,
                enum_values,
                format,
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
                let value = // Handle allOf, oneOf, anyOf
                if let Some(all_of_schemas) = all_of {
                    let schemas: Result<Vec<ZodValue>> = all_of_schemas
                        .iter()
                        .map(|s| self.schema_to_zod(s))
                        .collect();
                    ZodValue::Intersection(schemas?)
                } else if let Some(one_of_schemas) = one_of {
                    let schemas: Result<Vec<ZodValue>> = one_of_schemas
                        .iter()
                        .map(|s| self.schema_to_zod(s))
                        .collect();
                    ZodValue::Union(schemas?)
                } else if let Some(any_of_schemas) = any_of {
                    let schemas: Result<Vec<ZodValue>> = any_of_schemas
                        .iter()
                        .map(|s| self.schema_to_zod(s))
                        .collect();
                    ZodValue::Union(schemas?)
                } else {
                    // Handle basic types
                    match schema_type.as_deref() {
                        Some("string") => {
                            if let Some(enum_vals) = enum_values {
                                let enum_strings: Vec<String> = enum_vals
                                    .iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect();
                                ZodValue::Enum(enum_strings)
                            } else if format.as_deref() == Some("binary") {
                                // For multipart file uploads, use z.instanceof(File)
                                ZodValue::File
                            } else {
                                ZodValue::String(StringConstraints {
                                    format: format.clone(),
                                    min_length: *min_length,
                                    max_length: *max_length,
                                    pattern: pattern.clone(),
                                })
                            }
                        }
                        Some("number") | Some("integer") => ZodValue::Number(NumberConstraints {
                            is_integer: schema_type.as_deref() == Some("integer"),
                            minimum: *minimum,
                            maximum: *maximum,
                        }),
                        Some("boolean") => ZodValue::Boolean,
                        Some("array") => {
                            if let Some(items_schema) = items {
                                let item_type = self.schema_to_zod(items_schema)?;
                                ZodValue::Array(Box::new(item_type))
                            } else {
                                ZodValue::Array(Box::new(ZodValue::Unknown))
                            }
                        }
                        Some("object") | None => {
                            if let Some(props) = properties {
                                let mut object_props = Vec::new();
                                for (prop_name, prop_schema) in props {
                                    let prop_zod = self.schema_to_zod(prop_schema)?;
                                    let is_required = required
                                        .as_ref()
                                        .map(|req| req.contains(prop_name))
                                        .unwrap_or(false);
                                    object_props.push((prop_name.clone(), prop_zod, is_required));
                                }
                                ZodValue::Object(object_props)
                            } else {
                                ZodValue::Object(Vec::new())
                            }
                        }
                        _ => ZodValue::Unknown,
                    }
                };

                // Apply nullable if needed
                let result = if nullable.unwrap_or(false) {
                    ZodValue::Nullable(Box::new(value))
                } else {
                    value
                };

                Ok(result)
            }
        }
    }
}

impl Generator for ZodGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!("// Generated by {command}\n"));
        output.push_str("// Do not modify manually\n\n");

        let mut import_gen = ImportGenerator::new();
        import_gen.add_import("zod", "z", false);
        output.push_str(&import_gen.generate());

        if let Some(components) = &schema.components
            && let Some(schemas) = &components.schemas
            && !schemas.is_empty()
        {
            // Sort schemas topologically to handle dependencies
            let sorted_names = common::topological_sort(schemas)?;

            for name in sorted_names {
                if let Some(schema_def) = schemas.get(&name) {
                    let zod_schema = self.generate_schema(&name, schema_def)?;
                    output.push_str(&zod_schema);
                }
            }
        }

        // Remove trailing blank lines
        Ok(output.trim_end().to_string() + "\n")
    }
}

impl Clone for ZodGenerator {
    fn clone(&self) -> Self {
        Self {
            indent_level: self.indent_level,
        }
    }
}
