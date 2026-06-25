//! Intermediate representation (IR) for code generation.
//!
//! Today every generator under `src/generators/` consumes `&OpenApiSchema`
//! directly and re-implements the same schema walking in slightly different
//! ways (`$ref` resolution, optional/required handling, enum/map/alias
//! classification, topological sorting). Bugs fixed in one generator do not get
//! fixed in the others.
//!
//! This module is the classic compiler frontend/backend split:
//!
//! ```text
//!   OpenApiSchema --lower()--> IrDocument --Backend--> target source
//! ```
//!
//! `lower()` runs ONCE and absorbs the OpenAPI-specific logic. Each backend then
//! renders a target-agnostic IR and never sees a `Schema`.
//!
//! ## Frontends (input formats)
//!
//! `lower()` takes an [`OpenApiSchema`], which is also where the other input
//! formats already converge: the CLI converts **plain JSON** (`--from-json`)
//! and **JSON Schema** (`--from-json-schema`) into `OpenApiSchema` *before* any
//! generator runs (see `lib.rs`). So a backend built on this IR automatically
//! supports all three input formats — no per-format work needed.
//!
//! ## Status
//!
//! The Rust/serde generator (`rust_serde.rs`) is wired through this IR as the
//! first proof of concept; its golden tests pass byte-for-byte. Other backends
//! still walk `Schema` directly and can be migrated one at a time.

use crate::generators::common;
use crate::openapi::{AdditionalProperties, OpenApiSchema, Schema, schema_type_str};

// ---------------------------------------------------------------------------
// The IR
// ---------------------------------------------------------------------------

/// A scalar/leaf type, independent of any target language. `format` carries the
/// OpenAPI `format` hint (e.g. `int32`, `double`, `date-time`) so backends that
/// distinguish widths/representations can, while others ignore it.
#[derive(Debug, Clone)]
pub enum Scalar {
    String,
    Integer,
    Number,
    Boolean,
    /// A typed-but-shapeless object (`type: object` with no properties) — a
    /// string-keyed bag. Distinct from [`Scalar::Free`] so backends that have a
    /// dictionary type (C#) can use it while others fall back to "any".
    Object,
    /// Unconstrained — no `type`, arbitrary JSON.
    Free,
}

/// A target-agnostic type reference. `$ref`s are resolved to [`IrType::Named`];
/// nullability and not-required are kept on separate axes (see [`IrField`]).
#[derive(Debug, Clone)]
pub enum IrType {
    Scalar {
        kind: Scalar,
        format: Option<String>,
    },
    /// Reference to a top-level declaration by name (already normalized).
    Named(String),
    Array(Box<IrType>),
    /// `additionalProperties` — a string-keyed map of the inner type.
    Map(Box<IrType>),
    /// Schema-level nullability (`nullable: true` / type array includes `null`).
    /// Distinct from a field being not-`required`; backends combine the two.
    Optional(Box<IrType>),
    /// `oneOf` / `anyOf`.
    Union(Vec<IrType>),
    /// A closed set of string values used inline (not as a named type).
    Enum(Vec<String>),
}

/// One field of a struct/model.
#[derive(Debug, Clone)]
pub struct IrField {
    /// The wire/JSON key, verbatim.
    pub wire_name: String,
    pub ty: IrType,
    /// Whether the field is required. Orthogonal to `IrType::Optional`
    /// (nullability) — a backend decides how to render each combination.
    pub required: bool,
    pub docs: Option<String>,
}

/// A named object declaration (becomes an `interface`/`class`/`struct`).
#[derive(Debug, Clone)]
pub struct IrStruct {
    pub name: String,
    pub docs: Option<String>,
    pub fields: Vec<IrField>,
    /// The source schema had `oneOf`/`anyOf` variants that this flat struct
    /// could not fully model. Backends may surface this (e.g. as a note).
    pub has_unmodeled_variants: bool,
}

/// The members of a named enum, preserving their JSON value kind so each
/// backend can render them faithfully (string enum vs. integer enum) instead of
/// the IR collapsing one into the other.
#[derive(Debug, Clone)]
pub enum EnumValues {
    Strings(Vec<String>),
    Integers(Vec<i64>),
}

/// A top-level declaration.
#[derive(Debug, Clone)]
pub enum IrDecl {
    Struct(IrStruct),
    /// A named closed set of values.
    Enum {
        name: String,
        docs: Option<String>,
        values: EnumValues,
    },
    /// A named type alias / newtype (maps, scalars, refs, unions).
    Alias {
        name: String,
        docs: Option<String>,
        ty: IrType,
    },
}

impl IrDecl {
    pub fn name(&self) -> &str {
        match self {
            IrDecl::Struct(s) => &s.name,
            IrDecl::Enum { name, .. } => name,
            IrDecl::Alias { name, .. } => name,
        }
    }
}

/// The whole lowered document handed to a backend. `decls` are topologically
/// sorted so a backend can emit top-to-bottom without forward references.
#[derive(Debug, Clone, Default)]
pub struct IrDocument {
    pub decls: Vec<IrDecl>,
    // pub endpoints: Vec<IrEndpoint>,  // follow-up: lower paths/operations
}

// ---------------------------------------------------------------------------
// Backend trait — a thin convention for the type-emitting generators.
// ---------------------------------------------------------------------------

/// Renders an [`IrDocument`] to a target language. Deliberately NOT a per-node
/// visitor: a single `type_ref` that recurses internally matches how the
/// generators already work and avoids threading writer state through every node.
pub trait Backend {
    /// Render a type in reference position (right-hand side of a field).
    fn type_ref(&self, ty: &IrType) -> String;
    /// Render one declaration into the output buffer.
    fn decl(&self, decl: &IrDecl, out: &mut String);
    /// File header / imports. Default: nothing.
    fn preamble(&self, _doc: &IrDocument, _out: &mut String) {}

    fn render(&self, doc: &IrDocument) -> String {
        let mut out = String::new();
        self.preamble(doc, &mut out);
        for decl in &doc.decls {
            self.decl(decl, &mut out);
        }
        out
    }
}

// ---------------------------------------------------------------------------
// Lowering: OpenApiSchema -> IrDocument
// ---------------------------------------------------------------------------

/// Lower a parsed OpenAPI document (or anything normalized into one — see the
/// module docs on frontends) into the IR.
pub fn lower(schema: &OpenApiSchema) -> IrDocument {
    let mut decls = Vec::new();

    if let Some(components) = &schema.components
        && let Some(schemas) = &components.schemas
    {
        // Preserve source declaration order. Backends that cannot forward-
        // reference opt into dependency ordering via `topologically_sorted`.
        for (name, def) in schemas {
            decls.push(lower_decl(name, def));
        }
    }

    IrDocument { decls }
}

/// Reorder a document's declarations into dependency (topological) order, so a
/// backend that cannot forward-reference types can emit top-to-bottom. Reuses
/// the single canonical sort in `common` (previously also re-implemented as a
/// DFS in `python_dict.rs`). Backends that don't care keep source order.
pub fn topologically_sorted(mut doc: IrDocument, schema: &OpenApiSchema) -> IrDocument {
    if let Some(components) = &schema.components
        && let Some(schemas) = &components.schemas
        && let Ok(order) = common::topological_sort(schemas)
    {
        let rank: std::collections::HashMap<&str, usize> = order
            .iter()
            .enumerate()
            .map(|(i, n)| (n.as_str(), i))
            .collect();
        doc.decls
            .sort_by_key(|d| rank.get(d.name()).copied().unwrap_or(usize::MAX));
    }
    doc
}

/// Reorder declarations using a depth-first dependency walk with alphabetical
/// tie-breaking. This is a *different* total order from [`topologically_sorted`]
/// (Kahn's) — the python-dict generator historically used this DFS variant, and
/// reproducing it keeps that generator's output stable. The two orderings are
/// kept side by side here (rather than in two generators) so the divergence is
/// visible and can be reconciled later.
pub fn dfs_dependency_sorted(mut doc: IrDocument, schema: &OpenApiSchema) -> IrDocument {
    let Some(schemas) = schema.components.as_ref().and_then(|c| c.schemas.as_ref()) else {
        return doc;
    };
    let known: std::collections::HashSet<&str> = schemas.keys().map(String::as_str).collect();

    fn deps_of(schema: &Schema, known: &std::collections::HashSet<&str>) -> Vec<String> {
        let mut deps = std::collections::HashSet::new();
        fn walk(s: &Schema, known: &std::collections::HashSet<&str>, out: &mut std::collections::HashSet<String>) {
            match s {
                Schema::Reference { reference, .. } => {
                    if let Some(n) = reference.strip_prefix("#/components/schemas/")
                        && known.contains(n)
                    {
                        out.insert(n.to_string());
                    }
                }
                Schema::Object {
                    properties,
                    items,
                    one_of,
                    any_of,
                    ..
                } => {
                    for p in properties.iter().flat_map(|m| m.values()) {
                        walk(p, known, out);
                    }
                    if let Some(it) = items {
                        walk(it, known, out);
                    }
                    for v in [one_of, any_of].into_iter().flatten().flatten() {
                        walk(v, known, out);
                    }
                }
            }
        }
        walk(schema, known, &mut deps);
        let mut v: Vec<String> = deps.into_iter().collect();
        v.sort();
        v
    }

    #[allow(clippy::too_many_arguments)]
    fn visit(
        name: &str,
        schemas: &indexmap::IndexMap<String, Schema>,
        known: &std::collections::HashSet<&str>,
        visited: &mut std::collections::HashSet<String>,
        in_stack: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>,
    ) {
        if in_stack.contains(name) || visited.contains(name) {
            return;
        }
        in_stack.insert(name.to_string());
        if let Some(def) = schemas.get(name) {
            for dep in deps_of(def, known) {
                visit(&dep, schemas, known, visited, in_stack, order);
            }
        }
        in_stack.remove(name);
        visited.insert(name.to_string());
        order.push(name.to_string());
    }

    let mut keys: Vec<&String> = schemas.keys().collect();
    keys.sort();
    let mut order = Vec::new();
    let mut visited = std::collections::HashSet::new();
    let mut in_stack = std::collections::HashSet::new();
    for key in keys {
        visit(key, schemas, &known, &mut visited, &mut in_stack, &mut order);
    }

    let rank: std::collections::HashMap<&str, usize> =
        order.iter().enumerate().map(|(i, n)| (n.as_str(), i)).collect();
    doc.decls
        .sort_by_key(|d| rank.get(d.name()).copied().unwrap_or(usize::MAX));
    doc
}

/// Classify a top-level schema into a declaration. Mirrors the structural
/// decisions every model generator makes: map alias, enum, integer-enum alias,
/// struct, or opaque alias.
fn lower_decl(name: &str, schema: &Schema) -> IrDecl {
    let docs = schema.get_description().map(str::to_string);
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
            // Map type: additionalProperties with no declared properties.
            if properties.is_none()
                && let Some(AdditionalProperties::Schema(value)) = additional_properties
            {
                return IrDecl::Alias {
                    name: name.to_string(),
                    docs,
                    ty: IrType::Map(Box::new(lower_type(value))),
                };
            }

            // Enums keep their value kind so backends render them faithfully.
            if let Some(values) = enum_values {
                let enum_values = if values.iter().all(|v| v.is_i64() || v.is_u64()) {
                    EnumValues::Integers(values.iter().filter_map(|v| v.as_i64()).collect())
                } else {
                    EnumValues::Strings(
                        values
                            .iter()
                            .filter_map(|v| v.as_str().map(str::to_string))
                            .collect(),
                    )
                };
                return IrDecl::Enum {
                    name: name.to_string(),
                    docs,
                    values: enum_values,
                };
            }

            // Otherwise a struct: fields come from allOf members if present,
            // else from properties.
            let fields = if let Some(all_of_schemas) = all_of {
                let mut acc = Vec::new();
                for member in all_of_schemas {
                    if let Schema::Object {
                        properties: Some(props),
                        required: req,
                        ..
                    } = member
                    {
                        collect_fields(props, req.as_ref(), &mut acc);
                    }
                }
                acc
            } else if let Some(props) = properties {
                let mut acc = Vec::new();
                collect_fields(props, required.as_ref(), &mut acc);
                acc
            } else {
                Vec::new()
            };

            // Object-shaped schemas become structs; a bare scalar/array schema
            // at top level becomes a type alias.
            let is_object =
                schema_type_str(schema_type) == Some("object") || properties.is_some() || all_of.is_some();
            if is_object {
                IrDecl::Struct(IrStruct {
                    name: name.to_string(),
                    docs,
                    fields,
                    has_unmodeled_variants: one_of.is_some() || any_of.is_some(),
                })
            } else {
                IrDecl::Alias {
                    name: name.to_string(),
                    docs,
                    ty: lower_type(schema),
                }
            }
        }
        // A top-level bare `$ref` becomes an alias to the referenced type.
        Schema::Reference { reference, .. } => IrDecl::Alias {
            name: name.to_string(),
            docs,
            ty: IrType::Named(ref_name(reference)),
        },
    }
}

fn collect_fields(
    props: &indexmap::IndexMap<String, Schema>,
    required: Option<&Vec<String>>,
    out: &mut Vec<IrField>,
) {
    for (field_name, field_schema) in props {
        let is_required = required.is_some_and(|r| r.iter().any(|n| n == field_name));
        let mut ty = lower_type(field_schema);
        // Keep schema-level nullability as a distinct axis from required.
        if field_schema.is_nullable() {
            ty = IrType::Optional(Box::new(ty));
        }
        out.push(IrField {
            wire_name: field_name.clone(),
            ty,
            required: is_required,
            docs: field_schema.get_description().map(str::to_string),
        });
    }
}

/// Map a single `Schema` node (in field/value position) to an `IrType`. This is
/// the logic duplicated today as `schema_to_typescript` / `to_rust_type` /
/// `schema_to_python_type` — here it lives once.
fn lower_type(schema: &Schema) -> IrType {
    match schema {
        Schema::Reference { reference, .. } => IrType::Named(ref_name(reference)),
        Schema::Object {
            schema_type,
            items,
            enum_values,
            additional_properties,
            properties,
            one_of,
            any_of,
            format,
            ..
        } => {
            if let Some(variants) = one_of.as_ref().or(any_of.as_ref()) {
                return IrType::Union(variants.iter().map(lower_type).collect());
            }
            if let Some(values) = enum_values {
                // All-string enums become a closed set (literal union / Literal);
                // integer or mixed enums fall through to a plain scalar.
                if values.iter().all(|v| v.is_string()) {
                    let members = values
                        .iter()
                        .filter_map(|v| v.as_str().map(str::to_string))
                        .collect();
                    return IrType::Enum(members);
                }
            }
            match schema_type_str(schema_type) {
                Some("array") => {
                    let inner = items
                        .as_deref()
                        .map(lower_type)
                        .unwrap_or(IrType::Scalar {
                            kind: Scalar::Free,
                            format: None,
                        });
                    IrType::Array(Box::new(inner))
                }
                Some("object") => {
                    if properties.is_none()
                        && let Some(AdditionalProperties::Schema(value)) = additional_properties
                    {
                        IrType::Map(Box::new(lower_type(value)))
                    } else {
                        IrType::Scalar {
                            kind: Scalar::Object,
                            format: None,
                        }
                    }
                }
                Some("string") => scalar(Scalar::String, format),
                Some("integer") => scalar(Scalar::Integer, format),
                Some("number") => scalar(Scalar::Number, format),
                Some("boolean") => scalar(Scalar::Boolean, format),
                _ => IrType::Scalar {
                    kind: Scalar::Free,
                    format: None,
                },
            }
        }
    }
}

fn scalar(kind: Scalar, format: &Option<String>) -> IrType {
    IrType::Scalar {
        kind,
        format: format.clone(),
    }
}

/// `#/components/schemas/Foo` -> `Foo`.
fn ref_name(reference: &str) -> String {
    reference
        .strip_prefix("#/components/schemas/")
        .unwrap_or(reference)
        .to_string()
}
