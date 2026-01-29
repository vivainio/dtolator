use crate::generators::Generator;
use crate::openapi::{OpenApiSchema, Schema};
use anyhow::Result;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet};

pub struct PythonDictGenerator {
    indent_level: usize,
}

impl Default for PythonDictGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonDictGenerator {
    pub fn new() -> Self {
        Self { indent_level: 0 }
    }

    fn indent(&self) -> String {
        "    ".repeat(self.indent_level)
    }

    /// Collect all schema references (dependencies) from a schema
    fn collect_dependencies(
        &self,
        schema: &Schema,
        known_schemas: &HashSet<&str>,
    ) -> HashSet<String> {
        let mut deps = HashSet::new();
        self.collect_dependencies_recursive(schema, known_schemas, &mut deps);
        deps
    }

    fn collect_dependencies_recursive(
        &self,
        schema: &Schema,
        known_schemas: &HashSet<&str>,
        deps: &mut HashSet<String>,
    ) {
        match schema {
            Schema::Reference { reference } => {
                if let Some(type_name) = reference.strip_prefix("#/components/schemas/")
                    && known_schemas.contains(type_name)
                {
                    deps.insert(type_name.to_string());
                }
            }
            Schema::Object {
                properties,
                items,
                one_of,
                any_of,
                ..
            } => {
                if let Some(props) = properties {
                    for prop_schema in props.values() {
                        self.collect_dependencies_recursive(prop_schema, known_schemas, deps);
                    }
                }
                if let Some(item_schema) = items {
                    self.collect_dependencies_recursive(item_schema, known_schemas, deps);
                }
                if let Some(schemas) = one_of {
                    for s in schemas {
                        self.collect_dependencies_recursive(s, known_schemas, deps);
                    }
                }
                if let Some(schemas) = any_of {
                    for s in schemas {
                        self.collect_dependencies_recursive(s, known_schemas, deps);
                    }
                }
            }
        }
    }

    /// Topologically sort schemas so dependencies come before dependents
    fn topological_sort<'a>(
        &self,
        schemas: &'a IndexMap<String, Schema>,
    ) -> Result<Vec<&'a String>> {
        let known_schemas: HashSet<&str> = schemas.keys().map(|s| s.as_str()).collect();

        // Build dependency graph
        let mut dependencies: HashMap<&String, HashSet<String>> = HashMap::new();
        for (name, schema) in schemas {
            dependencies.insert(name, self.collect_dependencies(schema, &known_schemas));
        }

        // DFS-based topological sort
        let mut result: Vec<&String> = Vec::new();
        let mut visited: HashSet<&String> = HashSet::new();
        let mut in_stack: HashSet<&String> = HashSet::new();

        fn visit<'a>(
            name: &'a String,
            dependencies: &HashMap<&'a String, HashSet<String>>,
            schemas: &'a IndexMap<String, Schema>,
            visited: &mut HashSet<&'a String>,
            in_stack: &mut HashSet<&'a String>,
            result: &mut Vec<&'a String>,
        ) -> Result<()> {
            if in_stack.contains(name) {
                // Circular dependency - skip to avoid infinite loop
                return Ok(());
            }
            if visited.contains(name) {
                return Ok(());
            }

            in_stack.insert(name);

            if let Some(deps) = dependencies.get(name) {
                // Sort dependencies for deterministic order
                let mut sorted_deps: Vec<&String> = deps.iter().collect();
                sorted_deps.sort();
                for dep in sorted_deps {
                    if let Some(dep_name) = schemas.keys().find(|k| *k == dep) {
                        visit(dep_name, dependencies, schemas, visited, in_stack, result)?;
                    }
                }
            }

            in_stack.remove(name);
            visited.insert(name);
            result.push(name);

            Ok(())
        }

        // Sort keys alphabetically for deterministic output order
        let mut sorted_keys: Vec<&String> = schemas.keys().collect();
        sorted_keys.sort();

        for name in sorted_keys {
            visit(
                name,
                &dependencies,
                schemas,
                &mut visited,
                &mut in_stack,
                &mut result,
            )?;
        }

        Ok(result)
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

                // Handle composition types (Python 3.10+ union syntax)
                if let Some(one_of_schemas) = one_of {
                    let types: Result<Vec<String>, _> = one_of_schemas
                        .iter()
                        .map(|s| self.schema_to_python_type(s))
                        .collect();
                    output.push_str(&format!(
                        "{}{} = {}\n\n",
                        self.indent(),
                        name,
                        types?.join(" | ")
                    ));
                    return Ok(output);
                } else if let Some(any_of_schemas) = any_of {
                    let types: Result<Vec<String>, _> = any_of_schemas
                        .iter()
                        .map(|s| self.schema_to_python_type(s))
                        .collect();
                    output.push_str(&format!(
                        "{}{} = {}\n\n",
                        self.indent(),
                        name,
                        types?.join(" | ")
                    ));
                    return Ok(output);
                }

                // Handle object types
                if schema_type.as_deref() == Some("object") || properties.is_some() {
                    let empty_required = vec![];
                    let required_fields: Vec<&String> = required
                        .as_ref()
                        .unwrap_or(&empty_required)
                        .iter()
                        .collect();

                    if let Some(props) = properties {
                        if props.is_empty() {
                            output.push_str(&format!(
                                "{}class {}(TypedDict):\n",
                                self.indent(),
                                name
                            ));
                            output.push_str(&format!("{}    pass\n", self.indent()));
                        } else {
                            let has_required = required_fields
                                .iter()
                                .any(|field| props.contains_key(*field));
                            let has_optional =
                                props.keys().any(|key| !required_fields.contains(&key));

                            if has_required && has_optional {
                                // Create base TypedDict for required fields
                                output.push_str(&format!(
                                    "{}class {}Required(TypedDict):\n",
                                    self.indent(),
                                    name
                                ));
                                for field_name in &required_fields {
                                    if let Some(field_schema) = props.get(*field_name) {
                                        let field_type =
                                            self.schema_to_python_type(field_schema)?;
                                        output.push_str(&format!(
                                            "{}    {}: {}\n",
                                            self.indent(),
                                            field_name,
                                            field_type
                                        ));
                                    }
                                }
                                output.push_str("\n\n");

                                // Create full TypedDict that inherits from required and adds optional fields
                                output.push_str(&format!(
                                    "{}class {}({}Required, total=False):\n",
                                    self.indent(),
                                    name,
                                    name
                                ));
                                for (prop_name, prop_schema) in props {
                                    if !required_fields.contains(&prop_name) {
                                        let field_type = self.schema_to_python_type(prop_schema)?;
                                        output.push_str(&format!(
                                            "{}    {}: {}\n",
                                            self.indent(),
                                            prop_name,
                                            field_type
                                        ));
                                    }
                                }
                            } else if has_required {
                                // Only required fields
                                output.push_str(&format!(
                                    "{}class {}(TypedDict):\n",
                                    self.indent(),
                                    name
                                ));
                                for (prop_name, prop_schema) in props {
                                    let field_type = self.schema_to_python_type(prop_schema)?;
                                    output.push_str(&format!(
                                        "{}    {}: {}\n",
                                        self.indent(),
                                        prop_name,
                                        field_type
                                    ));
                                }
                            } else {
                                // Only optional fields
                                output.push_str(&format!(
                                    "{}class {}(TypedDict, total=False):\n",
                                    self.indent(),
                                    name
                                ));
                                for (prop_name, prop_schema) in props {
                                    let field_type = self.schema_to_python_type(prop_schema)?;
                                    output.push_str(&format!(
                                        "{}    {}: {}\n",
                                        self.indent(),
                                        prop_name,
                                        field_type
                                    ));
                                }
                            }
                        }
                    } else {
                        output.push_str(&format!("{}class {}(TypedDict):\n", self.indent(), name));
                        output.push_str(&format!("{}    pass\n", self.indent()));
                    }

                    output.push('\n');
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

    #[allow(clippy::only_used_in_recursion)]
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
                // Handle composition types (Python 3.10+ union syntax)
                if let Some(one_of_schemas) = one_of {
                    let types: Result<Vec<String>, _> = one_of_schemas
                        .iter()
                        .map(|s| self.schema_to_python_type(s))
                        .collect();
                    let union_type = types?.join(" | ");
                    return Ok(if nullable.unwrap_or(false) {
                        format!("{union_type} | None")
                    } else {
                        union_type
                    });
                } else if let Some(any_of_schemas) = any_of {
                    let types: Result<Vec<String>, _> = any_of_schemas
                        .iter()
                        .map(|s| self.schema_to_python_type(s))
                        .collect();
                    let union_type = types?.join(" | ");
                    return Ok(if nullable.unwrap_or(false) {
                        format!("{union_type} | None")
                    } else {
                        union_type
                    });
                }

                // Handle enum values
                if let Some(enum_vals) = enum_values
                    && enum_vals.iter().all(|v| v.is_string())
                {
                    let values: Vec<String> = enum_vals
                        .iter()
                        .filter_map(|v| v.as_str())
                        .map(|s| format!("\"{s}\""))
                        .collect();
                    let literal_type = format!("Literal[{}]", values.join(", "));
                    return Ok(if nullable.unwrap_or(false) {
                        format!("{literal_type} | None")
                    } else {
                        literal_type
                    });
                }

                let base_type = if let Some(type_str) = schema_type {
                    match type_str.as_str() {
                        "string" => match format.as_deref() {
                            Some("email") => "str",
                            Some("uri") | Some("url") => "str",
                            Some("uuid") => "str",
                            Some("date") => "str",
                            Some("date-time") => "str",
                            _ => "str",
                        },
                        "integer" => "int",
                        "number" => "float",
                        "boolean" => "bool",
                        "array" => {
                            if let Some(item_schema) = items {
                                let item_type = self.schema_to_python_type(item_schema)?;
                                return Ok(if nullable.unwrap_or(false) {
                                    format!("list[{item_type}] | None")
                                } else {
                                    format!("list[{item_type}]")
                                });
                            } else {
                                "list[Any]"
                            }
                        }
                        "object" => "dict[str, Any]",
                        _ => "Any",
                    }
                } else {
                    "Any"
                }
                .to_string();

                Ok(if nullable.unwrap_or(false) {
                    format!("{base_type} | None")
                } else {
                    base_type
                })
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

impl Generator for PythonDictGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!("# Generated by {command}\n"));
        output.push_str("# Do not modify manually\n\n");

        // Add imports (Python 3.10+ syntax)
        output.push_str("from typing import TypedDict, Literal, Any\n");
        output.push_str("from enum import Enum\n");
        output.push_str("from datetime import datetime\n\n");

        if let Some(components) = &schema.components
            && let Some(schemas) = &components.schemas
        {
            // Sort schemas topologically so dependencies come first
            let sorted_names = self.topological_sort(schemas)?;
            for name in sorted_names {
                if let Some(schema_def) = schemas.get(name) {
                    let typed_dict = self.generate_typed_dict(name, schema_def)?;
                    output.push_str(&typed_dict);
                }
            }
        }

        Ok(output)
    }
}

impl Clone for PythonDictGenerator {
    fn clone(&self) -> Self {
        Self {
            indent_level: self.indent_level,
        }
    }
}
