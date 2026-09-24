use crate::TsEnumStyle;
use crate::openapi::{AdditionalProperties, Schema, is_schema_nullable};
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
        .map(strip_punctuation)
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
        .map(strip_punctuation)
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

/// Convert a string to PascalCase by capitalizing the first letter of each word.
/// e.g. "All Users With Pagination" -> "AllUsersWithPagination"
pub fn to_pascal_case(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect()
}

/// Builder for TSDoc/JSDoc comment blocks.
///
/// Handles multiline text correctly for all sections: description, `@param`, `@returns`.
/// Used by all generators that produce TypeScript output.
///
/// # Example
/// ```text
/// /**
///  * Summary line
///  *
///  * Detailed description that may
///  * span multiple lines.
///  *
///  * @param id - User identifier
///  * @returns The user object
///  */
/// ```
pub struct TsDocBuilder {
    indent: String,
    /// Lines of body text (description, blank separators, @tags).
    lines: Vec<String>,
}

impl TsDocBuilder {
    pub fn new(indent: &str) -> Self {
        Self {
            indent: indent.to_string(),
            lines: Vec::new(),
        }
    }

    /// Append a block of text, properly wrapping each line.
    /// A blank line in the input becomes a ` *` separator.
    pub fn description(&mut self, text: &str) -> &mut Self {
        for line in text.lines() {
            if line.is_empty() {
                self.lines.push(String::new());
            } else {
                self.lines.push(line.to_string());
            }
        }
        self
    }

    /// Append a blank separator line (` *`).
    pub fn blank(&mut self) -> &mut Self {
        self.lines.push(String::new());
        self
    }

    /// Append a `@param name - description` tag.
    pub fn param(&mut self, name: &str, desc: &str) -> &mut Self {
        self.lines.push(format!("@param {name} - {desc}"));
        self
    }

    /// Append a `@returns description` tag.
    pub fn returns(&mut self, desc: &str) -> &mut Self {
        self.lines.push(format!("@returns {desc}"));
        self
    }

    /// Append an arbitrary pre-formatted line.
    pub fn raw(&mut self, line: &str) -> &mut Self {
        self.lines.push(line.to_string());
        self
    }

    /// Build the final comment string.
    pub fn build(&self) -> String {
        // Single-line shortcut: no tags, no newlines
        if self.lines.len() == 1 && !self.lines[0].starts_with('@') {
            return format!("{}/** {} */\n", self.indent, self.lines[0]);
        }

        let mut out = format!("{}/**\n", self.indent);
        for line in &self.lines {
            if line.is_empty() {
                out.push_str(&format!("{} *\n", self.indent));
            } else {
                out.push_str(&format!("{} * {line}\n", self.indent));
            }
        }
        out.push_str(&format!("{} */\n", self.indent));
        out
    }
}

/// Format a description as a JSDoc comment with proper multiline formatting.
/// Single-line descriptions produce `/** desc */`, multiline descriptions produce:
/// ```text
/// /**
///  * line 1
///  * line 2
///  */
/// ```
/// The `indent` parameter is prepended to each line.
pub fn format_jsdoc(description: &str, indent: &str) -> String {
    let mut doc = TsDocBuilder::new(indent);
    doc.description(description);
    doc.build()
}

/// Build a TypeScript enum member name from an enum value.
/// e.g. "in-progress" -> "InProgress", "IN_PROGRESS" -> "InProgress", -2 -> "ValueNeg2"
fn ts_enum_member_name(value: &serde_json::Value) -> Option<String> {
    let name = if let Some(s) = value.as_str() {
        let name: String = s
            .split(|c: char| !c.is_alphanumeric())
            .filter(|word| !word.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                let first = chars.next().unwrap_or_default();
                // Shouting words (IN_PROGRESS) become Pascal case; mixed case is kept
                let rest = if word.chars().any(|c| c.is_lowercase()) {
                    chars.as_str().to_string()
                } else {
                    chars.as_str().to_lowercase()
                };
                first.to_uppercase().collect::<String>() + &rest
            })
            .collect();
        if name.is_empty() {
            "Empty".to_string()
        } else if name.starts_with(|c: char| c.is_ascii_digit()) {
            format!("_{name}")
        } else {
            name
        }
    } else if value.is_number() {
        let n = value.to_string();
        let name = match n.strip_prefix('-') {
            Some(abs) => format!("ValueNeg{abs}"),
            None => format!("Value{n}"),
        };
        name.chars()
            .map(|c| if c.is_alphanumeric() { c } else { '_' })
            .collect()
    } else {
        return None;
    };
    Some(name)
}

/// Render a named top-level enum schema as a TypeScript `enum` or `as const` object.
///
/// Returns `None` when the schema should stay a literal union: union style, a
/// non-enum schema, a nullable enum (a declaration can't carry `null`), or values
/// other than strings and numbers.
pub fn render_ts_enum_decl(name: &str, schema: &Schema, style: TsEnumStyle) -> Option<String> {
    let Schema::Object {
        enum_values: Some(values),
        nullable,
        schema_type,
        ..
    } = schema
    else {
        return None;
    };
    if style == TsEnumStyle::Union || values.is_empty() || is_schema_nullable(nullable, schema_type)
    {
        return None;
    }

    let mut used = HashSet::new();
    let mut members = Vec::new();
    for value in values {
        let base = ts_enum_member_name(value)?;
        let mut member = base.clone();
        let mut n = 2;
        while !used.insert(member.clone()) {
            member = format!("{base}_{n}");
            n += 1;
        }
        members.push((member, value.to_string()));
    }

    let mut output = String::new();
    match style {
        TsEnumStyle::Enum => {
            output.push_str(&format!("export enum {name} {{\n"));
            for (member, literal) in members {
                output.push_str(&format!("  {member} = {literal},\n"));
            }
            output.push_str("}\n");
        }
        TsEnumStyle::Const => {
            output.push_str(&format!("export const {name} = {{\n"));
            for (member, literal) in members {
                output.push_str(&format!("  {member}: {literal},\n"));
            }
            output.push_str("} as const;\n");
        }
        TsEnumStyle::Union => unreachable!(),
    }
    Some(output)
}

/// Extract type name from a schema reference.
/// Returns the type name without the "#/components/schemas/" prefix.
pub fn extract_type_name(schema: &Schema) -> Option<String> {
    match schema {
        Schema::Reference { reference, .. } => Some(
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
        Schema::Reference { .. } => {
            if let Some(type_name) = extract_type_name(schema) {
                deps.insert(type_name);
            }
        }
        Schema::Object {
            properties,
            additional_properties,
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

            if let Some(AdditionalProperties::Schema(ap_schema)) = additional_properties {
                collect_dependencies_recursive(ap_schema, deps);
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
