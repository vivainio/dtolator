use crate::generators::Generator;
use crate::generators::common;
use crate::generators::import_generator::ImportGenerator;
use crate::openapi::{
    AdditionalProperties, OpenApiSchema, Schema, is_schema_nullable, schema_type_str,
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

    /// Generate TypeScript interfaces with imports from Zod schemas
    pub fn generate_with_imports(
        &self,
        schema: &OpenApiSchema,
        command_string: &str,
    ) -> Result<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!("// Generated by {command_string}\n"));
        output.push_str("// Do not modify manually\n\n");

        if let Some(components) = &schema.components
            && let Some(schemas) = &components.schemas
            && !schemas.is_empty()
        {
            let type_names: Vec<String> = schemas.keys().cloned().collect();

            // Collect actual request and response types from OpenAPI paths
            let (request_types_set, _response_types_set) =
                self.collect_request_and_response_types(schema);

            let request_types: Vec<String> = type_names
                .iter()
                .filter(|name| request_types_set.contains(*name))
                .cloned()
                .collect();

            let response_types: Vec<String> = type_names
                .iter()
                .filter(|name| !request_types_set.contains(*name))
                .cloned()
                .collect();

            // Import response types from schema.ts using 'import type'
            if !response_types.is_empty() {
                // Sort response types alphabetically
                let mut sorted_types = response_types.clone();
                sorted_types.sort();

                let mut import_gen = ImportGenerator::new();

                // Add type imports
                for name in &sorted_types {
                    import_gen.add_import("./schema", name, true);
                }

                // Add schema imports (runtime values)
                for name in &sorted_types {
                    import_gen.add_import("./schema", &format!("{name}Schema"), false);
                }

                output.push_str(&import_gen.generate());
                output.push('\n');
            }

            // Generate TypeScript interfaces for request types (direct interfaces, not z.infer)
            if !request_types.is_empty() {
                let ts_output = self.generate_with_command(schema, command_string)?;

                // Extract only request type interfaces from the TypeScript output
                let ts_lines: Vec<&str> = ts_output.lines().collect();
                let mut i = 0;
                while i < ts_lines.len() {
                    let line = ts_lines[i].trim();
                    if line.starts_with("export interface ") || line.starts_with("export type ") {
                        // Check if this is a request type
                        let mut is_request_type = false;
                        for request_type in &request_types {
                            if line.contains(&format!("interface {request_type}"))
                                || line.contains(&format!("type {request_type} "))
                            {
                                is_request_type = true;
                                break;
                            }
                        }

                        if is_request_type {
                            // Include this interface definition
                            let mut brace_count = 0;
                            let mut j = i;
                            while j < ts_lines.len() {
                                let current_line = ts_lines[j];
                                output.push_str(current_line);
                                output.push('\n');

                                // Count braces to know when interface ends
                                for ch in current_line.chars() {
                                    match ch {
                                        '{' => brace_count += 1,
                                        '}' => brace_count -= 1,
                                        _ => {}
                                    }
                                }

                                j += 1;
                                if brace_count == 0 && j > i {
                                    break;
                                }
                            }
                            i = j;
                        } else {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                output.push('\n');
            }

            // Generate query parameter types for Angular services
            use crate::generators::angular::AngularGenerator;
            let angular_generator = AngularGenerator::new();
            let query_param_types = angular_generator.generate_query_param_types(schema)?;
            if !query_param_types.trim().is_empty() {
                output.push_str(&query_param_types);
            }

            // Re-export types and schemas from schema.ts using export from syntax
            if !response_types.is_empty() {
                let mut export_gen = ImportGenerator::new();

                // Add type exports
                let response_types_vec: Vec<&str> =
                    response_types.iter().map(|s| s.as_str()).collect();
                export_gen.add_exports("./schema", response_types_vec.clone(), true);

                // Add schema exports (runtime values)
                let schema_names: Vec<String> = response_types
                    .iter()
                    .map(|name| format!("{name}Schema"))
                    .collect();
                let schema_names_refs: Vec<&str> =
                    schema_names.iter().map(|s| s.as_str()).collect();
                export_gen.add_exports("./schema", schema_names_refs, false);

                output.push_str(&export_gen.generate());
            }
        }

        // Remove trailing blank lines
        Ok(output.trim_end().to_string() + "\n")
    }

    /// Collect request and response types from OpenAPI paths
    pub fn collect_request_and_response_types(
        &self,
        schema: &OpenApiSchema,
    ) -> (HashSet<String>, HashSet<String>) {
        let mut request_types = HashSet::new();
        let mut response_types = HashSet::new();

        if let Some(paths) = &schema.paths {
            for (_path, path_item) in paths {
                // Check all HTTP methods
                let operations = [
                    &path_item.get,
                    &path_item.post,
                    &path_item.put,
                    &path_item.patch,
                    &path_item.delete,
                ];

                for operation in operations.into_iter().flatten() {
                    // Collect request body types
                    if let Some(request_body) = &operation.request_body
                        && let Some(content) = &request_body.content
                        && let Some(media_type) = content.get("application/json")
                        && let Some(schema_ref) = &media_type.schema
                        && let Some(type_name) = self.extract_type_name_from_schema(schema_ref)
                    {
                        request_types.insert(type_name);
                    }

                    // Collect response types
                    if let Some(responses) = &operation.responses {
                        for (_status, response) in responses {
                            if let Some(content) = &response.content
                                && let Some(media_type) = content.get("application/json")
                                && let Some(schema_ref) = &media_type.schema
                                && let Some(type_name) =
                                    self.extract_type_name_from_schema(schema_ref)
                            {
                                response_types.insert(type_name);
                            }
                        }
                    }
                }
            }
        }

        (request_types, response_types)
    }

    /// Extract type name from schema reference
    pub fn extract_type_name_from_schema(&self, schema: &Schema) -> Option<String> {
        match schema {
            Schema::Reference { reference } => Some(
                reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference)
                    .to_string(),
            ),
            _ => None,
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
