use crate::generators::Generator;
use crate::generators::common::topological_sort;
use crate::openapi::{
    AdditionalProperties, OpenApiSchema, Schema, is_schema_nullable, schema_type_str,
};
use anyhow::Result;
use indexmap::IndexMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum PydanticVersion {
    #[value(name = "1")]
    V1,
    #[value(name = "2")]
    V2,
}

pub struct PydanticGenerator {
    indent_level: usize,
    version: PydanticVersion,
}

impl Default for PydanticGenerator {
    fn default() -> Self {
        Self::new(PydanticVersion::V1)
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

/// Convert a camelCase or PascalCase identifier to snake_case.
/// e.g. "isActive" -> "is_active", "firstName" -> "first_name"
fn to_snake_case(name: &str) -> String {
    let mut result = String::new();
    let chars: Vec<char> = name.chars().collect();
    for (i, &c) in chars.iter().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                let prev = chars[i - 1];
                let next = chars.get(i + 1).copied();
                if prev.is_lowercase() || prev.is_ascii_digit() {
                    result.push('_');
                } else if let Some(next_c) = next
                    && next_c.is_lowercase()
                {
                    result.push('_');
                }
            }
            result.extend(c.to_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}

/// Tracks which Python symbols need to be imported.
#[derive(Default)]
struct UsedImports {
    base_model: bool,
    field: bool,
    email_str: bool,
    http_url: bool,
    config_dict: bool,
    any: bool,
    literal: bool,
    enum_cls: bool,
    date: bool,
    datetime_: bool,
    uuid: bool,
}

impl PydanticGenerator {
    pub fn new(version: PydanticVersion) -> Self {
        Self {
            indent_level: 0,
            version,
        }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    fn is_optional_type(&self, ty: &str) -> bool {
        ty.ends_with("| None")
    }

    fn wrap_optional(&self, ty: &str) -> String {
        format!("{ty} | None")
    }

    fn format_union(&self, types: &[String]) -> String {
        types.join(" | ")
    }

    fn format_list(&self, item_type: &str) -> String {
        format!("list[{item_type}]")
    }

    fn format_dict(&self, key_type: &str, value_type: &str) -> String {
        format!("dict[{key_type}, {value_type}]")
    }

    /// Collect all import requirements by scanning the schema map.
    fn collect_imports(&self, schemas: &IndexMap<String, Schema>) -> UsedImports {
        let mut used = UsedImports::default();
        for schema in schemas.values() {
            // Top-level enum → generates class X(str, Enum), not Literal
            if let Schema::Object {
                enum_values: Some(_),
                ..
            } = schema
            {
                used.enum_cls = true;
            }
            self.scan_top_level_schema(schema, &mut used);
        }
        used
    }

    /// Scan a top-level schema for import requirements.
    fn scan_top_level_schema(&self, schema: &Schema, used: &mut UsedImports) {
        let Schema::Object {
            schema_type,
            properties,
            additional_properties,
            items,
            enum_values,
            all_of,
            one_of,
            any_of,
            min_length,
            max_length,
            minimum,
            maximum,
            pattern,
            ..
        } = schema
        else {
            return;
        };

        // Top-level enum: generate_model returns early, no further scanning needed
        if enum_values.is_some() {
            return;
        }

        // allOf composition → BaseModel
        if let Some(all_of_schemas) = all_of {
            used.base_model = true;
            for sub in all_of_schemas {
                if let Schema::Object {
                    properties: Some(props),
                    ..
                } = sub
                {
                    if props.keys().any(|k| to_snake_case(k) != *k) {
                        used.field = true;
                        if self.version == PydanticVersion::V2 {
                            used.config_dict = true;
                        }
                    }
                    for prop_schema in props.values() {
                        if prop_schema.get_description().is_some() {
                            used.field = true;
                        }
                        self.scan_inline_schema(prop_schema, used);
                    }
                } else {
                    self.scan_inline_schema(sub, used);
                }
            }
            return;
        }

        // oneOf or anyOf at top level → type alias
        // Null schemas are filtered out (become "None" in the union, not "Any")
        if one_of.is_some() || any_of.is_some() {
            for schemas in [one_of, any_of].into_iter().flatten() {
                for s in schemas.iter().filter(|s| s.get_type() != Some("null")) {
                    self.scan_inline_schema(s, used);
                }
            }
            return;
        }

        // Map type (additionalProperties but no properties) → type alias
        if properties.is_none()
            && matches!(additional_properties, Some(AdditionalProperties::Schema(_)))
        {
            if let Some(AdditionalProperties::Schema(ap)) = additional_properties {
                self.scan_inline_schema(ap, used);
            }
            return;
        }

        // Constraints on the schema itself (rare for top-level objects)
        if min_length.is_some()
            || max_length.is_some()
            || minimum.is_some()
            || maximum.is_some()
            || pattern.is_some()
        {
            used.field = true;
        }

        // Object with properties → BaseModel
        if schema_type_str(schema_type) == Some("object") || properties.is_some() {
            used.base_model = true;
            if let Some(props) = properties {
                // Check for aliases (camelCase field names)
                if props.keys().any(|k| to_snake_case(k) != *k) {
                    used.field = true;
                    if self.version == PydanticVersion::V2 {
                        used.config_dict = true;
                    }
                }
                for prop_schema in props.values() {
                    if prop_schema.get_description().is_some() {
                        used.field = true;
                    }
                    self.scan_inline_schema(prop_schema, used);
                }
            }
            return;
        }

        // array type at top-level
        if schema_type_str(schema_type) == Some("array") {
            if let Some(items_schema) = items {
                self.scan_inline_schema(items_schema, used);
            } else {
                used.any = true;
            }
        }
    }

    /// Scan a schema used inline (as a property type, oneOf member, etc.)
    /// to determine what imports are needed. Mirrors schema_to_pydantic_type logic.
    fn scan_inline_schema(&self, schema: &Schema, used: &mut UsedImports) {
        let Schema::Object {
            schema_type,
            properties,
            additional_properties,
            items,
            enum_values,
            format,
            all_of,
            one_of,
            any_of,
            min_length,
            max_length,
            minimum,
            maximum,
            pattern,
            ..
        } = schema
        else {
            return; // Reference: just uses the type name, no imports needed
        };

        // Constraints on this inline schema → Field needed on parent
        if min_length.is_some()
            || max_length.is_some()
            || minimum.is_some()
            || maximum.is_some()
            || pattern.is_some()
        {
            used.field = true;
        }

        // Mirror schema_to_pydantic_type branching
        if all_of.is_some() {
            // allOf inline → dict[str, Any]
            used.any = true;
            if let Some(schemas) = all_of {
                for s in schemas {
                    self.scan_inline_schema(s, used);
                }
            }
            return;
        }

        // Null schemas in oneOf/anyOf are filtered out (they become "None" in the union)
        if let Some(one_of_schemas) = one_of {
            for s in one_of_schemas
                .iter()
                .filter(|s| s.get_type() != Some("null"))
            {
                self.scan_inline_schema(s, used);
            }
            return;
        }

        if let Some(any_of_schemas) = any_of {
            for s in any_of_schemas
                .iter()
                .filter(|s| s.get_type() != Some("null"))
            {
                self.scan_inline_schema(s, used);
            }
            return;
        }

        match schema_type_str(schema_type) {
            Some("string") => {
                if enum_values.is_some() {
                    used.literal = true;
                } else {
                    match format.as_deref() {
                        Some("email") => used.email_str = true,
                        Some("uri") => used.http_url = true,
                        Some("date") => used.date = true,
                        Some("date-time") => used.datetime_ = true,
                        Some("uuid") => used.uuid = true,
                        _ => {}
                    }
                }
            }
            Some("number") | Some("integer") | Some("boolean") => {}
            Some("array") => {
                if let Some(items_schema) = items {
                    self.scan_inline_schema(items_schema, used);
                } else {
                    used.any = true;
                }
            }
            Some("object") | None => {
                if properties.is_none() {
                    match additional_properties {
                        Some(AdditionalProperties::Schema(ap)) => {
                            self.scan_inline_schema(ap, used);
                        }
                        _ => {
                            used.any = true;
                        }
                    }
                } else {
                    // Has properties inline → dict[str, Any] in type context
                    used.any = true;
                }
            }
            _ => {
                // Unrecognized type (e.g., "null") → Any
                used.any = true;
            }
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

                // Handle allOf composition
                if let Some(all_of_schemas) = all_of {
                    output.push_str(&format!("{}class {}(BaseModel):\n", self.indent(), name));
                    output.push_str(&format!(
                        "{}    # This model combines multiple schemas (allOf)\n",
                        self.indent()
                    ));

                    let has_aliases = all_of_schemas.iter().any(|s| {
                        if let Schema::Object {
                            properties: Some(props),
                            ..
                        } = s
                        {
                            props.keys().any(|k| to_snake_case(k) != *k)
                        } else {
                            false
                        }
                    });
                    if has_aliases {
                        self.write_model_config(&mut output);
                    }

                    for sub_schema in all_of_schemas.iter() {
                        if let Schema::Object {
                            properties: Some(props),
                            required,
                            ..
                        } = sub_schema
                        {
                            for (prop_name, prop_schema) in props {
                                let snake_name = to_snake_case(prop_name);
                                let alias = if snake_name != *prop_name {
                                    Some(prop_name.as_str())
                                } else {
                                    None
                                };
                                let base_type = self.schema_to_pydantic_type(prop_schema)?;
                                let is_required = required
                                    .as_ref()
                                    .map(|req| req.contains(prop_name))
                                    .unwrap_or(false);
                                let field_def = self.generate_field_definition(
                                    &snake_name,
                                    alias,
                                    prop_schema,
                                    base_type,
                                    is_required,
                                )?;
                                output.push_str(&format!("{}    {}\n", self.indent(), field_def));
                            }
                        } else if let Schema::Reference { reference, .. } = sub_schema {
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
                }

                // oneOf → type alias
                if let Some(one_of_schemas) = one_of {
                    let types: Result<Vec<String>, _> = one_of_schemas
                        .iter()
                        .filter(|s| s.get_type() != Some("null"))
                        .map(|s| self.schema_to_pydantic_type(s))
                        .collect();
                    let has_null = one_of_schemas.iter().any(|s| s.get_type() == Some("null"));
                    let mut types = types?;
                    if has_null && !types.iter().any(|t| t == "None") {
                        types.push("None".to_string());
                    }
                    output.push_str(&format!(
                        "{}{} = {}\n\n",
                        self.indent(),
                        name,
                        self.format_union(&types)
                    ));
                    return Ok(output);
                }

                // anyOf → type alias
                if let Some(any_of_schemas) = any_of {
                    let types: Result<Vec<String>, _> = any_of_schemas
                        .iter()
                        .filter(|s| s.get_type() != Some("null"))
                        .map(|s| self.schema_to_pydantic_type(s))
                        .collect();
                    let has_null = any_of_schemas.iter().any(|s| s.get_type() == Some("null"));
                    let mut types = types?;
                    if has_null && !types.iter().any(|t| t == "None") {
                        types.push("None".to_string());
                    }
                    output.push_str(&format!(
                        "{}{} = {}\n\n",
                        self.indent(),
                        name,
                        self.format_union(&types)
                    ));
                    return Ok(output);
                }

                // Map type (additionalProperties without properties)
                if properties.is_none()
                    && matches!(additional_properties, Some(AdditionalProperties::Schema(_)))
                {
                    let py_type = self.schema_to_pydantic_type(schema)?;
                    output.push_str(&format!("{}{} = {}\n\n", self.indent(), name, py_type));
                    return Ok(output);
                }

                // Object type → BaseModel class
                if schema_type_str(schema_type) == Some("object") || properties.is_some() {
                    output.push_str(&format!("{}class {}(BaseModel):\n", self.indent(), name));

                    if let Some(desc) = schema.get_description() {
                        if desc.contains('\n') {
                            output.push_str(&format!("{}    \"\"\"\n", self.indent()));
                            for line in desc.lines() {
                                if line.is_empty() {
                                    output.push_str(&format!("{}\n", self.indent()));
                                } else {
                                    output.push_str(&format!("{}    {}\n", self.indent(), line));
                                }
                            }
                            output.push_str(&format!("{}    \"\"\"\n", self.indent()));
                        } else {
                            output.push_str(&format!(
                                "{}    \"\"\"{}\"\"\"\n",
                                self.indent(),
                                desc
                            ));
                        }
                    }

                    if let Some(props) = properties {
                        if props.is_empty() {
                            output.push_str(&format!("{}    pass\n", self.indent()));
                        } else {
                            let has_aliases = props.keys().any(|k| to_snake_case(k) != *k);
                            if has_aliases {
                                self.write_model_config(&mut output);
                            }

                            for (prop_name, prop_schema) in props {
                                let snake_name = to_snake_case(prop_name);
                                let alias = if snake_name != *prop_name {
                                    Some(prop_name.as_str())
                                } else {
                                    None
                                };
                                let base_type = self.schema_to_pydantic_type(prop_schema)?;
                                let is_required = required
                                    .as_ref()
                                    .map(|req| req.contains(prop_name))
                                    .unwrap_or(false);
                                let field_def = self.generate_field_definition(
                                    &snake_name,
                                    alias,
                                    prop_schema,
                                    base_type,
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
                    // Primitive type alias
                    let py_type = self.schema_to_pydantic_type(schema)?;
                    output.push_str(&format!("{}{} = {}\n\n", self.indent(), name, py_type));
                }
            }
            Schema::Reference { .. } => {
                let py_type = self.schema_to_pydantic_type(schema)?;
                output.push_str(&format!("{}{} = {}\n\n", self.indent(), name, py_type));
            }
        }

        Ok(output)
    }

    /// Emit the model-level config that allows both field name and alias for population.
    fn write_model_config(&self, output: &mut String) {
        match self.version {
            PydanticVersion::V1 => {
                output.push_str(&format!("{}    class Config:\n", self.indent()));
                output.push_str(&format!(
                    "{}        allow_population_by_field_name = True\n\n",
                    self.indent()
                ));
            }
            PydanticVersion::V2 => {
                output.push_str(&format!(
                    "{}    model_config = ConfigDict(populate_by_name=True)\n\n",
                    self.indent()
                ));
            }
        }
    }

    fn generate_field_definition(
        &self,
        snake_name: &str,
        alias: Option<&str>,
        schema: &Schema,
        base_type: String,
        is_required: bool,
    ) -> Result<String> {
        let field_type = if !is_required && !self.is_optional_type(&base_type) {
            self.wrap_optional(&base_type)
        } else {
            base_type
        };

        let has_constraints = matches!(
            schema,
            Schema::Object {
                min_length: Some(_),
                ..
            }
        ) || matches!(
            schema,
            Schema::Object {
                max_length: Some(_),
                ..
            }
        ) || matches!(
            schema,
            Schema::Object {
                minimum: Some(_),
                ..
            }
        ) || matches!(
            schema,
            Schema::Object {
                maximum: Some(_),
                ..
            }
        ) || matches!(
            schema,
            Schema::Object {
                pattern: Some(_),
                ..
            }
        );

        let needs_field = alias.is_some() || has_constraints || schema.get_description().is_some();

        if !needs_field {
            if is_required {
                return Ok(format!("{snake_name}: {field_type}"));
            } else {
                return Ok(format!("{snake_name}: {field_type} = None"));
            }
        }

        let mut field_args: Vec<String> = Vec::new();

        // Default value for optional fields (first positional arg)
        if !is_required {
            field_args.push("None".to_string());
        }

        // Alias
        if let Some(original) = alias {
            field_args.push(format!("alias=\"{original}\""));
        }

        // Constraints
        if let Schema::Object {
            min_length,
            max_length,
            minimum,
            maximum,
            pattern,
            ..
        } = schema
        {
            if let Some(v) = min_length {
                field_args.push(format!("min_length={v}"));
            }
            if let Some(v) = max_length {
                field_args.push(format!("max_length={v}"));
            }
            if let Some(v) = minimum {
                field_args.push(format!("ge={v}"));
            }
            if let Some(v) = maximum {
                field_args.push(format!("le={v}"));
            }
            if let Some(regex) = pattern {
                let pattern_key = match self.version {
                    PydanticVersion::V1 => "regex",
                    PydanticVersion::V2 => "pattern",
                };
                field_args.push(format!("{pattern_key}=r\"{regex}\""));
            }
        }

        // Description
        if let Some(desc) = schema.get_description() {
            let escaped = desc
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            field_args.push(format!("description=\"{escaped}\""));
        }

        Ok(format!(
            "{snake_name}: {field_type} = Field({})",
            field_args.join(", ")
        ))
    }

    #[allow(clippy::only_used_in_recursion)]
    fn schema_to_pydantic_type(&self, schema: &Schema) -> Result<String> {
        match schema {
            Schema::Reference { reference, .. } => {
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

                if let Some(_all_of_schemas) = all_of {
                    py_type = self.format_dict("str", "Any");
                } else if let Some(one_of_schemas) = one_of {
                    let types: Result<Vec<String>, _> = one_of_schemas
                        .iter()
                        .filter(|s| s.get_type() != Some("null"))
                        .map(|s| self.schema_to_pydantic_type(s))
                        .collect();
                    let has_null = one_of_schemas.iter().any(|s| s.get_type() == Some("null"));
                    let mut types = types?;
                    if has_null && !types.iter().any(|t| t == "None") {
                        types.push("None".to_string());
                    }
                    py_type = self.format_union(&types);
                } else if let Some(any_of_schemas) = any_of {
                    let types: Result<Vec<String>, _> = any_of_schemas
                        .iter()
                        .filter(|s| s.get_type() != Some("null"))
                        .map(|s| self.schema_to_pydantic_type(s))
                        .collect();
                    let has_null = any_of_schemas.iter().any(|s| s.get_type() == Some("null"));
                    let mut types = types?;
                    if has_null && !types.iter().any(|t| t == "None") {
                        types.push("None".to_string());
                    }
                    py_type = self.format_union(&types);
                } else {
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

        output.push_str(&format!("# Generated by {command}\n"));
        output.push_str("# Do not modify manually\n");

        let Some(components) = &schema.components else {
            return Ok(output);
        };
        let Some(schemas) = &components.schemas else {
            return Ok(output);
        };

        let used = self.collect_imports(schemas);

        // Generate models in topological order (dependencies before dependents)
        let mut models_code = String::new();
        let sorted_names = topological_sort(schemas)?;
        for name in &sorted_names {
            if let Some(schema_def) = schemas.get(name) {
                let model = self.generate_model(name, schema_def)?;
                models_code.push_str(&model);
            }
        }

        // Build import lines, ruff/isort style:
        //   standard library first, then third-party (with blank line between groups)

        let mut stdlib_lines: Vec<String> = Vec::new();

        if used.date || used.datetime_ {
            let mut items = Vec::new();
            if used.date {
                items.push("date");
            }
            if used.datetime_ {
                items.push("datetime");
            }
            stdlib_lines.push(format!("from datetime import {}", items.join(", ")));
        }
        if used.enum_cls {
            stdlib_lines.push("from enum import Enum".to_string());
        }
        if used.any || used.literal {
            let mut items = Vec::new();
            if used.any {
                items.push("Any");
            }
            if used.literal {
                items.push("Literal");
            }
            stdlib_lines.push(format!("from typing import {}", items.join(", ")));
        }
        if used.uuid {
            stdlib_lines.push("from uuid import UUID".to_string());
        }

        let mut pydantic_items: Vec<&str> = vec!["BaseModel"];
        if used.config_dict {
            pydantic_items.push("ConfigDict");
        }
        if used.email_str {
            pydantic_items.push("EmailStr");
        }
        if used.field {
            pydantic_items.push("Field");
        }
        if used.http_url {
            pydantic_items.push("HttpUrl");
        }
        let pydantic_line = format!("from pydantic import {}", pydantic_items.join(", "));

        output.push('\n'); // blank line after header
        if !stdlib_lines.is_empty() {
            output.push_str(&stdlib_lines.join("\n"));
            output.push('\n');
            output.push('\n'); // blank line between stdlib and pydantic groups
        }
        output.push_str(&pydantic_line);
        output.push('\n');
        output.push('\n'); // blank line before models
        output.push_str(&models_code);

        Ok(output)
    }
}
