use crate::generators::Generator;
use crate::openapi::{OpenApiSchema, Schema, is_schema_nullable, schema_type_str};
use anyhow::Result;

pub struct DotNetGenerator {
    indent_level: usize,
}

impl Default for DotNetGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DotNetGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn to_pascal_case(camel_case: &str) -> String {
        if camel_case.is_empty() {
            return camel_case.to_string();
        }

        let mut chars = camel_case.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    fn generate_class(&self, name: &str, schema: &Schema) -> Result<String> {
        let mut output = String::new();

        match schema {
            Schema::Object {
                schema_type,
                properties,
                required,
                enum_values,
                one_of: _,
                any_of: _,
                description,
                ..
            } => {
                // Handle enum types - keep as enums, not records
                if let Some(enum_vals) = enum_values {
                    if let Some(desc) = description {
                        output.push_str(&format!("{}/// <summary>\n", self.indent()));
                        output.push_str(&format!("{}/// {}\n", self.indent(), desc));
                        output.push_str(&format!("{}/// </summary>\n", self.indent()));
                    }
                    output.push_str(&format!("{}public enum {}\n", self.indent(), name));
                    output.push_str(&format!("{}{{\n", self.indent()));

                    for (i, enum_val) in enum_vals.iter().enumerate() {
                        if let Some(val_str) = enum_val.as_str() {
                            let enum_name = Self::to_pascal_case(val_str);
                            if i > 0 {
                                output.push_str(",\n");
                            }
                            output.push_str(&format!(
                                "{}    [JsonPropertyName(\"{}\")]\n",
                                self.indent(),
                                val_str
                            ));
                            output.push_str(&format!("{}    {}", self.indent(), enum_name));
                        }
                    }
                    output.push_str(&format!("\n{}}}\n\n", self.indent()));
                    return Ok(output);
                }

                // Handle object types - generate records instead of classes
                if schema_type_str(schema_type) == Some("object") || properties.is_some() {
                    if let Some(desc) = description {
                        output.push_str(&format!("{}/// <summary>\n", self.indent()));
                        output.push_str(&format!("{}/// {}\n", self.indent(), desc));
                        output.push_str(&format!("{}/// </summary>\n", self.indent()));
                    }
                    output.push_str(&format!("{}public record {}\n", self.indent(), name));
                    output.push_str(&format!("{}{{\n", self.indent()));

                    if let Some(props) = properties {
                        let empty_vec = Vec::new();
                        let required_fields = required.as_ref().unwrap_or(&empty_vec);

                        for (prop_name, prop_schema) in props {
                            let pascal_prop_name = Self::to_pascal_case(prop_name);
                            let cs_type = self.schema_to_csharp_type(prop_schema)?;
                            let is_required = required_fields.contains(prop_name);

                            // Add JsonPropertyName attribute for camelCase conversion
                            output.push_str(&format!(
                                "{}    [JsonPropertyName(\"{}\")]\n",
                                self.indent(),
                                prop_name
                            ));

                            // Handle required vs optional properties
                            if is_required {
                                // Required properties use the 'required' modifier
                                output.push_str(&format!(
                                    "{}    public required {} {} {{ get; set; }}\n",
                                    self.indent(),
                                    cs_type,
                                    pascal_prop_name
                                ));
                            } else {
                                // Non-required properties are nullable
                                let nullable_type =
                                    if !cs_type.ends_with('?') && !cs_type.starts_with("List<") {
                                        format!("{cs_type}?")
                                    } else {
                                        cs_type
                                    };

                                output.push_str(&format!(
                                    "{}    public {} {} {{ get; set; }}\n",
                                    self.indent(),
                                    nullable_type,
                                    pascal_prop_name
                                ));
                            }
                        }
                    }

                    output.push_str(&format!("{}}}\n\n", self.indent()));
                }
            }
            Schema::Reference { .. } => {
                let cs_type = self.schema_to_csharp_type(schema)?;
                output.push_str(&format!(
                    "{}// Type alias: {} = {}\n\n",
                    self.indent(),
                    name,
                    cs_type
                ));
            }
        }

        Ok(output)
    }

    fn is_value_type(&self, cs_type: &str) -> bool {
        matches!(
            cs_type,
            "int"
                | "long"
                | "float"
                | "double"
                | "decimal"
                | "bool"
                | "byte"
                | "sbyte"
                | "short"
                | "ushort"
                | "uint"
                | "ulong"
                | "char"
                | "DateTime"
                | "DateTimeOffset"
                | "TimeSpan"
                | "Guid"
        )
    }

    fn schema_to_csharp_type(&self, schema: &Schema) -> Result<String> {
        match schema {
            Schema::Object {
                schema_type,
                format,
                enum_values,
                items,
                nullable,
                ..
            } => {
                // Handle enum values
                if enum_values.is_some() {
                    return Ok("string".to_string());
                }

                let base_type = if let Some(type_str) = schema_type_str(schema_type) {
                    match type_str {
                        "string" => match format.as_deref() {
                            Some("uuid") => "Guid",
                            Some("date") => "DateOnly",
                            Some("date-time") => "DateTime",
                            Some("byte") => "byte[]",
                            _ => "string",
                        },
                        "integer" => match format.as_deref() {
                            Some("int64") => "long",
                            _ => "int",
                        },
                        "number" => match format.as_deref() {
                            Some("float") => "float",
                            Some("decimal") => "decimal",
                            _ => "double",
                        },
                        "boolean" => "bool",
                        "array" => {
                            if let Some(item_schema) = items {
                                let item_type = self.schema_to_csharp_type(item_schema)?;
                                return Ok(format!("List<{item_type}>"));
                            } else {
                                "List<object>"
                            }
                        }
                        "object" => "Dictionary<string, object>",
                        _ => "object",
                    }
                } else {
                    "object"
                }
                .to_string();

                Ok(
                    if is_schema_nullable(nullable, schema_type) && self.is_value_type(&base_type) {
                        format!("{base_type}?")
                    } else {
                        base_type
                    },
                )
            }
            Schema::Reference { reference } => {
                let type_name = reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                Ok(type_name.to_string())
            }
        }
    }
}

impl Generator for DotNetGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!("// Generated by {command}\n"));
        output.push_str("// Do not modify manually\n\n");

        // Add using statements
        output.push_str("using System.ComponentModel.DataAnnotations;\n");
        output.push_str("using System.Text.Json.Serialization;\n\n");

        if let Some(components) = &schema.components
            && let Some(schemas) = &components.schemas
        {
            for (name, schema_def) in schemas {
                let class_def = self.generate_class(name, schema_def)?;
                output.push_str(&class_def);
            }
        }

        Ok(output)
    }
}

impl Clone for DotNetGenerator {
    fn clone(&self) -> Self {
        Self {
            indent_level: self.indent_level,
        }
    }
}
