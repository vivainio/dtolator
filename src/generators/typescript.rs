use crate::generators::Generator;
use crate::generators::common;
use crate::generators::import_generator::ImportGenerator;
use crate::openapi::{
    AdditionalProperties, OpenApiSchema, Operation, Parameter, Schema, is_schema_nullable,
    schema_type_str,
};
use anyhow::Result;
use std::collections::HashSet;

pub struct TypeScriptGenerator {
    indent_level: usize,
}

impl Default for TypeScriptGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl TypeScriptGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    fn is_valid_identifier(name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        // Check if first character is a valid identifier start
        let first_char = name.chars().next().unwrap();
        if !first_char.is_alphabetic() && first_char != '_' && first_char != '$' {
            return false;
        }

        // Check if all characters are valid identifier characters
        name.chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '$')
    }

    fn quote_property_name(name: &str) -> String {
        if Self::is_valid_identifier(name) {
            name.to_string()
        } else {
            format!("\"{name}\"")
        }
    }

    fn sanitize_type_name(&self, name: &str) -> String {
        if Self::is_valid_identifier(name) {
            name.to_string()
        } else {
            // Convert special characters to PascalCase format
            let mut result = String::new();
            let mut capitalize_next = true;
            for ch in name.chars() {
                if ch.is_alphanumeric() {
                    if capitalize_next {
                        result.push(ch.to_uppercase().next().unwrap_or(ch));
                        capitalize_next = false;
                    } else {
                        result.push(ch);
                    }
                } else {
                    capitalize_next = true;
                }
            }
            if result.is_empty() {
                "Schema".to_string()
            } else if result.chars().next().unwrap().is_numeric() {
                format!("_{result}")
            } else {
                result
            }
        }
    }

    fn generate_interface(&self, name: &str, schema: &Schema) -> Result<String> {
        let mut output = String::new();

        if let Some(desc) = schema.get_description() {
            output.push_str(&format!("/** {desc} */\n"));
        }

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
                    output.push_str(&format!("export type {name} =\n"));
                    let enum_strings: Vec<String> = enum_vals
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("  | \"{s}\""))
                        .collect();
                    output.push_str(&enum_strings.join("\n"));
                    output.push_str(";\n\n");
                    return Ok(output);
                }

                // Handle composition types
                if let Some(all_of_schemas) = all_of {
                    output.push_str(&format!("export type {name} =\n"));
                    let types: Result<Vec<String>, _> = all_of_schemas
                        .iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    let type_list = types?;
                    for (i, type_str) in type_list.iter().enumerate() {
                        if i == 0 {
                            output.push_str(&format!("  {type_str}"));
                        } else {
                            output.push_str(&format!("\n  & {type_str}"));
                        }
                    }
                    output.push_str(";\n\n");
                    return Ok(output);
                } else if let Some(one_of_schemas) = one_of {
                    output.push_str(&format!("export type {name} =\n"));
                    let types: Result<Vec<String>, _> = one_of_schemas
                        .iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    let type_list = types?;
                    for (i, type_str) in type_list.iter().enumerate() {
                        if i == 0 {
                            output.push_str(&format!("  {type_str}"));
                        } else {
                            output.push_str(&format!("\n  | {type_str}"));
                        }
                    }
                    output.push_str(";\n\n");
                    return Ok(output);
                } else if let Some(any_of_schemas) = any_of {
                    output.push_str(&format!("export type {name} =\n"));
                    let types: Result<Vec<String>, _> = any_of_schemas
                        .iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    let type_list = types?;
                    for (i, type_str) in type_list.iter().enumerate() {
                        if i == 0 {
                            output.push_str(&format!("  {type_str}"));
                        } else {
                            output.push_str(&format!("\n  | {type_str}"));
                        }
                    }
                    output.push_str(";\n\n");
                    return Ok(output);
                }

                // Handle map types (additionalProperties without properties)
                if properties.is_none()
                    && matches!(additional_properties, Some(AdditionalProperties::Schema(_)))
                {
                    let map_type = self.schema_to_typescript(schema)?;
                    output.push_str(&format!("export type {name} = {map_type};\n\n"));
                    return Ok(output);
                }

                // Handle object types
                if schema_type_str(schema_type) == Some("object") || properties.is_some() {
                    let sanitized_name = self.sanitize_type_name(name);
                    output.push_str(&format!("export interface {sanitized_name} {{\n"));

                    if let Some(props) = properties {
                        for (prop_name, prop_schema) in props {
                            if let Some(desc) = prop_schema.get_description() {
                                output.push_str(&format!("  /** {desc} */\n"));
                            }
                            let prop_type = self.schema_to_typescript(prop_schema)?;
                            let is_required = required
                                .as_ref()
                                .map(|req| req.contains(prop_name))
                                .unwrap_or(false);

                            let optional_marker = if is_required { "" } else { "?" };
                            let quoted_name = Self::quote_property_name(prop_name);
                            output.push_str(&format!(
                                "  {quoted_name}{optional_marker}: {prop_type};\n"
                            ));
                        }
                    }

                    output.push_str("}\n\n");
                } else {
                    // Handle primitive type aliases
                    output.push_str(&format!(
                        "export type {name} = {};\n\n",
                        self.schema_to_typescript(schema)?
                    ));
                }
            }
            Schema::Reference { .. } => {
                // For references, create a type alias
                output.push_str(&format!(
                    "export type {name} = {};\n\n",
                    self.schema_to_typescript(schema)?
                ));
            }
        }

        Ok(output)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn schema_to_typescript(&self, schema: &Schema) -> Result<String> {
        match schema {
            Schema::Reference { reference } => {
                let ref_name = reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                let sanitized = self.sanitize_type_name(ref_name);
                Ok(sanitized)
            }
            Schema::Object {
                schema_type,
                properties,
                required,
                additional_properties,
                items,
                enum_values,
                nullable,
                all_of,
                one_of,
                any_of,
                ..
            } => {
                #[allow(unused_assignments)]
                let mut ts_type = String::new();

                // Handle composition types
                if let Some(all_of_schemas) = all_of {
                    let types: Result<Vec<String>, _> = all_of_schemas
                        .iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    ts_type = types?.join(" & ");
                } else if let Some(one_of_schemas) = one_of {
                    let types: Result<Vec<String>, _> = one_of_schemas
                        .iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    ts_type = types?.join(" | ");
                } else if let Some(any_of_schemas) = any_of {
                    let types: Result<Vec<String>, _> = any_of_schemas
                        .iter()
                        .map(|s| self.schema_to_typescript(s))
                        .collect();
                    ts_type = types?.join(" | ");
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
                                ts_type = format!("{item_type}[]");
                            } else {
                                ts_type = "unknown[]".to_string();
                            }
                        }
                        Some("object") | None => {
                            if properties.is_none() {
                                if let Some(AdditionalProperties::Schema(ap_schema)) =
                                    additional_properties
                                {
                                    let value_type = self.schema_to_typescript(ap_schema)?;
                                    ts_type = format!("Record<string, {value_type}>");
                                } else {
                                    ts_type = "Record<string, unknown>".to_string();
                                }
                            } else if let Some(props) = properties {
                                let mut object_props = Vec::new();
                                for (prop_name, prop_schema) in props {
                                    let prop_type = self.schema_to_typescript(prop_schema)?;
                                    let is_required = required
                                        .as_ref()
                                        .map(|req| req.contains(prop_name))
                                        .unwrap_or(false);

                                    let optional_marker = if is_required { "" } else { "?" };
                                    let quoted_name = Self::quote_property_name(prop_name);
                                    object_props.push(format!(
                                        "    {quoted_name}{optional_marker}: {prop_type}"
                                    ));
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
                if is_schema_nullable(nullable, schema_type) {
                    ts_type = format!("{ts_type} | null");
                }

                Ok(ts_type)
            }
        }
    }

    /// Generate dto.ts content for zod mode: only query/header param types (no request body
    /// interfaces, since those already exist as `z.infer` types in schema.ts).
    pub fn generate_with_imports(
        &self,
        schema: &OpenApiSchema,
        command_string: &str,
    ) -> Result<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!("// Generated by {command_string}\n"));
        output.push_str("// Do not modify manually\n\n");

        let query_param_types = self.generate_query_param_types(schema)?;

        // Collect schema types referenced by query/header parameters and import them
        let ref_types = self.collect_parameter_ref_types(schema);
        if !ref_types.is_empty() {
            let mut sorted: Vec<&String> = ref_types.iter().collect();
            sorted.sort();
            let mut import_gen = ImportGenerator::new();
            for name in sorted {
                import_gen.add_import("./schema", name, true);
            }
            output.push_str(&import_gen.generate());
            output.push('\n');
        }

        if !query_param_types.trim().is_empty() {
            output.push_str(&query_param_types);
        }

        // Remove trailing blank lines
        Ok(output.trim_end().to_string() + "\n")
    }

    pub fn generate_query_param_types(&self, schema: &OpenApiSchema) -> Result<String> {
        let mut types = String::new();
        let mut generated_types = HashSet::new();

        for operation in Self::all_operations(schema) {
            if let Some(parameters) = &operation.parameters {
                let query_params: Vec<&Parameter> = parameters
                    .iter()
                    .filter(|p| p.location == "query")
                    .collect();

                if !query_params.is_empty()
                    && let Some(type_name) = Self::query_param_type_name(operation)
                    && generated_types.insert(type_name.clone())
                {
                    if let Some(summary) = &operation.summary {
                        types.push_str(&format!("/**\n * Query parameters for {summary}\n */\n"));
                    }

                    let mandatory_params: Vec<&Parameter> = query_params
                        .iter()
                        .filter(|p| p.required.unwrap_or(false))
                        .cloned()
                        .collect();

                    let optional_params: Vec<&Parameter> = query_params
                        .iter()
                        .filter(|p| !p.required.unwrap_or(false))
                        .cloned()
                        .collect();

                    types.push_str(&format!("export type {type_name} = "));

                    if !mandatory_params.is_empty() {
                        types.push_str("{\n");
                        for param in &mandatory_params {
                            let param_type = self.parameter_type(param)?;
                            types.push_str(&format!("  {}: {};\n", param.name, param_type));
                        }
                        types.push('}');
                    }

                    if !optional_params.is_empty() {
                        if !mandatory_params.is_empty() {
                            types.push_str(" & ");
                        }
                        types.push_str("Partial<{\n");
                        for param in &optional_params {
                            let param_type = self.parameter_type(param)?;
                            types.push_str(&format!("  {}: {};\n", param.name, param_type));
                        }
                        types.push_str("}>");
                    }

                    types.push_str(";\n\n");
                }
            }
        }

        Ok(types)
    }

    pub fn generate_header_param_types(&self, schema: &OpenApiSchema) -> Result<String> {
        let mut types = String::new();
        let mut generated_types = HashSet::new();

        for operation in Self::all_operations(schema) {
            if let Some(parameters) = &operation.parameters {
                let header_params: Vec<&Parameter> = parameters
                    .iter()
                    .filter(|p| p.location == "header")
                    .collect();

                if !header_params.is_empty()
                    && let Some(type_name) = Self::header_param_type_name(operation)
                    && generated_types.insert(type_name.clone())
                {
                    if let Some(summary) = &operation.summary {
                        types.push_str(&format!("/**\n * Header parameters for {summary}\n */\n"));
                    }

                    types.push_str(&format!("export interface {type_name} {{\n"));
                    for param in &header_params {
                        let param_type = self.parameter_type(param)?;
                        let optional = if param.required.unwrap_or(false) {
                            ""
                        } else {
                            "?"
                        };

                        types.push_str(&format!(
                            "  \"{}\"{}: {};\n",
                            param.name, optional, param_type
                        ));
                    }
                    types.push_str("}\n\n");
                }
            }
        }

        Ok(types)
    }

    pub fn query_param_type_name(operation: &Operation) -> Option<String> {
        Self::param_type_name(operation, "QueryParams")
    }

    pub fn header_param_type_name(operation: &Operation) -> Option<String> {
        Self::param_type_name(operation, "Headers")
    }

    fn param_type_name(operation: &Operation, suffix: &str) -> Option<String> {
        operation.summary.as_ref().map(|summary| {
            let clean = summary
                .replace("Get ", "")
                .replace("Create ", "")
                .replace("Update ", "")
                .replace("Delete ", "")
                .replace("Retrieve ", "")
                .replace("Fetch ", "");
            let pascal = common::to_pascal_case(&clean);
            format!("{pascal}{suffix}")
        })
    }

    fn parameter_type(&self, parameter: &Parameter) -> Result<String> {
        if let Some(schema) = &parameter.schema {
            self.schema_to_typescript(schema)
        } else {
            Ok("unknown".to_string())
        }
    }

    fn all_operations(schema: &OpenApiSchema) -> Vec<&Operation> {
        let mut ops = Vec::new();
        if let Some(paths) = &schema.paths {
            for (_path, path_item) in paths {
                for op in [
                    &path_item.get,
                    &path_item.post,
                    &path_item.put,
                    &path_item.delete,
                    &path_item.patch,
                ]
                .into_iter()
                .flatten()
                {
                    ops.push(op);
                }
            }
        }
        ops
    }

    /// Collect schema `$ref` type names used by query and header parameters.
    fn collect_parameter_ref_types(&self, schema: &OpenApiSchema) -> HashSet<String> {
        let mut refs = HashSet::new();
        for operation in Self::all_operations(schema) {
            if let Some(parameters) = &operation.parameters {
                for param in parameters {
                    if (param.location == "query" || param.location == "header")
                        && let Some(param_schema) = &param.schema
                    {
                        self.collect_refs(param_schema, &mut refs);
                    }
                }
            }
        }
        refs
    }

    /// Recursively collect `$ref` schema names from a schema.
    fn collect_refs(&self, schema: &Schema, refs: &mut HashSet<String>) {
        match schema {
            Schema::Reference { reference } => {
                if let Some(name) = reference.strip_prefix("#/components/schemas/") {
                    refs.insert(name.to_string());
                }
            }
            Schema::Object {
                properties,
                items,
                all_of,
                one_of,
                any_of,
                additional_properties,
                ..
            } => {
                if let Some(props) = properties {
                    for prop in props.values() {
                        self.collect_refs(prop, refs);
                    }
                }
                if let Some(item) = items {
                    self.collect_refs(item, refs);
                }
                for vec in [all_of, one_of, any_of].into_iter().flatten() {
                    for s in vec {
                        self.collect_refs(s, refs);
                    }
                }
                if let Some(AdditionalProperties::Schema(s)) = additional_properties {
                    self.collect_refs(s, refs);
                }
            }
        }
    }
}

impl Generator for TypeScriptGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!("// Generated by {command}\n"));
        output.push_str("// Do not modify manually\n\n");

        if let Some(components) = &schema.components
            && let Some(schemas) = &components.schemas
            && !schemas.is_empty()
        {
            // Sort schemas topologically
            let sorted_names = common::topological_sort(schemas)?;

            // Generate interfaces
            for name in sorted_names {
                if let Some(schema_def) = schemas.get(&name) {
                    let interface = self.generate_interface(&name, schema_def)?;
                    output.push_str(&interface);
                }
            }
        }

        Ok(output.trim_end().to_string() + "\n")
    }
}

impl Clone for TypeScriptGenerator {
    fn clone(&self) -> Self {
        Self {
            indent_level: self.indent_level,
        }
    }
}
