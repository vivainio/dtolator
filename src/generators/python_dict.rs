use crate::generators::Generator;
use crate::generators::python_class::{PythonAttribute, PythonClassDef};
use crate::openapi::{
    AdditionalProperties, OpenApiSchema, Schema, is_schema_nullable, schema_type_str,
};
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
            Schema::Reference { reference, .. } => {
                if let Some(type_name) = reference.strip_prefix("#/components/schemas/")
                    && known_schemas.contains(type_name)
                {
                    deps.insert(type_name.to_string());
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
                if let Some(props) = properties {
                    for prop_schema in props.values() {
                        self.collect_dependencies_recursive(prop_schema, known_schemas, deps);
                    }
                }
                if let Some(item_schema) = items {
                    self.collect_dependencies_recursive(item_schema, known_schemas, deps);
                }
                if let Some(schemas) = all_of {
                    for s in schemas {
                        self.collect_dependencies_recursive(s, known_schemas, deps);
                    }
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
                additional_properties,
                enum_values,
                all_of,
                one_of,
                any_of,
                ..
            } => {
                // Handle enum types
                if let Some(enum_vals) = enum_values {
                    let all_int = enum_vals.iter().all(|v| v.is_i64() || v.is_u64());
                    let (base_classes, attributes) = if all_int {
                        let attributes = enum_vals
                            .iter()
                            .filter_map(|v| v.as_i64())
                            .map(|n| {
                                let member = if n >= 0 {
                                    format!("VALUE_{n}")
                                } else {
                                    format!("VALUE_NEG_{}", -n)
                                };
                                (member, PythonAttribute::assignment(n.to_string()))
                            })
                            .collect();
                        (vec!["IntEnum".to_string()], attributes)
                    } else {
                        let attributes = enum_vals
                            .iter()
                            .filter_map(|v| v.as_str())
                            .map(|val_str| {
                                let enum_name =
                                    val_str.to_uppercase().replace(" ", "_").replace("-", "_");
                                (
                                    enum_name,
                                    PythonAttribute::assignment(format!("\"{val_str}\"")),
                                )
                            })
                            .collect();
                        (vec!["str".to_string(), "Enum".to_string()], attributes)
                    };
                    let class = PythonClassDef {
                        name: name.to_string(),
                        base_classes,
                        attributes,
                        ..Default::default()
                    };
                    output.push_str(&class.render());
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

                // Handle allOf composition: `$ref` members become TypedDict base
                // classes (the inheritance the allOf expresses), inline object
                // members contribute their own fields. Required/optional fields are
                // split into a `*Required` base and a `total=False` subclass, the
                // same convention used for plain objects below.
                if let Some(all_of_schemas) = all_of {
                    let base_classes: Vec<String> = all_of_schemas
                        .iter()
                        .filter_map(|s| match s {
                            Schema::Reference { reference, .. } => Some(
                                reference
                                    .strip_prefix("#/components/schemas/")
                                    .unwrap_or(reference)
                                    .to_string(),
                            ),
                            _ => None,
                        })
                        .collect();
                    // A model with no `$ref` base still needs to extend `TypedDict`.
                    let bases = if base_classes.is_empty() {
                        vec!["TypedDict".to_string()]
                    } else {
                        base_classes
                    };

                    let mut required_props: Vec<(&String, String)> = Vec::new();
                    let mut optional_props: Vec<(&String, String)> = Vec::new();
                    for sub in all_of_schemas {
                        if let Schema::Object {
                            properties: Some(props),
                            required,
                            ..
                        } = sub
                        {
                            for (prop_name, prop_schema) in props {
                                let field_type = self.schema_to_python_type(prop_schema)?;
                                let is_required = required
                                    .as_ref()
                                    .map(|req| req.contains(prop_name))
                                    .unwrap_or(false);
                                if is_required {
                                    required_props.push((prop_name, field_type));
                                } else {
                                    optional_props.push((prop_name, field_type));
                                }
                            }
                        }
                    }

                    if !required_props.is_empty() && !optional_props.is_empty() {
                        // Required fields go in a base class; optional fields in a
                        // `total=False` subclass, matching the plain-object convention.
                        let required_name = format!("{name}Required");
                        let required_class =
                            typed_dict_class(required_name.clone(), bases, false, &required_props);
                        let full_class = typed_dict_class(
                            name.to_string(),
                            vec![required_name],
                            true,
                            &optional_props,
                        );
                        output.push_str(&required_class.render());
                        output.push_str("\n\n");
                        output.push_str(&full_class.render());
                    } else if !optional_props.is_empty() {
                        output.push_str(
                            &typed_dict_class(name.to_string(), bases, true, &optional_props)
                                .render(),
                        );
                    } else {
                        // Required-only fields, or a model with only base classes
                        // (which renders an empty `pass` body).
                        output.push_str(
                            &typed_dict_class(name.to_string(), bases, false, &required_props)
                                .render(),
                        );
                    }
                    output.push('\n');
                    return Ok(output);
                }

                // Handle map types (additionalProperties without properties)
                if properties.is_none()
                    && matches!(additional_properties, Some(AdditionalProperties::Schema(_)))
                {
                    let py_type = self.schema_to_python_type(schema)?;
                    output.push_str(&format!("{}{} = {}\n\n", self.indent(), name, py_type));
                    return Ok(output);
                }

                // Handle object types
                if schema_type_str(schema_type) == Some("object") || properties.is_some() {
                    let empty_required = vec![];
                    let required_fields: Vec<&String> = required
                        .as_ref()
                        .unwrap_or(&empty_required)
                        .iter()
                        .collect();

                    if let Some(props) = properties.as_ref().filter(|p| !p.is_empty()) {
                        let has_required = required_fields
                            .iter()
                            .any(|field| props.contains_key(*field));
                        let has_optional = props.keys().any(|key| !required_fields.contains(&key));

                        if has_required && has_optional {
                            // Required fields go in a base TypedDict; optional fields
                            // in a `total=False` subclass that inherits from it.
                            let mut required_props: Vec<(&String, String)> = Vec::new();
                            for field_name in &required_fields {
                                if let Some(field_schema) = props.get(*field_name) {
                                    required_props.push((
                                        *field_name,
                                        self.schema_to_python_type(field_schema)?,
                                    ));
                                }
                            }
                            let mut optional_props: Vec<(&String, String)> = Vec::new();
                            for (prop_name, prop_schema) in props {
                                if !required_fields.contains(&prop_name) {
                                    optional_props.push((
                                        prop_name,
                                        self.schema_to_python_type(prop_schema)?,
                                    ));
                                }
                            }

                            let required_name = format!("{name}Required");
                            output.push_str(
                                &typed_dict_class(
                                    required_name.clone(),
                                    vec!["TypedDict".to_string()],
                                    false,
                                    &required_props,
                                )
                                .render(),
                            );
                            output.push_str("\n\n");
                            output.push_str(
                                &typed_dict_class(
                                    name.to_string(),
                                    vec![required_name],
                                    true,
                                    &optional_props,
                                )
                                .render(),
                            );
                        } else {
                            // Every field is required, or every field is optional;
                            // a single class, `total=False` only in the latter case.
                            let mut fields: Vec<(&String, String)> = Vec::new();
                            for (prop_name, prop_schema) in props {
                                fields.push((prop_name, self.schema_to_python_type(prop_schema)?));
                            }
                            output.push_str(
                                &typed_dict_class(
                                    name.to_string(),
                                    vec!["TypedDict".to_string()],
                                    !has_required,
                                    &fields,
                                )
                                .render(),
                            );
                        }
                    } else {
                        // Empty or absent properties render an empty `pass` body.
                        output.push_str(
                            &typed_dict_class(
                                name.to_string(),
                                vec!["TypedDict".to_string()],
                                false,
                                &[],
                            )
                            .render(),
                        );
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
                properties,
                additional_properties,
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
                    return Ok(if is_schema_nullable(nullable, schema_type) {
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
                    return Ok(if is_schema_nullable(nullable, schema_type) {
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
                    return Ok(if is_schema_nullable(nullable, schema_type) {
                        format!("{literal_type} | None")
                    } else {
                        literal_type
                    });
                }

                let base_type = if let Some(type_str) = schema_type_str(schema_type) {
                    match type_str {
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
                                return Ok(if is_schema_nullable(nullable, schema_type) {
                                    format!("list[{item_type}] | None")
                                } else {
                                    format!("list[{item_type}]")
                                });
                            } else {
                                "list[Any]"
                            }
                        }
                        "object" => {
                            if properties.is_none()
                                && let Some(AdditionalProperties::Schema(ap_schema)) =
                                    additional_properties
                            {
                                let value_type = self.schema_to_python_type(ap_schema)?;
                                return Ok(if is_schema_nullable(nullable, schema_type) {
                                    format!("dict[str, {value_type}] | None")
                                } else {
                                    format!("dict[str, {value_type}]")
                                });
                            }
                            "dict[str, Any]"
                        }
                        _ => "Any",
                    }
                } else {
                    "Any"
                }
                .to_string();

                Ok(if is_schema_nullable(nullable, schema_type) {
                    format!("{base_type} | None")
                } else {
                    base_type
                })
            }
            Schema::Reference { reference, .. } => {
                let type_name = reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                Ok(type_name.to_string())
            }
        }
    }
}

/// Build a TypedDict class definition: bare `name: type` fields, with an
/// optional `total=False` marker on the header.
fn typed_dict_class(
    class_name: String,
    base_classes: Vec<String>,
    total_false: bool,
    fields: &[(&String, String)],
) -> PythonClassDef {
    let mut class = PythonClassDef::new(class_name, base_classes);
    if total_false {
        class
            .class_kwargs
            .insert("total".to_string(), "False".to_string());
    }
    for (prop_name, field_type) in fields {
        class.attributes.insert(
            (*prop_name).clone(),
            PythonAttribute::field(field_type.clone(), None),
        );
    }
    class
}

impl Generator for PythonDictGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        let mut output = String::new();

        // Add header comment
        output.push_str(&format!("# Generated by {command}\n"));
        output.push_str("# Do not modify manually\n\n");

        // Add imports (Python 3.10+ syntax)
        output.push_str("from typing import TypedDict, Literal, Any\n");
        output.push_str("from enum import Enum, IntEnum\n");
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
