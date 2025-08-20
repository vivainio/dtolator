use crate::generators::Generator;
use crate::openapi::{OpenApiSchema, Schema};
use anyhow::Result;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct ZodGenerator {
    indent_level: usize,
}

impl ZodGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    fn indent(&self) -> String {
        "  ".repeat(self.indent_level)
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

    fn extract_type_name(&self, schema: &crate::openapi::Schema) -> Option<String> {
        match schema {
            crate::openapi::Schema::Reference { reference } => Some(
                reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference)
                    .to_string(),
            ),
            _ => None,
        }
    }

    fn generate_schema(&self, name: &str, schema: &Schema) -> Result<String> {
        let mut output = String::new();

        let schema_name = format!("{name}Schema");
        output.push_str(&format!("{}export const {} = ", self.indent(), schema_name));
        output.push_str(&self.schema_to_zod(schema)?);
        output.push_str(";\n\n");

        output.push_str(&format!(
            "{}export type {} = z.infer<typeof {}>;\n\n",
            self.indent(),
            name,
            schema_name
        ));

        Ok(output)
    }

    #[allow(clippy::only_used_in_recursion)]
    fn schema_to_zod(&self, schema: &Schema) -> Result<String> {
        match schema {
            Schema::Reference { reference } => {
                let ref_name = reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                Ok(format!("{ref_name}Schema"))
            }
            Schema::Object {
                schema_type,
                properties,
                required,
                items,
                enum_values,
                format,
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
                #[allow(unused_assignments)]
                let mut zod_schema = String::new();

                // Handle allOf, oneOf, anyOf
                if let Some(all_of_schemas) = all_of {
                    let schemas: Result<Vec<String>, _> = all_of_schemas
                        .iter()
                        .map(|s| self.schema_to_zod(s))
                        .collect();
                    zod_schema = format!("z.intersection({})", schemas?.join(", z.intersection("));
                } else if let Some(one_of_schemas) = one_of {
                    let schemas: Result<Vec<String>, _> = one_of_schemas
                        .iter()
                        .map(|s| self.schema_to_zod(s))
                        .collect();
                    zod_schema = format!("z.union([{}])", schemas?.join(", "));
                } else if let Some(any_of_schemas) = any_of {
                    let schemas: Result<Vec<String>, _> = any_of_schemas
                        .iter()
                        .map(|s| self.schema_to_zod(s))
                        .collect();
                    zod_schema = format!("z.union([{}])", schemas?.join(", "));
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
                                zod_schema = format!("z.enum([{}])", enum_strings.join(", "));
                            } else {
                                zod_schema = "z.string()".to_string();

                                // Handle format validations - special case for uuid which should use z.uuid() instead of deprecated z.string().uuid()
                                if let Some(fmt) = format {
                                    match fmt.as_str() {
                                        "uuid" => {
                                            zod_schema = "z.uuid()".to_string();
                                        }
                                        "email" => zod_schema.push_str(".email()"),
                                        "uri" => zod_schema.push_str(".url()"),
                                        "date" => zod_schema.push_str(".date()"),
                                        "date-time" => zod_schema.push_str(".datetime()"),
                                        _ => {}
                                    }
                                }

                                // Add length constraints
                                if let Some(min_len) = min_length {
                                    zod_schema.push_str(&format!(".min({min_len})"));
                                }
                                if let Some(max_len) = max_length {
                                    zod_schema.push_str(&format!(".max({max_len})"));
                                }

                                // Add pattern constraint
                                if let Some(pat) = pattern {
                                    zod_schema.push_str(&format!(".regex(new RegExp(\"{pat}\"))"));
                                }
                            }
                        }
                        Some("number") | Some("integer") => {
                            zod_schema = "z.number()".to_string();

                            if let Some(min) = minimum {
                                zod_schema.push_str(&format!(".min({min})"));
                            }
                            if let Some(max) = maximum {
                                zod_schema.push_str(&format!(".max({max})"));
                            }

                            if schema_type.as_deref() == Some("integer") {
                                zod_schema.push_str(".int()");
                            }
                        }
                        Some("boolean") => {
                            zod_schema = "z.boolean()".to_string();
                        }
                        Some("array") => {
                            if let Some(items_schema) = items {
                                let item_type = self.schema_to_zod(items_schema)?;
                                zod_schema = format!("z.array({item_type})");
                            } else {
                                zod_schema = "z.array(z.unknown())".to_string();
                            }
                        }
                        Some("object") | None => {
                            if let Some(props) = properties {
                                let mut object_props = Vec::new();
                                for (prop_name, prop_schema) in props {
                                    let prop_zod = self.schema_to_zod(prop_schema)?;
                                    let is_required = required
                                        .as_ref()
                                        .map(|req| req.contains(prop_name))
                                        .unwrap_or(false);

                                    let prop_def = if is_required {
                                        format!("  {prop_name}: {prop_zod}")
                                    } else {
                                        format!("  {prop_name}: {prop_zod}.optional()")
                                    };
                                    object_props.push(prop_def);
                                }

                                zod_schema =
                                    format!("z.object({{\n{}\n}})", object_props.join(",\n"));
                            } else {
                                zod_schema = "z.object({})".to_string();
                            }
                        }
                        _ => {
                            zod_schema = "z.unknown()".to_string();
                        }
                    }
                }

                // Apply nullable if needed
                if nullable.unwrap_or(false) {
                    zod_schema = format!("{zod_schema}.nullable()");
                }

                Ok(zod_schema)
            }
        }
    }
}

impl Generator for ZodGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!("// Generated by {command}\n"));
        output.push_str("// Do not modify manually\n\n");

        output.push_str("import { z } from \"zod\";\n\n");

        if let Some(components) = &schema.components {
            if let Some(schemas) = &components.schemas {
                if !schemas.is_empty() {
                    // Sort schemas topologically to handle dependencies
                    let sorted_names = self.topological_sort(schemas)?;

                    for name in sorted_names {
                        if let Some(schema_def) = schemas.get(&name) {
                            let zod_schema = self.generate_schema(&name, schema_def)?;
                            output.push_str(&zod_schema);
                        }
                    }
                }
            }
        }

        Ok(output)
    }
}

impl Clone for ZodGenerator {
    fn clone(&self) -> Self {
        Self {
            indent_level: self.indent_level,
        }
    }
}
