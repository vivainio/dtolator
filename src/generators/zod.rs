use anyhow::Result;
use crate::openapi::{OpenApiSchema, Schema};
use crate::generators::Generator;

pub struct ZodGenerator {
    indent_level: usize,
}

impl ZodGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }
    
    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
    }
    
    fn with_indent<F>(&self, f: F) -> String 
    where F: FnOnce(&Self) -> String {
        let mut generator = self.clone();
        generator.indent_level += 1;
        f(&generator)
    }
    
    fn generate_schema(&self, name: &str, schema: &Schema) -> Result<String> {
        let mut output = String::new();
        
        let schema_name = format!("{}Schema", name);
        output.push_str(&format!("{}export const {} = ", self.indent(), schema_name));
        output.push_str(&self.schema_to_zod(schema)?);
        output.push_str(";\n\n");
        
        output.push_str(&format!("{}export type {} = z.infer<typeof {}>;\n\n", 
            self.indent(), name, schema_name));
        
        Ok(output)
    }
    
    fn schema_to_zod(&self, schema: &Schema) -> Result<String> {
        match schema {
            Schema::Reference { reference } => {
                let ref_name = reference.strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                Ok(format!("{}Schema", ref_name))
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
                let mut zod_schema = String::new();
                
                // Handle allOf, oneOf, anyOf
                if let Some(all_of_schemas) = all_of {
                    let schemas: Result<Vec<String>, _> = all_of_schemas.iter()
                        .map(|s| self.schema_to_zod(s))
                        .collect();
                    zod_schema = format!("z.intersection({})", 
                        schemas?.join(", z.intersection("));
                } else if let Some(one_of_schemas) = one_of {
                    let schemas: Result<Vec<String>, _> = one_of_schemas.iter()
                        .map(|s| self.schema_to_zod(s))
                        .collect();
                    zod_schema = format!("z.union([{}])", schemas?.join(", "));
                } else if let Some(any_of_schemas) = any_of {
                    let schemas: Result<Vec<String>, _> = any_of_schemas.iter()
                        .map(|s| self.schema_to_zod(s))
                        .collect();
                    zod_schema = format!("z.union([{}])", schemas?.join(", "));
                } else {
                    // Handle basic types
                    match schema_type.as_deref() {
                        Some("string") => {
                            if let Some(enum_vals) = enum_values {
                                let enum_strings: Vec<String> = enum_vals.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| format!("\"{}\"", s))
                                    .collect();
                                zod_schema = format!("z.enum([{}])", enum_strings.join(", "));
                            } else {
                                zod_schema = "z.string()".to_string();
                                
                                // Add format validations
                                if let Some(fmt) = format {
                                    match fmt.as_str() {
                                        "email" => zod_schema.push_str(".email()"),
                                        "uri" => zod_schema.push_str(".url()"),
                                        "uuid" => zod_schema.push_str(".uuid()"),
                                        "date" => zod_schema.push_str(".date()"),
                                        "date-time" => zod_schema.push_str(".datetime()"),
                                        _ => {}
                                    }
                                }
                                
                                // Add length constraints
                                if let Some(min_len) = min_length {
                                    zod_schema.push_str(&format!(".min({})", min_len));
                                }
                                if let Some(max_len) = max_length {
                                    zod_schema.push_str(&format!(".max({})", max_len));
                                }
                                
                                // Add pattern constraint
                                if let Some(pat) = pattern {
                                    zod_schema.push_str(&format!(".regex(new RegExp(\"{}\"))", pat));
                                }
                            }
                        }
                        Some("number") | Some("integer") => {
                            zod_schema = "z.number()".to_string();
                            
                            if let Some(min) = minimum {
                                zod_schema.push_str(&format!(".min({})", min));
                            }
                            if let Some(max) = maximum {
                                zod_schema.push_str(&format!(".max({})", max));
                            }
                            
                            if schema_type.as_deref() == Some("integer") {
                                zod_schema.push_str(".int()");
                            }
                        }
                        Some("boolean") => {
                            zod_schema = "z.boolean()".to_string();
                        }
                        Some("array") => {
                            if let Some(items_schema) = items {
                                let item_type = self.schema_to_zod(items_schema)?;
                                zod_schema = format!("z.array({})", item_type);
                            } else {
                                zod_schema = "z.array(z.unknown())".to_string();
                            }
                        }
                        Some("object") | None => {
                            if let Some(props) = properties {
                                let mut object_props = Vec::new();
                                for (prop_name, prop_schema) in props {
                                    let prop_zod = self.schema_to_zod(prop_schema)?;
                                    let is_required = required.as_ref()
                                        .map(|req| req.contains(prop_name))
                                        .unwrap_or(false);
                                    
                                    let prop_def = if is_required {
                                        format!("  {}: {}", prop_name, prop_zod)
                                    } else {
                                        format!("  {}: {}.optional()", prop_name, prop_zod)
                                    };
                                    object_props.push(prop_def);
                                }
                                
                                zod_schema = format!("z.object({{\n{}\n}})", 
                                    object_props.join(",\n"));
                            } else {
                                zod_schema = "z.object({})".to_string();
                            }
                        }
                        _ => {
                            zod_schema = "z.unknown()".to_string();
                        }
                    }
                }
                
                // Apply nullable if needed
                if nullable.unwrap_or(false) {
                    zod_schema = format!("{}.nullable()", zod_schema);
                }
                
                Ok(zod_schema)
            }
        }
    }
}

impl Generator for ZodGenerator {
    fn generate(&self, schema: &OpenApiSchema) -> Result<String> {
        let mut output = String::new();
        
        output.push_str("import { z } from 'zod';\n\n");
        
        if let Some(components) = &schema.components {
            if let Some(schemas) = &components.schemas {
                for (name, schema_def) in schemas {
                    output.push_str(&self.generate_schema(name, schema_def)?);
                }
            }
        }
        
        Ok(output)
    }
}

impl Clone for ZodGenerator {
    fn clone(&self) -> Self {
        Self {
            indent_level: self.indent_level,
        }
    }
} 