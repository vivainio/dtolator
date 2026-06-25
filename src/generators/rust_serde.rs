use crate::generators::Generator;
use crate::generators::common;
use crate::generators::ir::{self, Backend, EnumValues, IrDecl, IrField, IrStruct, IrType, Scalar};
use crate::openapi::OpenApiSchema;
use anyhow::Result;

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
}

/// Renders the shared [`ir::IrDocument`] as Rust types with serde derives.
///
/// This is the first generator migrated onto the IR (see `ir.rs`): all
/// schema-walking now happens once in `ir::lower`, and this backend only decides
/// how the IR is spelled in Rust.
struct RustBackend;

impl RustBackend {
    /// Render an enum variant identifier from a wire value, e.g. `low-stock`
    /// or `low_stock` -> `LowStock`.
    fn enum_variant(value: &str) -> String {
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

    fn render_field(&self, field: &IrField, out: &mut String) {
        let field_name = common::to_snake_case(&field.wire_name);
        let base_type = self.type_ref(&field.ty);
        let final_type = if field.required {
            base_type
        } else {
            format!("Option<{}>", base_type)
        };

        if field.wire_name != field_name {
            out.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.wire_name));
        }
        out.push_str(&format!("    pub {}: {},\n", field_name, final_type));
    }

    fn render_struct(&self, s: &IrStruct, out: &mut String) {
        out.push_str("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]\n");
        out.push_str("#[serde(rename_all = \"camelCase\")]\n");
        out.push_str(&format!("pub struct {} {{\n", s.name));
        for field in &s.fields {
            self.render_field(field, out);
        }
        if s.has_unmodeled_variants {
            out.push_str("    // Note: oneOf/anyOf schemas are combined as optional fields\n");
        }
        out.push_str("}\n\n");
    }
}

impl Backend for RustBackend {
    fn type_ref(&self, ty: &IrType) -> String {
        match ty {
            // Rust models optionality via `Option<T>` driven by `required`, so a
            // nullable inner type is rendered transparently here.
            IrType::Optional(inner) => self.type_ref(inner),
            IrType::Named(name) => name.clone(),
            IrType::Array(inner) => format!("Vec<{}>", self.type_ref(inner)),
            IrType::Map(value) => {
                format!("std::collections::HashMap<String, {}>", self.type_ref(value))
            }
            // Inline string enums and unmodeled unions degrade to opaque values,
            // matching the original generator.
            IrType::Enum(_) => "String".to_string(),
            IrType::Union(_) => "serde_json::Value".to_string(),
            IrType::Scalar { kind, format } => match kind {
                Scalar::String => "String".to_string(),
                Scalar::Boolean => "bool".to_string(),
                Scalar::Integer => match format.as_deref() {
                    Some("int32") => "i32".to_string(),
                    _ => "i64".to_string(),
                },
                Scalar::Number => match format.as_deref() {
                    Some("float") => "f32".to_string(),
                    _ => "f64".to_string(),
                },
                Scalar::Object | Scalar::Free => "serde_json::Value".to_string(),
            },
        }
    }

    fn decl(&self, decl: &IrDecl, out: &mut String) {
        match decl {
            IrDecl::Struct(s) => self.render_struct(s, out),
            IrDecl::Enum { name, values, .. } => match values {
                // Integer enums degrade to a plain integer alias (matching the
                // original generator).
                EnumValues::Integers(_) => {
                    out.push_str(&format!("pub type {} = i64;\n\n", name));
                }
                EnumValues::Strings(members) => {
                    out.push_str("#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]\n");
                    out.push_str("#[serde(rename_all = \"snake_case\")]\n");
                    out.push_str(&format!("pub enum {} {{\n", name));
                    for member in members {
                        out.push_str(&format!("    #[serde(rename = \"{}\")]\n", member));
                        out.push_str(&format!("    {},\n", Self::enum_variant(member)));
                    }
                    out.push_str("}\n\n");
                }
            },
            IrDecl::Alias { name, ty, .. } => {
                out.push_str(&format!("pub type {} = {};\n\n", name, self.type_ref(ty)));
            }
        }
    }

    fn preamble(&self, _doc: &ir::IrDocument, out: &mut String) {
        out.push_str("use serde::{Deserialize, Serialize};\n\n");
    }
}

impl Generator for RustSerdeGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, _command: &str) -> Result<String> {
        // Rust has no forward-reference problem in a single file, but the
        // original generator emitted types in dependency order; preserve that.
        let document = ir::topologically_sorted(ir::lower(schema), schema);
        Ok(RustBackend.render(&document))
    }
}
