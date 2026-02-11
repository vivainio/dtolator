use crate::openapi::Schema;
use anyhow::Result;
use indexmap::IndexMap;
use std::collections::{HashMap, HashSet, VecDeque};

/// Strip non-alphanumeric characters from a word, keeping only letters and digits.
fn strip_punctuation(word: &str) -> String {
    word.chars().filter(|c| c.is_alphanumeric()).collect()
}

/// Convert a summary string to camelCase, stripping punctuation from each word.
/// e.g. "Get user's data." -> "getUsersData"
pub fn summary_to_camel_case(summary: &str) -> String {
    summary
        .split_whitespace()
        .map(|word| strip_punctuation(word))
        .filter(|word| !word.is_empty())
        .enumerate()
        .map(|(i, word)| {
            if i == 0 {
                word.to_lowercase()
            } else {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            }
        })
        .collect::<String>()
}

/// Convert a summary string to PascalCase, stripping punctuation from each word.
/// e.g. "Push message." -> "PushMessage"
pub fn summary_to_pascal_case(summary: &str) -> String {
    summary
        .split_whitespace()
        .map(|word| strip_punctuation(word))
        .filter(|word| !word.is_empty())
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Extract type name from a schema reference.
/// Returns the type name without the "#/components/schemas/" prefix.
pub fn extract_type_name(schema: &Schema) -> Option<String> {
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

/// Recursively collect all type dependencies from a schema.
pub fn collect_dependencies_recursive(schema: &Schema, deps: &mut HashSet<String>) {
    match schema {
        Schema::Reference { reference: _ } => {
            if let Some(type_name) = extract_type_name(schema) {
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
                    collect_dependencies_recursive(prop_schema, deps);
                }
            }

            if let Some(items_schema) = items {
                collect_dependencies_recursive(items_schema, deps);
            }

            if let Some(schemas) = all_of {
                for s in schemas {
                    collect_dependencies_recursive(s, deps);
                }
            }
            if let Some(schemas) = one_of {
                for s in schemas {
                    collect_dependencies_recursive(s, deps);
                }
            }
            if let Some(schemas) = any_of {
                for s in schemas {
                    collect_dependencies_recursive(s, deps);
                }
            }
        }
    }
}

/// Collect all dependencies for a schema.
pub fn collect_dependencies(schema: &Schema) -> HashSet<String> {
    let mut deps = HashSet::new();
    collect_dependencies_recursive(schema, &mut deps);
    deps
}

/// Topologically sort schemas using Kahn's algorithm.
/// Returns schema names ordered so that dependencies come before dependents.
pub fn topological_sort(schemas: &IndexMap<String, Schema>) -> Result<Vec<String>> {
    let mut graph: HashMap<String, HashSet<String>> = HashMap::new();
    let mut in_degree: HashMap<String, usize> = HashMap::new();

    for name in schemas.keys() {
        graph.insert(name.clone(), HashSet::new());
        in_degree.insert(name.clone(), 0);
    }

    for (name, schema) in schemas {
        let deps = collect_dependencies(schema);
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
