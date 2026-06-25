use crate::generators::Generator;
use crate::generators::ir::{self, Backend, EnumValues, IrDecl, IrField, IrStruct, IrType, Scalar};
use crate::openapi::OpenApiSchema;
use anyhow::Result;

pub struct PythonDictGenerator;

impl Default for PythonDictGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PythonDictGenerator {
    pub fn new() -> Self {
        Self
    }
}

/// Renders the shared [`ir::IrDocument`] as `typing.TypedDict` classes. The
/// third generator migrated onto the IR.
struct PythonDictBackend {
    command: String,
}

impl PythonDictBackend {
    fn render_fields(&self, fields: &[&IrField], out: &mut String) {
        for field in fields {
            out.push_str(&format!("    {}: {}\n", field.wire_name, self.type_ref(&field.ty)));
        }
    }

    fn render_struct(&self, s: &IrStruct, out: &mut String) {
        if s.fields.is_empty() {
            out.push_str(&format!("class {}(TypedDict):\n", s.name));
            out.push_str("    pass\n\n");
            return;
        }

        let required: Vec<&IrField> = s.fields.iter().filter(|f| f.required).collect();
        let optional: Vec<&IrField> = s.fields.iter().filter(|f| !f.required).collect();

        match (required.is_empty(), optional.is_empty()) {
            // Mixed: a `*Required` base TypedDict plus a `total=False` subclass.
            (false, false) => {
                out.push_str(&format!("class {}Required(TypedDict):\n", s.name));
                self.render_fields(&required, out);
                out.push_str("\n\n");
                out.push_str(&format!("class {}({}Required, total=False):\n", s.name, s.name));
                self.render_fields(&optional, out);
                out.push('\n');
            }
            // All required.
            (false, true) => {
                out.push_str(&format!("class {}(TypedDict):\n", s.name));
                self.render_fields(&required, out);
                out.push('\n');
            }
            // All optional.
            (true, false) => {
                out.push_str(&format!("class {}(TypedDict, total=False):\n", s.name));
                self.render_fields(&optional, out);
                out.push('\n');
            }
            (true, true) => unreachable!("non-empty fields split into neither group"),
        }
    }
}

impl Backend for PythonDictBackend {
    fn type_ref(&self, ty: &IrType) -> String {
        match ty {
            // Python 3.10+ union syntax for nullability.
            IrType::Optional(inner) => format!("{} | None", self.type_ref(inner)),
            IrType::Named(name) => name.clone(),
            IrType::Array(inner) => format!("list[{}]", self.type_ref(inner)),
            IrType::Map(value) => format!("dict[str, {}]", self.type_ref(value)),
            IrType::Union(variants) => variants
                .iter()
                .map(|v| self.type_ref(v))
                .collect::<Vec<_>>()
                .join(" | "),
            IrType::Enum(members) => {
                let quoted: Vec<String> = members.iter().map(|m| format!("\"{m}\"")).collect();
                format!("Literal[{}]", quoted.join(", "))
            }
            IrType::Scalar { kind, .. } => match kind {
                Scalar::String => "str".to_string(),
                Scalar::Integer => "int".to_string(),
                Scalar::Number => "float".to_string(),
                Scalar::Boolean => "bool".to_string(),
                Scalar::Object => "dict[str, Any]".to_string(),
                Scalar::Free => "Any".to_string(),
            },
        }
    }

    fn decl(&self, decl: &IrDecl, out: &mut String) {
        match decl {
            IrDecl::Struct(s) => self.render_struct(s, out),
            IrDecl::Enum { name, values, .. } => {
                match values {
                    EnumValues::Integers(ints) => {
                        out.push_str(&format!("class {}(IntEnum):\n", name));
                        for n in ints {
                            let member = if *n >= 0 {
                                format!("VALUE_{n}")
                            } else {
                                format!("VALUE_NEG_{}", -n)
                            };
                            out.push_str(&format!("    {member} = {n}\n"));
                        }
                    }
                    EnumValues::Strings(members) => {
                        out.push_str(&format!("class {}(str, Enum):\n", name));
                        for member in members {
                            let upper = member.to_uppercase().replace([' ', '-'], "_");
                            out.push_str(&format!("    {upper} = \"{member}\"\n"));
                        }
                    }
                }
                out.push_str("\n\n");
            }
            IrDecl::Alias { name, ty, .. } => {
                out.push_str(&format!("{} = {}\n\n", name, self.type_ref(ty)));
            }
        }
    }

    fn preamble(&self, _doc: &ir::IrDocument, out: &mut String) {
        out.push_str(&format!("# Generated by {}\n", self.command));
        out.push_str("# Do not modify manually\n\n");
        out.push_str("from typing import TypedDict, Literal, Any\n");
        out.push_str("from enum import Enum, IntEnum\n");
        out.push_str("from datetime import datetime\n\n");
    }
}

impl Generator for PythonDictGenerator {
    fn generate_with_command(&self, schema: &OpenApiSchema, command: &str) -> Result<String> {
        // Preserve the original generator's depth-first dependency ordering.
        let document = ir::dfs_dependency_sorted(ir::lower(schema), schema);
        let backend = PythonDictBackend {
            command: command.to_string(),
        };
        Ok(backend.render(&document))
    }
}
