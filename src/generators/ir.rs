//! Intermediate representation (IR) for code generation — PROPOSAL / FOR DISCUSSION.
//!
//! Today every generator under `src/generators/` consumes `&OpenApiSchema`
//! directly and re-implements the same schema walking in slightly different
//! ways (`$ref` resolution, nullable/optional handling, inline-object
//! extraction, `oneOf`/`anyOf`/`allOf`, topological sorting). Bugs fixed in one
//! generator do not get fixed in the others.
//!
//! This module proposes the classic compiler frontend/backend split:
//!
//! ```text
//!   OpenApiSchema --lower()--> IrDocument --Backend--> target source
//! ```
//!
//! `lower()` runs ONCE and absorbs all the gnarly OpenAPI-specific logic. Each
//! backend then renders a clean, target-agnostic IR and never sees a `Schema`.
//!
//! Nothing is wired into the existing generators yet — this is here to react to
//! in review. See the PR description for scope, migration path, and open
//! questions.

use crate::openapi::{OpenApiSchema, Schema, SchemaType};
use indexmap::IndexMap;

// ---------------------------------------------------------------------------
// The IR
// ---------------------------------------------------------------------------

/// A scalar type, independent of any target language.
///
/// Each backend maps these to its own spelling, e.g. `Prim::Int` becomes
/// `number` (TS), `int` (Python/C#), `i64` (Rust), `z.number().int()` (Zod).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Prim {
    String,
    Int,
    Float,
    Bool,
    /// `format: binary` / byte payloads.
    Bytes,
    /// `format: date-time` / `date`.
    DateTime,
    /// No constraints — `any`/`object`/`unknown` depending on backend.
    Any,
}

/// A target-agnostic type. No OpenAPI concepts leak past lowering: nullability,
/// `$ref`, and inline objects are all already resolved into these variants.
#[derive(Debug, Clone)]
pub enum IrType {
    Prim(Prim),
    /// Reference to a top-level model by name (already normalized).
    Named(String),
    Array(Box<IrType>),
    /// `additionalProperties` — a string-keyed map of the inner type.
    Map(Box<IrType>),
    /// `nullable` / not-`required` collapsed into one place.
    Optional(Box<IrType>),
    /// `oneOf` / `anyOf`.
    Union(Vec<IrType>),
    /// An anonymous inline object. Lowering MAY instead hoist these into named
    /// `IrModel`s — see the open question in the PR description.
    Object(IrObject),
    /// A closed set of string values.
    Enum(Vec<String>),
}

/// An object shape (fields + their types).
#[derive(Debug, Clone, Default)]
pub struct IrObject {
    pub fields: Vec<IrField>,
}

/// One field of an object/model.
#[derive(Debug, Clone)]
pub struct IrField {
    /// Name as it appears in the wire format (the JSON key).
    pub wire_name: String,
    pub ty: IrType,
    /// Whether the field is required. Note `Optional` (nullable) is a property
    /// of `ty`; this is the separate "may be absent" axis.
    pub required: bool,
    pub docs: Option<String>,
}

/// A named, top-level model (what becomes a `interface`/`class`/`struct`).
#[derive(Debug, Clone)]
pub struct IrModel {
    pub name: String,
    pub object: IrObject,
    pub docs: Option<String>,
}

/// The whole lowered document handed to a backend.
#[derive(Debug, Clone, Default)]
pub struct IrDocument {
    /// Topologically sorted so a backend can emit top-to-bottom without
    /// forward references. (Lowering owns the sort that today lives in
    /// `common.rs` and a duplicate in `python_dict.rs`.)
    pub models: Vec<IrModel>,
    // pub endpoints: Vec<IrEndpoint>,  // follow-up: lower paths/operations
}

// ---------------------------------------------------------------------------
// Backend trait — a thin convention for the type-emitting generators.
// ---------------------------------------------------------------------------

/// Renders an [`IrDocument`] to a target language.
///
/// Deliberately NOT a per-node visitor (`render_array`, `render_optional`, …):
/// that fights the borrow checker and forces awkward state threading. A single
/// `type_ref` that recurses internally matches how the generators already work.
pub trait Backend {
    /// Render a type in reference position, e.g. the right-hand side of a field
    /// declaration: `string | null`, `Optional[str]`, `z.string().nullable()`.
    fn type_ref(&self, ty: &IrType) -> String;

    /// Render one named model into `w`.
    fn model(&self, model: &IrModel, w: &mut CodeWriter);

    /// Imports / file header. Default: nothing.
    fn preamble(&self, _doc: &IrDocument, _w: &mut CodeWriter) {}

    /// Drive the whole document. Most backends won't need to override this.
    fn render(&self, doc: &IrDocument) -> String {
        let mut w = CodeWriter::new(self.indent_unit());
        self.preamble(doc, &mut w);
        for model in &doc.models {
            self.model(model, &mut w);
        }
        w.finish()
    }

    /// The indentation unit for this language ("  " or "    ").
    fn indent_unit(&self) -> &'static str {
        "  "
    }
}

// ---------------------------------------------------------------------------
// Shared writer — replaces the per-generator `indent_level` + `repeat` dance.
// ---------------------------------------------------------------------------

/// Accumulates source text with managed indentation. Replaces the
/// `indent_level: usize` + `"  ".repeat(n)` pattern copied across generators.
pub struct CodeWriter {
    buf: String,
    unit: &'static str,
    level: usize,
}

impl CodeWriter {
    pub fn new(unit: &'static str) -> Self {
        Self {
            buf: String::new(),
            unit,
            level: 0,
        }
    }

    pub fn indent(&mut self) {
        self.level += 1;
    }

    pub fn dedent(&mut self) {
        self.level = self.level.saturating_sub(1);
    }

    /// Write one indented line (no trailing newline needed in `text`).
    pub fn line(&mut self, text: &str) {
        if text.is_empty() {
            self.buf.push('\n');
            return;
        }
        for _ in 0..self.level {
            self.buf.push_str(self.unit);
        }
        self.buf.push_str(text);
        self.buf.push('\n');
    }

    /// A blank separator line.
    pub fn blank(&mut self) {
        self.buf.push('\n');
    }

    pub fn finish(self) -> String {
        self.buf
    }
}

// ---------------------------------------------------------------------------
// Lowering: OpenApiSchema -> IrDocument
// ---------------------------------------------------------------------------

/// Lower a parsed OpenAPI document into the IR.
///
/// Partial implementation covering the common object/field cases — enough to
/// show the shape against the real `Schema` type. `allOf` merging, full
/// `oneOf`/`anyOf` discrimination, and path/operation lowering are intentionally
/// left as follow-ups (see PR open questions).
pub fn lower(schema: &OpenApiSchema) -> IrDocument {
    let mut models = Vec::new();

    if let Some(components) = &schema.components
        && let Some(schemas) = &components.schemas
    {
        for (name, def) in schemas {
            models.push(lower_model(name, def));
        }
        // NOTE: topological sort over `models` would happen here, reusing the
        // single Kahn's-algorithm implementation from `common.rs`.
    }

    IrDocument { models }
}

fn lower_model(name: &str, schema: &Schema) -> IrModel {
    let docs = schema.get_description().map(str::to_string);
    let object = match schema {
        Schema::Object {
            properties,
            required,
            ..
        } => lower_object(properties.as_ref(), required.as_ref()),
        // A top-level `$ref` alias or scalar newtype: model with no fields for
        // now; a real impl would carry a type alias variant.
        Schema::Reference { .. } => IrObject::default(),
    };
    IrModel {
        name: name.to_string(),
        object,
        docs,
    }
}

fn lower_object(
    properties: Option<&IndexMap<String, Schema>>,
    required: Option<&Vec<String>>,
) -> IrObject {
    let mut fields = Vec::new();
    if let Some(props) = properties {
        for (field_name, field_schema) in props {
            let is_required = required.is_some_and(|r| r.iter().any(|n| n == field_name));
            let mut ty = lower_type(field_schema);
            // Fold nullability into the type.
            if field_schema.is_nullable() {
                ty = IrType::Optional(Box::new(ty));
            }
            fields.push(IrField {
                wire_name: field_name.clone(),
                ty,
                required: is_required,
                docs: field_schema.get_description().map(str::to_string),
            });
        }
    }
    IrObject { fields }
}

/// Map a single `Schema` node to an `IrType`. This is the logic currently
/// duplicated as `schema_to_typescript` / `schema_to_zod` / `schema_to_python_type`
/// / etc. — here it lives exactly once.
fn lower_type(schema: &Schema) -> IrType {
    match schema {
        Schema::Reference { reference, .. } => IrType::Named(ref_name(reference)),
        Schema::Object {
            schema_type,
            items,
            enum_values,
            additional_properties,
            one_of,
            any_of,
            format,
            ..
        } => {
            if let Some(variants) = one_of.as_ref().or(any_of.as_ref()) {
                return IrType::Union(variants.iter().map(lower_type).collect());
            }
            if let Some(values) = enum_values {
                let members = values
                    .iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect();
                return IrType::Enum(members);
            }
            match schema_type.as_ref().and_then(SchemaType::primary_type) {
                Some("array") => {
                    let inner = items.as_deref().map(lower_type).unwrap_or(IrType::Prim(Prim::Any));
                    IrType::Array(Box::new(inner))
                }
                Some("object") | None => {
                    if let Some(crate::openapi::AdditionalProperties::Schema(value)) =
                        additional_properties
                    {
                        IrType::Map(Box::new(lower_type(value)))
                    } else {
                        IrType::Prim(Prim::Any)
                    }
                }
                Some("string") => match format.as_deref() {
                    Some("date-time") | Some("date") => IrType::Prim(Prim::DateTime),
                    Some("binary") | Some("byte") => IrType::Prim(Prim::Bytes),
                    _ => IrType::Prim(Prim::String),
                },
                Some("integer") => IrType::Prim(Prim::Int),
                Some("number") => IrType::Prim(Prim::Float),
                Some("boolean") => IrType::Prim(Prim::Bool),
                _ => IrType::Prim(Prim::Any),
            }
        }
    }
}

/// `#/components/schemas/Foo` -> `Foo`.
fn ref_name(reference: &str) -> String {
    reference
        .rsplit('/')
        .next()
        .unwrap_or(reference)
        .to_string()
}
