use anyhow::Result;
use crate::openapi::{OpenApiSchema, Schema};
use crate::generators::Generator;

pub struct TypeScriptGenerator {
    indent_level: usize,
}

impl TypeScriptGenerator {
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
    
    fn generate_interface(&self, name: &str, schema: &Schema) -> Result<String> {
        let mut output = String::new();
        
        match schema {
            Schema::Object { 
                schema_type, 
                properties, 
                required,
                enum_values,
                all_of,
                one_of,
                any_of,
                ..
            } => {
                // Handle enum types
                if let Some(enum_vals) = enum_values {
                    output.push_str(&format!("export type {} =\n", name));
                    let enum_strings: Vec<String> = enum_vals.iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("  | \"{}\"", s))
                        .collect();
                    output.push_str(&enum_strings.join("\n"));
                    output.push_str(";\n\n");
                    return Ok(output);
                }
                
                // Handle composition types
                if let Some(all_of_schemas) = all_of {
                    output.push_str(&format!("export type {} =\n", name));
                    let types: Result<Vec<String>, _> = all_of_schemas.iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    let type_list = types?;
                    for (i, type_str) in type_list.iter().enumerate() {
                        if i == 0 {
                            output.push_str(&format!("  {}", type_str));
                        } else {
                            output.push_str(&format!("\n  & {}", type_str));
                        }
                    }
                    output.push_str(";\n\n");
                    return Ok(output);
                } else if let Some(one_of_schemas) = one_of {
                    output.push_str(&format!("export type {} =\n", name));
                    let types: Result<Vec<String>, _> = one_of_schemas.iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    let type_list = types?;
                    for (i, type_str) in type_list.iter().enumerate() {
                        if i == 0 {
                            output.push_str(&format!("  {}", type_str));
                        } else {
                            output.push_str(&format!("\n  | {}", type_str));
                        }
                    }
                    output.push_str(";\n\n");
                    return Ok(output);
                } else if let Some(any_of_schemas) = any_of {
                    output.push_str(&format!("export type {} =\n", name));
                    let types: Result<Vec<String>, _> = any_of_schemas.iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    let type_list = types?;
                    for (i, type_str) in type_list.iter().enumerate() {
                        if i == 0 {
                            output.push_str(&format!("  {}", type_str));
                        } else {
                            output.push_str(&format!("\n  | {}", type_str));
                        }
                    }
                    output.push_str(";\n\n");
                    return Ok(output);
                }
                
                // Handle object types
                if schema_type.as_deref() == Some("object") || properties.is_some() {
                    output.push_str(&format!("export interface {} {{\n", name));
                    
                    if let Some(props) = properties {
                        for (prop_name, prop_schema) in props {
                            let prop_type = self.schema_to_typescript(prop_schema)?;
                            let is_required = required.as_ref()
                                .map(|req| req.contains(prop_name))
                                .unwrap_or(false);
                            
                            let optional_marker = if is_required { "" } else { "?" };
                            output.push_str(&format!("  {}{}: {};\n", 
                                prop_name, optional_marker, prop_type));
                        }
                    }
                    
                    output.push_str("}\n\n");
                } else {
                    // Handle primitive type aliases
                    output.push_str(&format!("export type {} = {};\n\n", 
                        name, self.schema_to_typescript(schema)?));
                }
            }
            Schema::Reference { .. } => {
                // For references, create a type alias
                output.push_str(&format!("export type {} = {};\n\n", 
                    name, self.schema_to_typescript(schema)?));
            }
        }
        
        Ok(output)
    }
    
    fn schema_to_typescript(&self, schema: &Schema) -> Result<String> {
        match schema {
            Schema::Reference { reference } => {
                let ref_name = reference.strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                Ok(ref_name.to_string())
            }
            Schema::Object { 
                schema_type, 
                properties, 
                required,
                items,
                enum_values,
                nullable,
                all_of,
                one_of,
                any_of,
                ..
            } => {
                let mut ts_type = String::new();
                
                // Handle composition types
                if let Some(all_of_schemas) = all_of {
                    let types: Result<Vec<String>, _> = all_of_schemas.iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    ts_type = types?.join(" & ");
                } else if let Some(one_of_schemas) = one_of {
                    let types: Result<Vec<String>, _> = one_of_schemas.iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    ts_type = types?.join(" | ");
                } else if let Some(any_of_schemas) = any_of {
                    let types: Result<Vec<String>, _> = any_of_schemas.iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    ts_type = types?.join(" | ");
                } else {
                    // Handle basic types
                    match schema_type.as_deref() {
                        Some("string") => {
                            if let Some(enum_vals) = enum_values {
                                let enum_strings: Vec<String> = enum_vals.iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| format!("\"{}\"", s))
                                    .collect();
                                ts_type = enum_strings.join(" | ");
                            } else {
                                ts_type = "string".to_string();
                            }
                        }
                        Some("number") | Some("integer") => {
                            ts_type = "number".to_string();
                        }
                        Some("boolean") => {
                            ts_type = "boolean".to_string();
                        }
                        Some("array") => {
                            if let Some(items_schema) = items {
                                let item_type = self.schema_to_typescript(items_schema)?;
                                ts_type = format!("{}[]", item_type);
                            } else {
                                ts_type = "unknown[]".to_string();
                            }
                        }
                        Some("object") | None => {
                            if let Some(props) = properties {
                                let mut object_props = Vec::new();
                                for (prop_name, prop_schema) in props {
                                    let prop_type = self.schema_to_typescript(prop_schema)?;
                                    let is_required = required.as_ref()
                                        .map(|req| req.contains(prop_name))
                                        .unwrap_or(false);
                                    
                                    let optional_marker = if is_required { "" } else { "?" };
                                    object_props.push(format!("    {}{}: {}", 
                                        prop_name, optional_marker, prop_type));
                                }
                                
                                if object_props.is_empty() {
                                    ts_type = "Record<string, unknown>".to_string();
                                } else {
                                    ts_type = format!("{{\n{};\n  }}", object_props.join(";\n"));
                                }
                            } else {
                                ts_type = "Record<string, unknown>".to_string();
                            }
                        }
                        _ => {
                            ts_type = "unknown".to_string();
                        }
                    }
                }
                
                // Apply nullable if needed
                if nullable.unwrap_or(false) {
                    ts_type = format!("{} | null", ts_type);
                }
                
                Ok(ts_type)
            }
        }
    }
}

impl Generator for TypeScriptGenerator {
    fn generate(&self, schema: &OpenApiSchema) -> Result<String> {
        let mut output = String::new();
        
        // Add header comment
        output.push_str("// Generated TypeScript interfaces from OpenAPI schema\n");
        output.push_str("// Do not modify this file manually\n\n");
        
        if let Some(components) = &schema.components {
            if let Some(schemas) = &components.schemas {
                for (name, schema_def) in schemas {
                    output.push_str(&self.generate_interface(name, schema_def)?);
                }
            }
        }
        
        Ok(output)
    }
}

impl Clone for TypeScriptGenerator {
    fn clone(&self) -> Self {
        Self {
            indent_level: self.indent_level,
        }
    }
} 