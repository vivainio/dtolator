use crate::generators::Generator;
use crate::generators::common::{collect_dependencies, extract_type_name, topological_sort};
use crate::openapi::{AdditionalProperties, OpenApiSchema, Operation, Schema, schema_type_str};
use anyhow::Result;
use std::collections::HashSet;

pub struct MarkdownGenerator;

impl Default for MarkdownGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownGenerator {
    pub fn new() -> Self {
        Self
    }

    fn schema_type_inline(&self, schema: &Schema) -> String {
        match schema {
            Schema::Reference { reference, .. } => {
                let name = reference
                    .strip_prefix("#/components/schemas/")
                    .unwrap_or(reference);
                name.to_string()
            }
            Schema::Object {
                schema_type,
                items,
                enum_values,
                additional_properties,
                all_of,
                one_of,
                any_of,
                nullable,
                ..
            } => {
                // Enum values inline
                if let Some(values) = enum_values {
                    let vals: Vec<String> = values
                        .iter()
                        .map(|v| match v {
                            serde_json::Value::String(s) => format!("\"{s}\""),
                            other => format!("{other}"),
                        })
                        .collect();
                    return vals.join(" | ");
                }

                // Composition
                if let Some(schemas) = all_of {
                    let parts: Vec<String> =
                        schemas.iter().map(|s| self.schema_type_inline(s)).collect();
                    return parts.join(" & ");
                }
                if let Some(schemas) = one_of {
                    let non_null: Vec<&Schema> = schemas
                        .iter()
                        .filter(|s| s.get_type() != Some("null"))
                        .collect();
                    if non_null.len() == 1 && schemas.len() == 2 {
                        // nullable oneOf pattern
                        let inner = self.schema_type_inline(non_null[0]);
                        return format!("{inner} | null");
                    }
                    let parts: Vec<String> =
                        schemas.iter().map(|s| self.schema_type_inline(s)).collect();
                    return parts.join(" | ");
                }
                if let Some(schemas) = any_of {
                    let non_null: Vec<&Schema> = schemas
                        .iter()
                        .filter(|s| s.get_type() != Some("null"))
                        .collect();
                    if non_null.len() == 1 && schemas.len() == 2 {
                        let inner = self.schema_type_inline(non_null[0]);
                        return format!("{inner} | null");
                    }
                    let parts: Vec<String> =
                        schemas.iter().map(|s| self.schema_type_inline(s)).collect();
                    return parts.join(" | ");
                }

                let base = match schema_type_str(schema_type) {
                    Some("array") => {
                        if let Some(item_schema) = items {
                            let inner = self.schema_type_inline(item_schema);
                            format!("{inner}[]")
                        } else {
                            "any[]".to_string()
                        }
                    }
                    Some("object") => {
                        if let Some(ap) = additional_properties {
                            match ap {
                                AdditionalProperties::Schema(s) => {
                                    let val = self.schema_type_inline(s);
                                    format!("Map<string, {val}>")
                                }
                                AdditionalProperties::Boolean(true) => {
                                    "Map<string, any>".to_string()
                                }
                                _ => "object".to_string(),
                            }
                        } else {
                            "object".to_string()
                        }
                    }
                    Some("string") => "string".to_string(),
                    Some("integer") => "integer".to_string(),
                    Some("number") => "number".to_string(),
                    Some("boolean") => "boolean".to_string(),
                    Some(other) => other.to_string(),
                    None => "any".to_string(),
                };

                if nullable == &Some(true) {
                    format!("{base} | null")
                } else {
                    base
                }
            }
        }
    }

    /// Extract format hint from a schema (e.g. "email", "uuid", "date-time").
    fn get_format(schema: &Schema) -> Option<&str> {
        match schema {
            Schema::Object { format, .. } => format.as_deref(),
            _ => None,
        }
    }

    /// Format a single field line for IDL output.
    /// Produces lines like: `  name?: string  // email, description`
    fn format_field(&self, name: &str, schema: &Schema, required: bool) -> String {
        let type_str = self.schema_type_inline(schema);
        let opt = if required { "" } else { "?" };
        let fmt = Self::get_format(schema);
        let desc = schema.get_description().map(|d| d.replace('\n', " "));

        // Build comment from format + description
        let comment = match (fmt, desc.as_deref()) {
            (Some(f), Some(d)) if !d.is_empty() => Some(format!("{f}, {d}")),
            (Some(f), _) => Some(f.to_string()),
            (_, Some(d)) if !d.is_empty() => Some(d.to_string()),
            _ => None,
        };

        match comment {
            Some(c) => format!("  {name}{opt}: {type_str}  // {c}\n"),
            None => format!("  {name}{opt}: {type_str}\n"),
        }
    }

    /// Emit properties as IDL fields inside a `{ }` block.
    fn emit_fields(
        &self,
        out: &mut String,
        props: &indexmap::IndexMap<String, Schema>,
        required: &std::collections::HashSet<&str>,
    ) {
        for (field_name, field_schema) in props {
            out.push_str(&self.format_field(
                field_name,
                field_schema,
                required.contains(field_name.as_str()),
            ));
        }
    }

    fn generate_schema_section(
        &self,
        name: &str,
        schema: &Schema,
        _all_schemas: &indexmap::IndexMap<String, Schema>,
    ) -> String {
        let mut out = String::new();

        // Description as a comment block before the type
        if let Some(desc) = schema.get_description() {
            for line in desc.lines() {
                if line.is_empty() {
                    out.push_str("//\n");
                } else {
                    out.push_str(&format!("// {line}\n"));
                }
            }
        }

        match schema {
            Schema::Object {
                enum_values,
                properties,
                all_of,
                one_of,
                any_of,
                ..
            } => {
                // Enum
                if let Some(values) = enum_values {
                    let vals: Vec<String> = values
                        .iter()
                        .map(|v| match v {
                            serde_json::Value::String(s) => format!("\"{s}\""),
                            other => format!("{other}"),
                        })
                        .collect();
                    out.push_str(&format!("enum {name} = {}\n\n", vals.join(" | ")));
                    return out;
                }

                // allOf composition
                if let Some(schemas) = all_of {
                    let refs: Vec<String> = schemas.iter().filter_map(extract_type_name).collect();
                    let extends = if refs.is_empty() {
                        String::new()
                    } else {
                        format!(" extends {}", refs.join(", "))
                    };

                    // Collect inline properties
                    let inline_props: Vec<_> = schemas
                        .iter()
                        .filter_map(|s| match s {
                            Schema::Object {
                                properties,
                                required,
                                ..
                            } => properties.as_ref().map(|p| (p, required.as_ref())),
                            _ => None,
                        })
                        .collect();

                    if inline_props.is_empty() {
                        out.push_str(&format!("type {name}{extends}\n\n"));
                    } else {
                        out.push_str(&format!("type {name}{extends} {{\n"));
                        for (props, required) in &inline_props {
                            let req_set: std::collections::HashSet<&str> = required
                                .map(|r| r.iter().map(|s| s.as_str()).collect())
                                .unwrap_or_default();
                            self.emit_fields(&mut out, props, &req_set);
                        }
                        out.push_str("}\n\n");
                    }
                    return out;
                }

                // oneOf / anyOf
                if let Some(schemas) = one_of {
                    let parts: Vec<String> =
                        schemas.iter().map(|s| self.schema_type_inline(s)).collect();
                    out.push_str(&format!("type {name} = {}\n\n", parts.join(" | ")));
                    return out;
                }
                if let Some(schemas) = any_of {
                    let parts: Vec<String> =
                        schemas.iter().map(|s| self.schema_type_inline(s)).collect();
                    out.push_str(&format!("type {name} = {}\n\n", parts.join(" | ")));
                    return out;
                }

                // Regular object with properties
                if let Some(props) = properties {
                    let required_set: std::collections::HashSet<&str> = match schema {
                        Schema::Object { required, .. } => required
                            .as_ref()
                            .map(|r| r.iter().map(|s| s.as_str()).collect())
                            .unwrap_or_default(),
                        _ => Default::default(),
                    };

                    out.push_str(&format!("type {name} {{\n"));
                    self.emit_fields(&mut out, props, &required_set);
                    out.push_str("}\n\n");
                } else {
                    // Type alias (e.g. array or map at top level)
                    let type_str = self.schema_type_inline(schema);
                    out.push_str(&format!("type {name} = {type_str}\n\n"));
                }
            }
            Schema::Reference { .. } => {
                out.push_str(&format!(
                    "type {name} = {}\n\n",
                    self.schema_type_inline(schema)
                ));
            }
        }

        out
    }

    /// Returns (summary_html, body_markdown) for an endpoint.
    /// The caller wraps these in a `<details>` block.
    fn generate_operation(
        &self,
        method: &str,
        path: &str,
        operation: &Operation,
    ) -> (String, String) {
        // Build the <summary> line: `GET` /path — Summary
        let mut summary_line = format!("<code>{method}</code> {path}");
        if let Some(summary) = &operation.summary {
            summary_line.push_str(&format!(" — <strong>{summary}</strong>"));
        }

        let mut out = String::new();

        // Description (full text) goes into the body
        if let Some(summary) = &operation.summary {
            if let Some(desc) = &operation.description
                && desc != summary
            {
                out.push_str(desc);
                out.push_str("\n\n");
            }
        } else if let Some(desc) = &operation.description {
            out.push_str(desc);
            out.push_str("\n\n");
        }

        // Parameters
        if let Some(params) = &operation.parameters {
            let has_params = params.iter().any(|p| p.location != "header");
            if has_params {
                out.push_str("**Parameters:**\n\n");
                out.push_str("| Name | In | Type | Required | Description |\n");
                out.push_str("|------|-----|------|----------|-------------|\n");
                for param in params {
                    if param.location == "header" {
                        continue; // skip headers for conciseness
                    }
                    let type_str = param
                        .schema
                        .as_ref()
                        .map(|s| self.schema_type_inline(s))
                        .unwrap_or_else(|| "any".to_string());
                    let required = if param.required.unwrap_or(false) {
                        "yes"
                    } else {
                        ""
                    };
                    out.push_str(&format!(
                        "| `{}` | {} | `{}` | {} |  |\n",
                        param.name, param.location, type_str, required
                    ));
                }
                out.push('\n');
            }
        }

        // Request body
        if let Some(request_body) = &operation.request_body
            && let Some(media_type) = request_body.content.get("application/json")
            && let Some(schema) = &media_type.schema
        {
            let type_str = self.schema_type_inline(schema);
            out.push_str(&format!("**Request body:** `{type_str}`\n\n"));
        }

        // Responses
        if let Some(responses) = &operation.responses {
            out.push_str("**Responses:**\n\n");
            for (status, response) in responses {
                let body_type = response
                    .content
                    .as_ref()
                    .and_then(|c| c.get("application/json"))
                    .and_then(|m| m.schema.as_ref())
                    .map(|s| self.schema_type_inline(s));

                match body_type {
                    Some(t) => out.push_str(&format!(
                        "- **{status}**: {} → `{t}`\n",
                        response.description
                    )),
                    None => out.push_str(&format!("- **{status}**: {}\n", response.description)),
                }
            }
            out.push('\n');
        }

        (summary_line, out)
    }
}

impl MarkdownGenerator {
    /// Collect top-level schema names directly referenced by an operation
    /// (request body, responses, parameter schemas).
    fn collect_operation_refs(operation: &Operation) -> Vec<String> {
        let mut refs = Vec::new();

        // Request body
        if let Some(rb) = &operation.request_body
            && let Some(mt) = rb.content.get("application/json")
            && let Some(s) = &mt.schema
            && let Some(name) = extract_type_name(s)
        {
            refs.push(name);
        }

        // Responses
        if let Some(responses) = &operation.responses {
            for (_status, response) in responses {
                if let Some(content) = &response.content
                    && let Some(mt) = content.get("application/json")
                    && let Some(s) = &mt.schema
                {
                    if let Some(name) = extract_type_name(s) {
                        refs.push(name);
                    }
                    // Also handle array of $ref
                    if let Schema::Object {
                        items: Some(item), ..
                    } = s
                        && let Some(name) = extract_type_name(item)
                    {
                        refs.push(name);
                    }
                }
            }
        }

        refs
    }

    /// Emit a schema and then its not-yet-emitted dependencies (breadth-first).
    /// Top-level type appears first, then the types it references, for reading order.
    fn emit_schema_with_deps(
        &self,
        name: &str,
        schemas: &indexmap::IndexMap<String, Schema>,
        emitted: &mut HashSet<String>,
    ) -> String {
        if emitted.contains(name) {
            return String::new();
        }
        let Some(schema) = schemas.get(name) else {
            return String::new();
        };

        // Emit this schema first (reader sees the top-level type immediately)
        let mut out = String::new();
        emitted.insert(name.to_string());
        out.push_str(&self.generate_schema_section(name, schema, schemas));

        // Then emit dependencies in breadth-first order
        let mut queue: std::collections::VecDeque<String> = std::collections::VecDeque::new();
        let deps = collect_dependencies(schema);
        let mut sorted_deps: Vec<&String> = deps.iter().collect();
        sorted_deps.sort();
        for dep in sorted_deps {
            if schemas.contains_key(dep.as_str()) && !emitted.contains(dep.as_str()) {
                queue.push_back(dep.clone());
            }
        }

        while let Some(dep_name) = queue.pop_front() {
            if emitted.contains(&dep_name) {
                continue;
            }
            let Some(dep_schema) = schemas.get(&dep_name) else {
                continue;
            };
            emitted.insert(dep_name.clone());
            out.push_str(&self.generate_schema_section(&dep_name, dep_schema, schemas));

            // Enqueue transitive deps
            let transitive = collect_dependencies(dep_schema);
            let mut sorted_transitive: Vec<&String> = transitive.iter().collect();
            sorted_transitive.sort();
            for t in sorted_transitive {
                if schemas.contains_key(t.as_str()) && !emitted.contains(t.as_str()) {
                    queue.push_back(t.clone());
                }
            }
        }
        out
    }
}

impl Generator for MarkdownGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, _command: &str) -> Result<String> {
        let mut output = String::new();
        let all_schemas = schema.components.as_ref().and_then(|c| c.schemas.as_ref());
        let mut emitted: HashSet<String> = HashSet::new();

        // Title
        output.push_str(&format!("# {}\n\n", schema.info.title));
        output.push_str(&format!("**Version:** {}\n\n", schema.info.version));
        if let Some(desc) = &schema.info.description {
            output.push_str(desc);
            output.push_str("\n\n");
        }

        // Endpoints with inline schemas
        if let Some(paths) = &schema.paths
            && !paths.is_empty()
        {
            // Group by tag, preserving order
            let mut tagged: indexmap::IndexMap<String, Vec<(&str, &str, &Operation)>> =
                indexmap::IndexMap::new();

            for (path, path_item) in paths {
                let methods: Vec<(&str, &Operation)> = [
                    ("GET", &path_item.get),
                    ("POST", &path_item.post),
                    ("PUT", &path_item.put),
                    ("PATCH", &path_item.patch),
                    ("DELETE", &path_item.delete),
                ]
                .into_iter()
                .filter_map(|(m, op)| op.as_ref().map(|o| (m, o)))
                .collect();

                for (method, operation) in methods {
                    let tag = operation
                        .tags
                        .as_ref()
                        .and_then(|t| t.first())
                        .map(|s| s.as_str())
                        .unwrap_or("Other");

                    tagged.entry(tag.to_string()).or_default().push((
                        method,
                        path.as_str(),
                        operation,
                    ));
                }
            }

            for (tag, endpoints) in &tagged {
                output.push_str(&format!("## {tag}\n\n"));

                for &(method, path, operation) in endpoints {
                    let (summary_line, body) = self.generate_operation(method, path, operation);

                    output.push_str(&format!("<details>\n<summary>{summary_line}</summary>\n\n"));
                    output.push_str(&body);

                    // Emit referenced schemas inline after this endpoint
                    if let Some(schemas) = all_schemas {
                        let refs = Self::collect_operation_refs(operation);
                        let mut idl_block = String::new();
                        for ref_name in &refs {
                            idl_block.push_str(&self.emit_schema_with_deps(
                                ref_name,
                                schemas,
                                &mut emitted,
                            ));
                        }
                        if !idl_block.is_empty() {
                            output.push_str("```typescript\n");
                            output.push_str(&idl_block);
                            output.push_str("```\n\n");
                        }
                    }

                    output.push_str("</details>\n\n");
                }
            }
        }

        // Remaining schemas not referenced by any endpoint
        if let Some(schemas) = all_schemas {
            let sorted = topological_sort(schemas)?;
            let mut orphan_block = String::new();
            for name in &sorted {
                orphan_block.push_str(&self.emit_schema_with_deps(name, schemas, &mut emitted));
            }
            if !orphan_block.is_empty() {
                output.push_str("## Other Schemas\n\n");
                output.push_str("```typescript\n");
                output.push_str(&orphan_block);
                output.push_str("```\n");
            }
        }

        Ok(output)
    }
}
