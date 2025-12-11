use crate::generators::Generator;
use crate::openapi::{OpenApiSchema, Schema};
use anyhow::Result;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet, VecDeque};

pub struct RustSerdeGenerator;

impl Default for RustSerdeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl RustSerdeGenerator {
    pub fn new() -> Self {
        Self
    }

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
                if let Some(props) = properties {
                    for (_, prop_schema) in props {
                        self.collect_dependencies_recursive(prop_schema, deps);
                    }
                }

                if let Some(items_schema) = items {
                    self.collect_dependencies_recursive(items_schema, deps);
                }

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

    fn topological_sort(&self, schemas: &IndexMap<String, Schema>) -> Result<Vec<String>> {
        let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        for name in schemas.keys() {
            graph.insert(name.clone(), HashSet::new());
            in_degree.insert(name.clone(), 0);
        }

        for (name, schema) in schemas {
            let deps = self.collect_dependencies(schema);
            for dep in deps {
                if schemas.contains_key(&dep) {
                    graph.get_mut(&dep).unwrap().insert(name.clone());
                    *in_degree.get_mut(name).unwrap() += 1;
                }
            }
        }

        let mut queue = VecDeque::new();
        let mut result = Vec::new();

        let mut zero_degree_nodes: Vec<_> = in_degree
            .iter()
            .filter(|&(_, &degree)| degree == 0)
            .map(|(name, _)| name.clone())
            .collect();
        zero_degree_nodes.sort();
        for name in zero_degree_nodes {
            queue.push_back(name);
        }

        while let Some(current) = queue.pop_front() {
            result.push(current.clone());

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

        if result.len() != schemas.len() {
            return Err(anyhow::anyhow!("Circular dependency detected in schemas"));
        }

        Ok(result)
    }

    fn to_snake_case(&self, name: &str) -> String {
        let mut result = String::new();
        let mut prev_was_upper = false;

        for (i, ch) in name.chars().enumerate() {
            if ch.is_uppercase() {
                if i > 0 && !prev_was_upper {
                    result.push('_');
                }
                result.push(ch.to_lowercase().next().unwrap_or(ch));
                prev_was_upper = true;
            } else if ch == '-' || ch == ' ' {
                if !result.ends_with('_') {
                    result.push('_');
                }
                prev_was_upper = false;
            } else {
                result.push(ch);
                prev_was_upper = false;
            }
        }

        if result.is_empty() {
            "field".to_string()
        } else {
            result
        }
    }

    fn to_rust_type(&self, schema: &Schema) -> Result<String> {
        match schema {
            Schema::Reference { reference } => {
                let type_name = reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference)
                    .to_string();
                Ok(type_name)
            }
            Schema::Object {
                schema_type,
                format,
                enum_values,
                items,
                ..
            } => {
                if let Some(enum_vals) = enum_values {
                    if enum_vals.iter().all(|v| v.is_string()) {
                        return Ok("String".to_string());
                    }
                }

                match schema_type.as_deref() {
                    Some("string") => match format.as_deref() {
                        Some("date") => Ok("String".to_string()),
                        Some("date-time") => Ok("String".to_string()),
                        Some("uuid") => Ok("String".to_string()),
                        Some("email") => Ok("String".to_string()),
                        Some("uri") | Some("url") => Ok("String".to_string()),
                        _ => Ok("String".to_string()),
                    },
                    Some("integer") => match format.as_deref() {
                        Some("int32") => Ok("i32".to_string()),
                        Some("int64") => Ok("i64".to_string()),
                        _ => Ok("i64".to_string()),
                    },
                    Some("number") => match format.as_deref() {
                        Some("float") => Ok("f32".to_string()),
                        Some("double") => Ok("f64".to_string()),
                        _ => Ok("f64".to_string()),
                    },
                    Some("boolean") => Ok("bool".to_string()),
                    Some("array") => {
                        if let Some(items_schema) = items {
                            let item_type = self.to_rust_type(items_schema)?;
                            Ok(format!("Vec<{}>", item_type))
                        } else {
                            Ok("Vec<serde_json::Value>".to_string())
                        }
                    }
                    Some("object") => Ok("serde_json::Value".to_string()),
                    _ => Ok("serde_json::Value".to_string()),
                }
            }
        }
    }

    fn generate_struct(&self, name: &str, schema: &Schema) -> Result<String> {
        let mut output = String::new();

        match schema {
            Schema::Object {
                properties,
                required,
                enum_values,
                all_of,
                one_of,
                any_of,
                ..
            } => {
                if let Some(enum_vals) = enum_values {
                    output.push_str("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]\n");
                    output.push_str("#[serde(rename_all = \"snake_case\")]\n");
                    output.push_str(&format!("pub enum {} {{\n", name));

                    for enum_val in enum_vals {
                        if let Some(val_str) = enum_val.as_str() {
                            let variant_name = self.to_rust_enum_variant(val_str);
                            output.push_str(&format!("    #[serde(rename = \"{}\")]\n", val_str));
                            output.push_str(&format!("    {},\n", variant_name));
                        }
                    }

                    output.push_str("}\n\n");
                    return Ok(output);
                }

                output.push_str("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]\n");
                output.push_str("#[serde(rename_all = \"camelCase\")]\n");
                output.push_str(&format!("pub struct {} {{\n", name));

                if let Some(all_of_schemas) = all_of {
                    for schema in all_of_schemas.iter() {
                        if let Schema::Object {
                            properties: Some(props),
                            required: req,
                            ..
                        } = schema
                        {
                            for (prop_name, prop_schema) in props {
                                let field_name = self.to_snake_case(prop_name);
                                let field_type = self.to_rust_type(prop_schema)?;
                                let is_required =
                                    req.as_ref().map(|r| r.contains(prop_name)).unwrap_or(false);

                                let final_type = if !is_required {
                                    format!("Option<{}>", field_type)
                                } else {
                                    field_type
                                };

                                if prop_name != &field_name {
                                    output.push_str(&format!(
                                        "    #[serde(rename = \"{}\")]\n",
                                        prop_name
                                    ));
                                }
                                output.push_str(&format!(
                                    "    pub {}: {},\n",
                                    field_name, final_type
                                ));
                            }
                        }
                    }
                } else if let Some(props) = properties {
                    for (prop_name, prop_schema) in props {
                        let field_name = self.to_snake_case(prop_name);
                        let field_type = self.to_rust_type(prop_schema)?;
                        let is_required = required
                            .as_ref()
                            .map(|req| req.contains(prop_name))
                            .unwrap_or(false);

                        let final_type = if !is_required {
                            format!("Option<{}>", field_type)
                        } else {
                            field_type
                        };

                        if prop_name != &field_name {
                            output.push_str(&format!("    #[serde(rename = \"{}\")]\n", prop_name));
                        }
                        output.push_str(&format!("    pub {}: {},\n", field_name, final_type));
                    }
                }

                if one_of.is_some() || any_of.is_some() {
                    output.push_str(
                        "    // Note: oneOf/anyOf schemas are combined as optional fields\n",
                    );
                }

                output.push_str("}\n\n");
            }
            _ => {
                output.push_str(&format!("pub type {} = serde_json::Value;\n\n", name));
            }
        }

        Ok(output)
    }

    fn to_rust_enum_variant(&self, value: &str) -> String {
        let mut result = String::new();
        let mut capitalize_next = true;

        for ch in value.chars() {
            if ch.is_alphanumeric() {
                if capitalize_next {
                    result.push(ch.to_uppercase().next().unwrap_or(ch));
                    capitalize_next = false;
                } else {
                    result.push(ch);
                }
            } else if ch == '-' || ch == '_' || ch == ' ' {
                capitalize_next = true;
            }
        }

        if result.is_empty() {
            "Value".to_string()
        } else {
            result
        }
    }
}

impl Generator for RustSerdeGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, _command: &str) -> Result<String> {
        let mut output = String::new();

        output.push_str("use serde::{Deserialize, Serialize};\n\n");

        if let Some(components) = &schema.components {
            if let Some(schemas) = &components.schemas {
                let sorted_names = self.topological_sort(schemas)?;

                for name in sorted_names {
                    if let Some(schema) = schemas.get(&name) {
                        output.push_str(&self.generate_struct(&name, schema)?);
                    }
                }
            }
        }

        Ok(output)
    }
}
