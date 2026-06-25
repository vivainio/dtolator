use crate::generators::Generator;
use crate::generators::ir::{self, Backend, EnumValues, IrDecl, IrField, IrStruct, IrType, Scalar};
use crate::openapi::OpenApiSchema;
use anyhow::Result;

pub struct DotNetGenerator;

impl Default for DotNetGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl DotNetGenerator {
    pub fn new() -> Self {
        Self
    }
}

/// Renders the shared [`ir::IrDocument`] as C# records/enums with
/// `System.Text.Json` attributes. The second generator migrated onto the IR
/// (after `rust_serde`): all schema-walking happens once in `ir::lower`.
struct DotNetBackend {
    command: String,
}

impl DotNetBackend {
    /// Uppercase the first character only, leaving the rest unchanged
    /// (`isActive` -> `IsActive`). Note this differs from `common::to_pascal_case`,
    /// which splits on whitespace.
    fn to_pascal_case(name: &str) -> String {
        let mut chars = name.chars();
        match chars.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        }
    }

    /// C# value types get a `?` suffix when nullable; reference types do not.
    fn is_value_type(cs_type: &str) -> bool {
        matches!(
            cs_type,
            "int"
                | "long"
                | "float"
                | "double"
                | "decimal"
                | "bool"
                | "byte"
                | "sbyte"
                | "short"
                | "ushort"
                | "uint"
                | "ulong"
                | "char"
                | "DateTime"
                | "DateTimeOffset"
                | "TimeSpan"
                | "Guid"
        )
    }

    fn summary(docs: Option<&String>, out: &mut String) {
        if let Some(desc) = docs {
            out.push_str("/// <summary>\n");
            for line in desc.lines() {
                out.push_str(&format!("/// {}\n", line));
            }
            out.push_str("/// </summary>\n");
        }
    }

    fn render_field(&self, field: &IrField, out: &mut String) {
        let pascal = Self::to_pascal_case(&field.wire_name);
        let cs_type = self.type_ref(&field.ty);

        out.push_str(&format!("    [JsonPropertyName(\"{}\")]\n", field.wire_name));
        if field.required {
            out.push_str(&format!(
                "    public required {} {} {{ get; set; }}\n",
                cs_type, pascal
            ));
        } else {
            // Non-required properties become nullable (unless already nullable
            // or a collection, which is non-null but possibly empty).
            let nullable_type = if !cs_type.ends_with('?') && !cs_type.starts_with("List<") {
                format!("{cs_type}?")
            } else {
                cs_type
            };
            out.push_str(&format!(
                "    public {} {} {{ get; set; }}\n",
                nullable_type, pascal
            ));
        }
    }

    fn render_struct(&self, s: &IrStruct, out: &mut String) {
        Self::summary(s.docs.as_ref(), out);
        out.push_str(&format!("public record {}\n", s.name));
        out.push_str("{\n");
        for field in &s.fields {
            self.render_field(field, out);
        }
        out.push_str("}\n\n");
    }
}

impl Backend for DotNetBackend {
    fn type_ref(&self, ty: &IrType) -> String {
        match ty {
            // Nullable value types take a `?`; reference types are already
            // nullable references and are left untouched here.
            IrType::Optional(inner) => {
                let base = self.type_ref(inner);
                if Self::is_value_type(&base) {
                    format!("{base}?")
                } else {
                    base
                }
            }
            IrType::Named(name) => name.clone(),
            IrType::Array(inner) => format!("List<{}>", self.type_ref(inner)),
            IrType::Map(value) => format!("Dictionary<string, {}>", self.type_ref(value)),
            IrType::Enum(_) => "string".to_string(),
            IrType::Union(_) => "object".to_string(),
            IrType::Scalar { kind, format } => match kind {
                Scalar::String => match format.as_deref() {
                    Some("uuid") => "Guid",
                    Some("date") => "DateOnly",
                    Some("date-time") => "DateTime",
                    Some("byte") => "byte[]",
                    _ => "string",
                }
                .to_string(),
                Scalar::Integer => match format.as_deref() {
                    Some("int64") => "long",
                    _ => "int",
                }
                .to_string(),
                Scalar::Number => match format.as_deref() {
                    Some("float") => "float",
                    Some("decimal") => "decimal",
                    _ => "double",
                }
                .to_string(),
                Scalar::Boolean => "bool".to_string(),
                Scalar::Object => "Dictionary<string, object>".to_string(),
                Scalar::Free => "object".to_string(),
            },
        }
    }

    fn decl(&self, decl: &IrDecl, out: &mut String) {
        match decl {
            IrDecl::Struct(s) => self.render_struct(s, out),
            IrDecl::Enum { name, docs, values } => {
                Self::summary(docs.as_ref(), out);
                out.push_str(&format!("public enum {}\n", name));
                out.push_str("{\n");
                // Only string members map to C# enum members (mirrors the
                // original generator, which skipped non-string enum values).
                if let EnumValues::Strings(members) = values {
                    for (i, member) in members.iter().enumerate() {
                        if i > 0 {
                            out.push_str(",\n");
                        }
                        out.push_str(&format!("    [JsonPropertyName(\"{}\")]\n", member));
                        out.push_str(&format!("    {}", Self::to_pascal_case(member)));
                    }
                }
                out.push_str("\n}\n\n");
            }
            IrDecl::Alias { name, ty, .. } => {
                out.push_str(&format!("// Type alias: {} = {}\n\n", name, self.type_ref(ty)));
            }
        }
    }

    fn preamble(&self, _doc: &ir::IrDocument, out: &mut String) {
        out.push_str(&format!("// Generated by {}\n", self.command));
        out.push_str("// Do not modify manually\n\n");
        out.push_str("using System.ComponentModel.DataAnnotations;\n");
        out.push_str("using System.Text.Json.Serialization;\n\n");
    }
}

impl Generator for DotNetGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        // C# emits all types in one namespace and tolerates forward references,
        // so source declaration order is preserved (no topological sort).
        let document = ir::lower(schema);
        let backend = DotNetBackend {
            command: command.to_string(),
        };
        Ok(backend.render(&document))
    }
}
