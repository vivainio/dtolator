use crate::generators::Generator;
use crate::openapi::{OpenApiSchema, Schema};
use anyhow::Result;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet, VecDeque};

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
    where
        F: FnOnce(&Self) -> String,
    {
        let mut generator = self.clone();
        generator.indent_level += 1;
        f(&generator)
    }

    // Collect dependencies for a schema
    fn collect_dependencies(&self, schema: &Schema) -> HashSet<String> {
        let mut deps = HashSet::new();
        self.collect_dependencies_recursive(schema, &mut deps);
        deps
    }

    fn collect_dependencies_recursive(&self, schema: &Schema, deps: &mut HashSet<String>) {
        match schema {
            Schema::Reference { reference } => {
                if let Some(type_name) = self.extract_type_name(schema) {
                    deps.insert(type_name);
                }
            }
            Schema::Object {
                properties,
                items,
                all_of,
                one_of,
                any_of,
                ..
            } => {
                // Collect dependencies from properties
                if let Some(props) = properties {
                    for (_, prop_schema) in props {
                        self.collect_dependencies_recursive(prop_schema, deps);
                    }
                }

                // Collect dependencies from array items
                if let Some(items_schema) = items {
                    self.collect_dependencies_recursive(items_schema, deps);
                }

                // Collect dependencies from composition schemas
                if let Some(schemas) = all_of {
                    for s in schemas {
                        self.collect_dependencies_recursive(s, deps);
                    }
                }
                if let Some(schemas) = one_of {
                    for s in schemas {
                        self.collect_dependencies_recursive(s, deps);
                    }
                }
                if let Some(schemas) = any_of {
                    for s in schemas {
                        self.collect_dependencies_recursive(s, deps);
                    }
                }
            }
        }
    }

    fn extract_type_name(&self, schema: &Schema) -> Option<String> {
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

    // Topological sort to order schemas by dependencies
    fn topological_sort(&self, schemas: &IndexMap<String, Schema>) -> Result<Vec<String>> {
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize graph and in-degree map
        for name in schemas.keys() {
            graph.insert(name.clone(), HashSet::new());
            in_degree.insert(name.clone(), 0);
        }

        // Build dependency graph
        for (name, schema) in schemas {
            let deps = self.collect_dependencies(schema);
            for dep in deps {
                if schemas.contains_key(&dep) {
                    graph.get_mut(&dep).unwrap().insert(name.clone());
                    *in_degree.get_mut(name).unwrap() += 1;
                }
            }
        }

        // Kahn's algorithm for topological sorting
        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        // Start with nodes that have no incoming edges (sorted for deterministic output)
        let mut zero_degree_nodes: Vec<_> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();
        zero_degree_nodes.sort();
        for name in zero_degree_nodes {
            queue.push_back(name);
        }

        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

            // Reduce in-degree for all dependent nodes (sorted for deterministic output)
            if let Some(dependents) = graph.get(&current) {
                let mut new_zero_degree: Vec<_> = dependents
                    .iter()
                    .filter_map(|dependent| {
                        let degree = in_degree.get_mut(dependent).unwrap();
                        *degree -= 1;
                        if *degree == 0 {
                            Some(dependent.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                new_zero_degree.sort();
                for dependent in new_zero_degree {
                    queue.push_back(dependent);
                }
            }
        }

        // Check for circular dependencies
        if result.len() != schemas.len() {
            return Err(anyhow::anyhow!("Circular dependency detected in schemas"));
        }

        Ok(result)
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

                // Handle object types
                if schema_type.as_deref() == Some("object") || properties.is_some() {
                    output.push_str(&format!("export interface {name} {{\n"));

                    if let Some(props) = properties {
                        for (prop_name, prop_schema) in props {
                            let prop_type = self.schema_to_typescript(prop_schema)?;
                            let is_required = required
                                .as_ref()
                                .map(|req| req.contains(prop_name))
                                .unwrap_or(false);

                            let optional_marker = if is_required { "" } else { "?" };
                            output.push_str(&format!(
                                "  {prop_name}{optional_marker}: {prop_type};\n"
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

    fn schema_to_typescript(&self, schema: &Schema) -> Result<String> {
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
                    match schema_type.as_deref() {
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
                            if let Some(props) = properties {
                                let mut object_props = Vec::new();
                                for (prop_name, prop_schema) in props {
                                    let prop_type = self.schema_to_typescript(prop_schema)?;
                                    let is_required = required
                                        .as_ref()
                                        .map(|req| req.contains(prop_name))
                                        .unwrap_or(false);

                                    let optional_marker = if is_required { "" } else { "?" };
                                    object_props.push(format!(
                                        "    {prop_name}{optional_marker}: {prop_type}"
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
                if nullable.unwrap_or(false) {
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

        if let Some(components) = &schema.components {
            if let Some(schemas) = &components.schemas {
                if !schemas.is_empty() {
                    let type_names: Vec<String> = schemas.keys().cloned().collect();

                    // Collect actual request and response types from OpenAPI paths
                    let (request_types_set, response_types_set) =
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

                    // Import only response schemas from schema.ts
                    if !response_types.is_empty() {
                        output.push_str("import {\n");

                        let mut import_lines = Vec::new();
                        for name in &response_types {
                            import_lines.push(format!("  {name}Schema,"));
                        }

                        output.push_str(&import_lines.join("\n"));
                        output.push_str("\n} from \"./schema\";\n");
                        output.push_str("import { z } from \"zod\";\n\n");
                    }

                    // Generate TypeScript interfaces for request types (direct interfaces, not z.infer)
                    if !request_types.is_empty() {
                        let ts_output = self.generate_with_command(schema, command_string)?;

                        // Extract only request type interfaces from the TypeScript output
                        let ts_lines: Vec<&str> = ts_output.lines().collect();
                        let mut i = 0;
                        while i < ts_lines.len() {
                            let line = ts_lines[i].trim();
                            if line.starts_with("export interface ")
                                || line.starts_with("export type ")
                            {
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

                    // Create and export inferred types from response schemas only
                    for name in &response_types {
                        output.push_str(&format!(
                            "export type {name} = z.infer<typeof {name}Schema>;\n"
                        ));
                    }
                    output.push('\n');

                    // Re-export only response schemas
                    for name in &response_types {
                        output.push_str(&format!("export {{ {name}Schema }};\n"));
                    }
                    output.push('\n');
                }
            }
        }

        Ok(output)
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
                    if let Some(request_body) = &operation.request_body {
                        if let Some(content) = &request_body.content {
                            if let Some(media_type) = content.get("application/json") {
                                if let Some(schema_ref) = &media_type.schema {
                                    if let Some(type_name) =
                                        self.extract_type_name_from_schema(schema_ref)
                                    {
                                        request_types.insert(type_name);
                                    }
                                }
                            }
                        }
                    }

                    // Collect response types
                    if let Some(responses) = &operation.responses {
                        for (_status, response) in responses {
                            if let Some(content) = &response.content {
                                if let Some(media_type) = content.get("application/json") {
                                    if let Some(schema_ref) = &media_type.schema {
                                        if let Some(type_name) =
                                            self.extract_type_name_from_schema(schema_ref)
                                        {
                                            response_types.insert(type_name);
                                        }
                                    }
                                }
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
    fn generate(&self, schema: &OpenApiSchema) -> Result<String> {
        self.generate_with_command(schema, "dtolator")
    }

    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!("// Generated by {command}\n"));
        output.push_str("// Do not modify manually\n\n");

        if let Some(components) = &schema.components {
            if let Some(schemas) = &components.schemas {
                if !schemas.is_empty() {
                    // Sort schemas topologically
                    let sorted_names = self.topological_sort(schemas)?;

                    // Generate interfaces
                    for name in sorted_names {
                        if let Some(schema_def) = schemas.get(&name) {
                            let interface = self.generate_interface(&name, schema_def)?;
                            output.push_str(&interface);
                        }
                    }
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
