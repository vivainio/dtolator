use crate::generators::Generator;
use crate::generators::common;
use crate::openapi::{OpenApiSchema, Schema};
use anyhow::Result;

pub struct RustSerdeGenerator;

impl Default for RustSerdeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RustSerdeGenerator {
    pub fn new() -> Self {
        Self
    }

    fn to_snake_case(&self, name: &str) -> String {
        let mut result = String::new();
        let mut prev_was_upper = false;

        for (i, ch) in name.chars().enumerate() {
            if ch.is_uppercase() {
                if i > 0 && !prev_was_upper {
                    result.push('_');
                }
                result.push(ch.to_lowercase().next().unwrap_or(ch));
                prev_was_upper = true;
            } else if ch == '-' || ch == ' ' {
                if !result.ends_with('_') {
                    result.push('_');
                }
                prev_was_upper = false;
            } else {
                result.push(ch);
                prev_was_upper = false;
            }
        }

        if result.is_empty() {
            "field".to_string()
        } else {
            result
        }
    }

    fn to_rust_type(&self, schema: &Schema) -> Result<String> {
        match schema {
            Schema::Reference { reference } => {
                let type_name = reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference)
                    .to_string();
                Ok(type_name)
            }
            Schema::Object {
                schema_type,
                format,
                enum_values,
                items,
                ..
            } => {
                if let Some(enum_vals) = enum_values
                    && enum_vals.iter().all(|v| v.is_string())
                {
                    return Ok("String".to_string());
                }

                match schema_type.as_deref() {
                    Some("string") => match format.as_deref() {
                        Some("date") => Ok("String".to_string()),
                        Some("date-time") => Ok("String".to_string()),
                        Some("uuid") => Ok("String".to_string()),
                        Some("email") => Ok("String".to_string()),
                        Some("uri") | Some("url") => Ok("String".to_string()),
                        _ => Ok("String".to_string()),
                    },
                    Some("integer") => match format.as_deref() {
                        Some("int32") => Ok("i32".to_string()),
                        Some("int64") => Ok("i64".to_string()),
                        _ => Ok("i64".to_string()),
                    },
                    Some("number") => match format.as_deref() {
                        Some("float") => Ok("f32".to_string()),
                        Some("double") => Ok("f64".to_string()),
                        _ => Ok("f64".to_string()),
                    },
                    Some("boolean") => Ok("bool".to_string()),
                    Some("array") => {
                        if let Some(items_schema) = items {
                            let item_type = self.to_rust_type(items_schema)?;
                            Ok(format!("Vec<{}>", item_type))
                        } else {
                            Ok("Vec<serde_json::Value>".to_string())
                        }
                    }
                    Some("object") => Ok("serde_json::Value".to_string()),
                    _ => Ok("serde_json::Value".to_string()),
                }
            }
        }
    }

    fn generate_struct(&self, name: &str, schema: &Schema) -> Result<String> {
        let mut output = String::new();

        match schema {
            Schema::Object {
                properties,
                required,
                enum_values,
                all_of,
                one_of,
                any_of,
                ..
            } => {
                if let Some(enum_vals) = enum_values {
                    output.push_str("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]\n");
                    output.push_str("#[serde(rename_all = \"snake_case\")]\n");
                    output.push_str(&format!("pub enum {} {{\n", name));

                    for enum_val in enum_vals {
                        if let Some(val_str) = enum_val.as_str() {
                            let variant_name = self.to_rust_enum_variant(val_str);
                            output.push_str(&format!("    #[serde(rename = \"{}\")]\n", val_str));
                            output.push_str(&format!("    {},\n", variant_name));
                        }
                    }

                    output.push_str("}\n\n");
                    return Ok(output);
                }

                output.push_str("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]\n");
                output.push_str("#[serde(rename_all = \"camelCase\")]\n");
                output.push_str(&format!("pub struct {} {{\n", name));

                if let Some(all_of_schemas) = all_of {
                    for schema in all_of_schemas.iter() {
                        if let Schema::Object {
                            properties: Some(props),
                            required: req,
                            ..
                        } = schema
                        {
                            for (prop_name, prop_schema) in props {
                                let field_name = self.to_snake_case(prop_name);
                                let field_type = self.to_rust_type(prop_schema)?;
                                let is_required =
                                    req.as_ref().map(|r| r.contains(prop_name)).unwrap_or(false);

                                let final_type = if !is_required {
                                    format!("Option<{}>", field_type)
                                } else {
                                    field_type
                                };

                                if prop_name != &field_name {
                                    output.push_str(&format!(
                                        "    #[serde(rename = \"{}\")]\n",
                                        prop_name
                                    ));
                                }
                                output.push_str(&format!(
                                    "    pub {}: {},\n",
                                    field_name, final_type
                                ));
                            }
                        }
                    }
                } else if let Some(props) = properties {
                    for (prop_name, prop_schema) in props {
                        let field_name = self.to_snake_case(prop_name);
                        let field_type = self.to_rust_type(prop_schema)?;
                        let is_required = required
                            .as_ref()
                            .map(|req| req.contains(prop_name))
                            .unwrap_or(false);

                        let final_type = if !is_required {
                            format!("Option<{}>", field_type)
                        } else {
                            field_type
                        };

                        if prop_name != &field_name {
                            output.push_str(&format!("    #[serde(rename = \"{}\")]\n", prop_name));
                        }
                        output.push_str(&format!("    pub {}: {},\n", field_name, final_type));
                    }
                }

                if one_of.is_some() || any_of.is_some() {
                    output.push_str(
                        "    // Note: oneOf/anyOf schemas are combined as optional fields\n",
                    );
                }

                output.push_str("}\n\n");
            }
            _ => {
                output.push_str(&format!("pub type {} = serde_json::Value;\n\n", name));
            }
        }

        Ok(output)
    }

    fn to_rust_enum_variant(&self, value: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;

        for ch in value.chars() {
            if ch.is_alphanumeric() {
                if capitalize_next {
                    result.push(ch.to_uppercase().next().unwrap_or(ch));
                    capitalize_next = false;
                } else {
                    result.push(ch);
                }
            } else if ch == '-' || ch == '_' || ch == ' ' {
                capitalize_next = true;
            }
        }

        if result.is_empty() {
            "Value".to_string()
        } else {
            result
        }
    }
}

impl Generator for RustSerdeGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, _command: &str) -> Result<String> {
        let mut output = String::new();

        output.push_str("use serde::{Deserialize, Serialize};\n\n");

        if let Some(components) = &schema.components
            && let Some(schemas) = &components.schemas
        {
            let sorted_names = common::topological_sort(schemas)?;

            for name in sorted_names {
                if let Some(schema) = schemas.get(&name) {
                    output.push_str(&self.generate_struct(&name, schema)?);
                }
            }
        }

        Ok(output)
    }
}
