use crate::generators::Generator;
use crate::openapi::{OpenApiSchema, Schema};
use anyhow::Result;
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct JsonSchemaGenerator {
    indent_level: usize,
}

impl Default for JsonSchemaGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonSchemaGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    // Collect dependencies for a schema
    fn collect_dependencies(&self, schema: &Schema) -> HashSet<String> {
        let mut deps = HashSet::new();
        self.collect_dependencies_recursive(schema, &mut deps);
        deps
    }

    fn collect_dependencies_recursive(&self, schema: &Schema, deps: &mut HashSet<String>) {
        match schema {
            Schema::Reference { reference: _ } => {
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

    #[allow(clippy::only_used_in_recursion)]
    fn schema_to_json_schema(&self, schema: &Schema) -> Result<Value> {
        match schema {
            Schema::Reference { reference } => {
                let type_name = reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                Ok(json!({
                    "$ref": format!("#/$defs/{}", type_name)
                }))
            }
            Schema::Object {
                schema_type,
                properties,
                required,
                items,
                enum_values,
                format,
                description,
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
                let mut json_schema = serde_json::Map::new();

                // Handle composition schemas
                if let Some(all_of_schemas) = all_of {
                    let mut all_of_json = Vec::new();
                    for s in all_of_schemas {
                        all_of_json.push(self.schema_to_json_schema(s)?);
                    }
                    json_schema.insert("allOf".to_string(), Value::Array(all_of_json));
                    if let Some(desc) = description {
                        json_schema.insert("description".to_string(), Value::String(desc.clone()));
                    }
                    return Ok(Value::Object(json_schema));
                }

                if let Some(one_of_schemas) = one_of {
                    let mut one_of_json = Vec::new();
                    for s in one_of_schemas {
                        one_of_json.push(self.schema_to_json_schema(s)?);
                    }
                    json_schema.insert("oneOf".to_string(), Value::Array(one_of_json));
                    if let Some(desc) = description {
                        json_schema.insert("description".to_string(), Value::String(desc.clone()));
                    }
                    return Ok(Value::Object(json_schema));
                }

                if let Some(any_of_schemas) = any_of {
                    let mut any_of_json = Vec::new();
                    for s in any_of_schemas {
                        any_of_json.push(self.schema_to_json_schema(s)?);
                    }
                    json_schema.insert("anyOf".to_string(), Value::Array(any_of_json));
                    if let Some(desc) = description {
                        json_schema.insert("description".to_string(), Value::String(desc.clone()));
                    }
                    return Ok(Value::Object(json_schema));
                }

                // Handle enum values
                if let Some(enum_vals) = enum_values {
                    json_schema.insert("enum".to_string(), Value::Array(enum_vals.clone()));
                    if let Some(desc) = description {
                        json_schema.insert("description".to_string(), Value::String(desc.clone()));
                    }
                    return Ok(Value::Object(json_schema));
                }

                // Handle type
                if let Some(type_str) = schema_type {
                    json_schema.insert("type".to_string(), Value::String(type_str.clone()));
                }

                // Handle object properties
                if let Some(props) = properties {
                    let mut properties_json = serde_json::Map::new();
                    for (key, prop_schema) in props {
                        properties_json
                            .insert(key.clone(), self.schema_to_json_schema(prop_schema)?);
                    }
                    json_schema.insert("properties".to_string(), Value::Object(properties_json));

                    if let Some(req) = required {
                        if !req.is_empty() {
                            json_schema.insert(
                                "required".to_string(),
                                Value::Array(
                                    req.iter().map(|r| Value::String(r.clone())).collect(),
                                ),
                            );
                        }
                    }

                    json_schema.insert("additionalProperties".to_string(), Value::Bool(false));
                }

                // Handle array items
                if let Some(items_schema) = items {
                    json_schema.insert(
                        "items".to_string(),
                        self.schema_to_json_schema(items_schema)?,
                    );
                }

                // Handle format
                if let Some(fmt) = format {
                    json_schema.insert("format".to_string(), Value::String(fmt.clone()));
                }

                // Handle description
                if let Some(desc) = description {
                    json_schema.insert("description".to_string(), Value::String(desc.clone()));
                }

                // Handle nullable
                if let Some(nullable_val) = nullable {
                    if *nullable_val {
                        if let Some(existing_type) = json_schema.get("type") {
                            json_schema
                                .insert("type".to_string(), json!([existing_type.clone(), "null"]));
                        }
                    }
                }

                // Handle number constraints
                if let Some(min) = minimum {
                    json_schema.insert("minimum".to_string(), json!(min));
                }
                if let Some(max) = maximum {
                    json_schema.insert("maximum".to_string(), json!(max));
                }

                // Handle string constraints
                if let Some(min_len) = min_length {
                    json_schema.insert("minLength".to_string(), json!(min_len));
                }
                if let Some(max_len) = max_length {
                    json_schema.insert("maxLength".to_string(), json!(max_len));
                }
                if let Some(pat) = pattern {
                    json_schema.insert("pattern".to_string(), Value::String(pat.clone()));
                }

                Ok(Value::Object(json_schema))
            }
        }
    }

    fn generate_full_schema(&self, schema: &OpenApiSchema) -> Result<Value> {
        let mut json_schema = serde_json::Map::new();

        // JSON Schema metadata
        json_schema.insert(
            "$schema".to_string(),
            Value::String("https://json-schema.org/draft/2020-12/schema".to_string()),
        );

        // Add title and description from OpenAPI info
        if let Some(info) = &schema.info {
            json_schema.insert("title".to_string(), Value::String(info.title.clone()));
            if let Some(desc) = &info.description {
                json_schema.insert("description".to_string(), Value::String(desc.clone()));
            }
        }

        // Process schemas
        if let Some(components) = &schema.components {
            if let Some(schemas) = &components.schemas {
                if !schemas.is_empty() {
                    // Create $defs section for reusable schemas
                    let mut defs = serde_json::Map::new();

                    // Sort schemas by dependencies
                    let sorted_names = self.topological_sort(schemas)?;

                    for name in sorted_names {
                        if let Some(schema_def) = schemas.get(&name) {
                            defs.insert(name.clone(), self.schema_to_json_schema(schema_def)?);
                        }
                    }

                    json_schema.insert("$defs".to_string(), Value::Object(defs));

                    // If there's a root schema, make it the main schema
                    if let Some(root_schema) = schemas.get("Root") {
                        let root_json = self.schema_to_json_schema(root_schema)?;
                        if let Value::Object(root_obj) = root_json {
                            for (key, value) in root_obj {
                                if key != "$ref" {
                                    json_schema.insert(key, value);
                                }
                            }
                        }
                    } else {
                        // If no root schema, reference the first schema
                        if let Some(first_name) = schemas.keys().next() {
                            json_schema.insert(
                                "$ref".to_string(),
                                Value::String(format!("#/$defs/{first_name}")),
                            );
                        }
                    }
                }
            }
        }

        Ok(Value::Object(json_schema))
    }
}

impl Generator for JsonSchemaGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, _command: &str) -> Result<String> {
        let json_schema = self.generate_full_schema(schema)?;
        // JSON doesn't support comments, so we output valid JSON only
        let output = serde_json::to_string_pretty(&json_schema)?;
        Ok(output)
    }
}

impl Clone for JsonSchemaGenerator {
    fn clone(&self) -> Self {
        Self {
            indent_level: self.indent_level,
        }
    }
}
