use anyhow::Result;
use crate::openapi::{OpenApiSchema, Schema};
use crate::generators::Generator;

pub struct PythonDictGenerator {
    indent_level: usize,
}

impl PythonDictGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }
    
    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }
    
    fn generate_typed_dict(&self, name: &str, schema: &Schema) -> Result<String> {
        let mut output = String::new();
        
        match schema {
            Schema::Object { 
                schema_type, 
                properties, 
                required,
                enum_values,

                one_of,
                any_of,
                ..
            } => {
                // Handle enum types
                if let Some(enum_vals) = enum_values {
                    output.push_str(&format!("{}class {}(str, Enum):\n", self.indent(), name));
                    for enum_val in enum_vals {
                        if let Some(val_str) = enum_val.as_str() {
                            let enum_name = val_str.to_uppercase().replace(" ", "_").replace("-", "_");
                            output.push_str(&format!("{}    {} = \"{}\"\n", self.indent(), enum_name, val_str));
                        }
                    }
                    output.push_str("\n\n");
                    return Ok(output);
                }
                
                // Handle composition types
                if let Some(one_of_schemas) = one_of {
                    // For oneOf, we'll create a Union type
                    let types: Result<Vec<String>, _> = one_of_schemas.iter()
                        .map(|s| self.schema_to_python_type(s))
                        .collect();
                    output.push_str(&format!("{}{} = Union[{}]\n\n", 
                        self.indent(), name, types?.join(", ")));
                    return Ok(output);
                } else if let Some(any_of_schemas) = any_of {
                    // For anyOf, we'll also create a Union type
                    let types: Result<Vec<String>, _> = any_of_schemas.iter()
                        .map(|s| self.schema_to_python_type(s))
                        .collect();
                    output.push_str(&format!("{}{} = Union[{}]\n\n", 
                        self.indent(), name, types?.join(", ")));
                    return Ok(output);
                }
                
                // Handle object types
                if schema_type.as_deref() == Some("object") || properties.is_some() {
                    let empty_required = vec![];
                    let required_fields: Vec<&String> = required.as_ref().unwrap_or(&empty_required).iter().collect();
                    
                    if let Some(props) = properties {
                        if props.is_empty() {
                            output.push_str(&format!("{}class {}(TypedDict):\n", self.indent(), name));
                            output.push_str(&format!("{}    pass\n", self.indent()));
                        } else {
                            let has_required = required_fields.iter().any(|field| props.contains_key(*field));
                            let has_optional = props.keys().any(|key| !required_fields.contains(&key));
                            
                            if has_required && has_optional {
                                // Create base TypedDict for required fields
                                output.push_str(&format!("{}class {}Required(TypedDict):\n", self.indent(), name));
                                for field_name in &required_fields {
                                    if let Some(field_schema) = props.get(*field_name) {
                                        let field_type = self.schema_to_python_type(field_schema)?;
                                        output.push_str(&format!("{}    {}: {}\n", 
                                            self.indent(), field_name, field_type));
                                    }
                                }
                                output.push_str("\n\n");
                                
                                // Create full TypedDict that inherits from required and adds optional fields
                                output.push_str(&format!("{}class {}({}Required, total=False):\n", self.indent(), name, name));
                                for (prop_name, prop_schema) in props {
                                    if !required_fields.contains(&prop_name) {
                                        let field_type = self.schema_to_python_type(prop_schema)?;
                                        output.push_str(&format!("{}    {}: {}\n", 
                                            self.indent(), prop_name, field_type));
                                    }
                                }
                            } else if has_required {
                                // Only required fields
                                output.push_str(&format!("{}class {}(TypedDict):\n", self.indent(), name));
                                for (prop_name, prop_schema) in props {
                                    let field_type = self.schema_to_python_type(prop_schema)?;
                                    output.push_str(&format!("{}    {}: {}\n", 
                                        self.indent(), prop_name, field_type));
                                }
                            } else {
                                // Only optional fields
                                output.push_str(&format!("{}class {}(TypedDict, total=False):\n", self.indent(), name));
                                for (prop_name, prop_schema) in props {
                                    let field_type = self.schema_to_python_type(prop_schema)?;
                                    output.push_str(&format!("{}    {}: {}\n", 
                                        self.indent(), prop_name, field_type));
                                }
                            }
                        }
                    } else {
                        output.push_str(&format!("{}class {}(TypedDict):\n", self.indent(), name));
                        output.push_str(&format!("{}    pass\n", self.indent()));
                    }
                    
                    output.push_str("\n");
                } else {
                    // Handle primitive type aliases
                    let py_type = self.schema_to_python_type(schema)?;
                    output.push_str(&format!("{}{} = {}\n\n", self.indent(), name, py_type));
                }
            }
            Schema::Reference { .. } => {
                // For references, create a type alias
                let py_type = self.schema_to_python_type(schema)?;
                output.push_str(&format!("{}{} = {}\n\n", self.indent(), name, py_type));
            }
        }
        
        Ok(output)
    }
    
    fn schema_to_python_type(&self, schema: &Schema) -> Result<String> {
        match schema {
            Schema::Object { 
                schema_type, 
                format, 
                enum_values, 
                items, 
                nullable,
                one_of,
                any_of,
                ..
            } => {
                // Handle composition types
                if let Some(one_of_schemas) = one_of {
                    let types: Result<Vec<String>, _> = one_of_schemas.iter()
                        .map(|s| self.schema_to_python_type(s))
                        .collect();
                    let union_type = format!("Union[{}]", types?.join(", "));
                    return Ok(if nullable.unwrap_or(false) { format!("Optional[{}]", union_type) } else { union_type });
                } else if let Some(any_of_schemas) = any_of {
                    let types: Result<Vec<String>, _> = any_of_schemas.iter()
                        .map(|s| self.schema_to_python_type(s))
                        .collect();
                    let union_type = format!("Union[{}]", types?.join(", "));
                    return Ok(if nullable.unwrap_or(false) { format!("Optional[{}]", union_type) } else { union_type });
                }
                
                // Handle enum values
                if let Some(enum_vals) = enum_values {
                    if enum_vals.iter().all(|v| v.is_string()) {
                        let values: Vec<String> = enum_vals.iter()
                            .filter_map(|v| v.as_str())
                            .map(|s| format!("\"{}\"", s))
                            .collect();
                        let literal_type = format!("Literal[{}]", values.join(", "));
                        return Ok(if nullable.unwrap_or(false) { format!("Optional[{}]", literal_type) } else { literal_type });
                    }
                }
                
                let base_type = if let Some(type_str) = schema_type {
                    match type_str.as_str() {
                        "string" => {
                            match format.as_deref() {
                                Some("email") => "str",
                                Some("uri") | Some("url") => "str",
                                Some("uuid") => "str",
                                Some("date") => "str",
                                Some("date-time") => "str",
                                _ => "str"
                            }
                        },
                        "integer" => "int",
                        "number" => "float",
                        "boolean" => "bool",
                        "array" => {
                            if let Some(item_schema) = items {
                                let item_type = self.schema_to_python_type(item_schema)?;
                                return Ok(if nullable.unwrap_or(false) { format!("Optional[List[{}]]", item_type) } else { format!("List[{}]", item_type) });
                            } else {
                                "List[Any]"
                            }
                        },
                        "object" => "Dict[str, Any]",
                        _ => "Any"
                    }
                } else {
                    "Any"
                }.to_string();
                
                Ok(if nullable.unwrap_or(false) { format!("Optional[{}]", base_type) } else { base_type })
            }
            Schema::Reference { reference } => {
                let type_name = reference.strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                Ok(type_name.to_string())
            }
        }
    }
}

impl Generator for PythonDictGenerator {
    fn generate(&self, schema: &OpenApiSchema) -> Result<String> {
        let mut output = String::new();
        
        // Add file header
        output.push_str("# Generated Python TypedDict definitions from OpenAPI schema\n");
        output.push_str("# Do not modify this file manually\n\n");
        
        // Add imports
        output.push_str("from datetime import date, datetime\n");
        output.push_str("from enum import Enum\n");
        output.push_str("from typing import Any, Dict, List, Literal, Optional, Union\n");
        output.push_str("from typing_extensions import TypedDict\n");
        output.push_str("from uuid import UUID\n\n\n");
        
        // Generate TypedDict definitions
        if let Some(components) = &schema.components {
            if let Some(schemas) = &components.schemas {
                let mut first = true;
                for (name, schema) in schemas {
                    if !first {
                        output.push_str("\n\n");
                    }
                    first = false;
                    let model_output = self.generate_typed_dict(name, schema)?;
                    output.push_str(&model_output);
                }
            }
        }
        
        // Remove trailing newlines
        let output = output.trim_end().to_string() + "\n";
        Ok(output)
    }
}

impl Clone for PythonDictGenerator {
    fn clone(&self) -> Self {
        Self { indent_level: self.indent_level }
    }
} 