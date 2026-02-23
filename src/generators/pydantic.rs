use crate::generators::Generator;
use crate::openapi::{
    AdditionalProperties, OpenApiSchema, Schema, is_schema_nullable, schema_type_str,
};
use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PydanticVersion {
    V1,
    V2,
}

pub struct PydanticGenerator {
    indent_level: usize,
    version: PydanticVersion,
}

impl Default for PydanticGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PydanticGenerator {
    pub fn new() -> Self {
        Self {
            indent_level: 0,
            version: PydanticVersion::V1,
        }
    }

    pub fn new_v2() -> Self {
        Self {
            indent_level: 0,
            version: PydanticVersion::V2,
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn is_optional_type(&self, ty: &str) -> bool {
        match self.version {
            PydanticVersion::V1 => ty.starts_with("Optional["),
            PydanticVersion::V2 => ty.ends_with("| None"),
        }
    }

    fn wrap_optional(&self, ty: &str) -> String {
        match self.version {
            PydanticVersion::V1 => format!("Optional[{ty}]"),
            PydanticVersion::V2 => format!("{ty} | None"),
        }
    }

    fn format_union(&self, types: &[String]) -> String {
        match self.version {
            PydanticVersion::V1 => format!("Union[{}]", types.join(", ")),
            PydanticVersion::V2 => types.join(" | "),
        }
    }

    fn format_list(&self, item_type: &str) -> String {
        match self.version {
            PydanticVersion::V1 => format!("List[{item_type}]"),
            PydanticVersion::V2 => format!("list[{item_type}]"),
        }
    }

    fn format_dict(&self, key_type: &str, value_type: &str) -> String {
        match self.version {
            PydanticVersion::V1 => format!("Dict[{key_type}, {value_type}]"),
            PydanticVersion::V2 => format!("dict[{key_type}, {value_type}]"),
        }
    }

    fn generate_model(&self, name: &str, schema: &Schema) -> Result<String> {
        let mut output = String::new();

        match schema {
            Schema::Object {
                schema_type,
                properties,
                required,
                additional_properties,
                enum_values,
                all_of,
                one_of,
                any_of,
                ..
            } => {
                // Handle enum types
                if let Some(enum_vals) = enum_values {
                    output.push_str(&format!("{}class {}(str, Enum):\n", self.indent(), name));
                    for enum_val in enum_vals {
                        if let Some(val_str) = enum_val.as_str() {
                            let enum_name =
                                val_str.to_uppercase().replace(" ", "_").replace("-", "_");
                            output.push_str(&format!(
                                "{}    {} = \"{}\"\n",
                                self.indent(),
                                enum_name,
                                val_str
                            ));
                        }
                    }
                    output.push_str("\n\n");
                    return Ok(output);
                }

                // Handle composition types
                if let Some(all_of_schemas) = all_of {
                    // For allOf, we'll create a model that inherits from multiple bases
                    output.push_str(&format!("{}class {}(BaseModel):\n", self.indent(), name));
                    output.push_str(&format!(
                        "{}    # This model combines multiple schemas (allOf)\n",
                        self.indent()
                    ));

                    // Add fields from all schemas
                    for schema in all_of_schemas.iter() {
                        if let Schema::Object {
                            properties: Some(props),
                            required,
                            ..
                        } = schema
                        {
                            for (prop_name, prop_schema) in props {
                                let base_field_type = self.schema_to_pydantic_type(prop_schema)?;
                                let is_required = required
                                    .as_ref()
                                    .map(|req| req.contains(prop_name))
                                    .unwrap_or(false);

                                if is_required {
                                    output.push_str(&format!(
                                        "{}    {}: {}\n",
                                        self.indent(),
                                        prop_name,
                                        base_field_type
                                    ));
                                } else {
                                    // Handle nullable + optional case to avoid double-wrapping
                                    if self.is_optional_type(&base_field_type) {
                                        output.push_str(&format!(
                                            "{}    {}: {} = None\n",
                                            self.indent(),
                                            prop_name,
                                            base_field_type
                                        ));
                                    } else {
                                        output.push_str(&format!(
                                            "{}    {}: {} = None\n",
                                            self.indent(),
                                            prop_name,
                                            self.wrap_optional(&base_field_type)
                                        ));
                                    }
                                }
                            }
                        } else if let Schema::Reference { reference } = schema {
                            let ref_name = reference
                                .strip_prefix("#/components/schemas/")
                                .unwrap_or(reference);
                            output.push_str(&format!(
                                "{}    # Inherits from {}\n",
                                self.indent(),
                                ref_name
                            ));
                        }
                    }
                    output.push_str("\n\n");
                    return Ok(output);
                } else if let Some(one_of_schemas) = one_of {
                    // For oneOf, we'll create a Union type
                    let types: Result<Vec<String>, _> = one_of_schemas
                        .iter()
                        .map(|s| self.schema_to_pydantic_type(s))
                        .collect();
                    output.push_str(&format!(
                        "{}{} = {}\n\n",
                        self.indent(),
                        name,
                        self.format_union(&types?)
                    ));
                    return Ok(output);
                } else if let Some(any_of_schemas) = any_of {
                    // For anyOf, we'll also create a Union type
                    let types: Result<Vec<String>, _> = any_of_schemas
                        .iter()
                        .map(|s| self.schema_to_pydantic_type(s))
                        .collect();
                    output.push_str(&format!(
                        "{}{} = {}\n\n",
                        self.indent(),
                        name,
                        self.format_union(&types?)
                    ));
                    return Ok(output);
                }

                // Handle map types (additionalProperties without properties)
                if properties.is_none()
                    && matches!(additional_properties, Some(AdditionalProperties::Schema(_)))
                {
                    let py_type = self.schema_to_pydantic_type(schema)?;
                    output.push_str(&format!("{}{} = {}\n\n", self.indent(), name, py_type));
                    return Ok(output);
                }

                // Handle object types
                if schema_type_str(schema_type) == Some("object") || properties.is_some() {
                    output.push_str(&format!("{}class {}(BaseModel):\n", self.indent(), name));

                    if let Some(props) = properties {
                        if props.is_empty() {
                            output.push_str(&format!("{}    pass\n", self.indent()));
                        } else {
                            for (prop_name, prop_schema) in props {
                                let base_field_type = self.schema_to_pydantic_type(prop_schema)?;
                                let is_required = required
                                    .as_ref()
                                    .map(|req| req.contains(prop_name))
                                    .unwrap_or(false);

                                // Handle nullable + optional case to avoid double Optional
                                let field_type = base_field_type;

                                // Add field with validation constraints
                                let field_def = self.generate_field_definition(
                                    prop_name,
                                    prop_schema,
                                    field_type,
                                    is_required,
                                )?;
                                output.push_str(&format!("{}    {}\n", self.indent(), field_def));
                            }
                        }
                    } else {
                        output.push_str(&format!("{}    pass\n", self.indent()));
                    }

                    output.push('\n');
                } else {
                    // Handle primitive type aliases
                    let py_type = self.schema_to_pydantic_type(schema)?;
                    output.push_str(&format!("{}{} = {}\n\n", self.indent(), name, py_type));
                }
            }
            Schema::Reference { .. } => {
                // For references, create a type alias
                let py_type = self.schema_to_pydantic_type(schema)?;
                output.push_str(&format!("{}{} = {}\n\n", self.indent(), name, py_type));
            }
        }

        Ok(output)
    }

    fn generate_field_definition(
        &self,
        name: &str,
        schema: &Schema,
        field_type: String,
        is_required: bool,
    ) -> Result<String> {
        if let Schema::Object {
            min_length,
            max_length,
            minimum,
            maximum,
            pattern,
            description,
            ..
        } = schema
        {
            let mut constraints = Vec::new();

            // Add validation constraints
            if let Some(min_len) = min_length {
                constraints.push(format!("min_length={min_len}"));
            }
            if let Some(max_len) = max_length {
                constraints.push(format!("max_length={max_len}"));
            }
            if let Some(min_val) = minimum {
                constraints.push(format!("ge={min_val}"));
            }
            if let Some(max_val) = maximum {
                constraints.push(format!("le={max_val}"));
            }
            if let Some(regex) = pattern {
                let pattern_key = match self.version {
                    PydanticVersion::V1 => "regex",
                    PydanticVersion::V2 => "pattern",
                };
                constraints.push(format!("{pattern_key}=r\"{regex}\""));
            }
            if let Some(desc) = description {
                constraints.push(format!("description=\"{}\"", desc.replace("\"", "\\\"")));
            }

            if is_required {
                if constraints.is_empty() {
                    Ok(format!("{name}: {field_type}"))
                } else {
                    Ok(format!(
                        "{}: {} = Field({})",
                        name,
                        field_type,
                        constraints.join(", ")
                    ))
                }
            } else if constraints.is_empty() {
                if self.is_optional_type(&field_type) {
                    Ok(format!("{name}: {field_type} = None"))
                } else {
                    Ok(format!(
                        "{}: {} = None",
                        name,
                        self.wrap_optional(&field_type)
                    ))
                }
            } else if self.is_optional_type(&field_type) {
                Ok(format!(
                    "{}: {} = Field(None, {})",
                    name,
                    field_type,
                    constraints.join(", ")
                ))
            } else {
                Ok(format!(
                    "{}: {} = Field(None, {})",
                    name,
                    self.wrap_optional(&field_type),
                    constraints.join(", ")
                ))
            }
        } else if is_required {
            Ok(format!("{name}: {field_type}"))
        } else if self.is_optional_type(&field_type) {
            Ok(format!("{name}: {field_type} = None"))
        } else {
            Ok(format!(
                "{}: {} = None",
                name,
                self.wrap_optional(&field_type)
            ))
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn schema_to_pydantic_type(&self, schema: &Schema) -> Result<String> {
        match schema {
            Schema::Reference { reference } => {
                let ref_name = reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                Ok(ref_name.to_string())
            }
            Schema::Object {
                schema_type,
                properties,
                additional_properties,
                items,
                enum_values,
                nullable,
                all_of,
                one_of,
                any_of,
                format,
                ..
            } => {
                let py_type;

                // Handle composition types
                if let Some(_all_of_schemas) = all_of {
                    // For inline allOf, we'll create an anonymous model
                    py_type = self.format_dict("str", "Any"); // Fallback for complex inline types
                } else if let Some(one_of_schemas) = one_of {
                    let types: Result<Vec<String>, _> = one_of_schemas
                        .iter()
                        .map(|s| self.schema_to_pydantic_type(s))
                        .collect();
                    py_type = self.format_union(&types?);
                } else if let Some(any_of_schemas) = any_of {
                    let types: Result<Vec<String>, _> = any_of_schemas
                        .iter()
                        .map(|s| self.schema_to_pydantic_type(s))
                        .collect();
                    py_type = self.format_union(&types?);
                } else {
                    // Handle basic types
                    match schema_type_str(schema_type) {
                        Some("string") => {
                            if let Some(enum_vals) = enum_values {
                                let enum_strings: Vec<String> = enum_vals
                                    .iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| format!("\"{s}\""))
                                    .collect();
                                py_type = format!("Literal[{}]", enum_strings.join(", "));
                            } else {
                                match format.as_deref() {
                                    Some("email") => py_type = "EmailStr".to_string(),
                                    Some("uri") => py_type = "HttpUrl".to_string(),
                                    Some("date") => py_type = "date".to_string(),
                                    Some("date-time") => py_type = "datetime".to_string(),
                                    Some("uuid") => py_type = "UUID".to_string(),
                                    _ => py_type = "str".to_string(),
                                }
                            }
                        }
                        Some("number") => {
                            py_type = "float".to_string();
                        }
                        Some("integer") => {
                            py_type = "int".to_string();
                        }
                        Some("boolean") => {
                            py_type = "bool".to_string();
                        }
                        Some("array") => {
                            if let Some(items_schema) = items {
                                let item_type = self.schema_to_pydantic_type(items_schema)?;
                                py_type = self.format_list(&item_type);
                            } else {
                                py_type = self.format_list("Any");
                            }
                        }
                        Some("object") | None => {
                            if properties.is_none() {
                                if let Some(AdditionalProperties::Schema(ap_schema)) =
                                    additional_properties
                                {
                                    let value_type = self.schema_to_pydantic_type(ap_schema)?;
                                    py_type = self.format_dict("str", &value_type);
                                } else {
                                    py_type = self.format_dict("str", "Any");
                                }
                            } else {
                                py_type = self.format_dict("str", "Any");
                            }
                        }
                        _ => {
                            py_type = "Any".to_string();
                        }
                    }
                }

                // Handle nullable types
                let final_type = if is_schema_nullable(nullable, schema_type) {
                    self.wrap_optional(&py_type)
                } else {
                    py_type
                };

                Ok(final_type)
            }
        }
    }
}

impl Generator for PydanticGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!("# Generated by {command}\n"));
        output.push_str("# Do not modify manually\n\n");

        // Add imports (version-specific)
        match self.version {
            PydanticVersion::V1 => {
                output.push_str("from pydantic import BaseModel, Field, EmailStr, HttpUrl\n");
                output.push_str("from typing import Optional, Union, List, Dict, Any, Literal\n");
                output.push_str("from enum import Enum\n");
                output.push_str("from datetime import date, datetime\n");
                output.push_str("from uuid import UUID\n\n");
            }
            PydanticVersion::V2 => {
                output.push_str("from pydantic import BaseModel, Field, EmailStr, HttpUrl\n");
                output.push_str("from typing import Any, Literal\n");
                output.push_str("from enum import Enum\n");
                output.push_str("from datetime import date, datetime\n");
                output.push_str("from uuid import UUID\n\n");
            }
        }

        if let Some(components) = &schema.components
            && let Some(schemas) = &components.schemas
        {
            for (name, schema_def) in schemas {
                let model = self.generate_model(name, schema_def)?;
                output.push_str(&model);
            }
        }

        Ok(output)
    }
}

impl Clone for PydanticGenerator {
    fn clone(&self) -> Self {
        Self {
            indent_level: self.indent_level,
            version: self.version,
        }
    }
}
