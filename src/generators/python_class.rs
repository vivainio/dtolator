//! Structured representation of a Python class definition.
//!
//! This mirrors the approach taken by [`super::zod_schema::ZodValue`]: build the
//! shape of the output as Rust data first, then render it to a string in one
//! place. Keeping the data transformation (schema -> `PythonClassDef`) separate
//! from the printing (`PythonClassDef` -> `String`) means formatting and syntax
//! bugs can be fixed without touching the schema-walking logic, and vice versa.

use indexmap::IndexMap;
use std::fmt;

/// A single attribute (field, enum member, or class-level assignment) of a
/// [`PythonClassDef`].
///
/// Renders, in order, as `name[: type_annotation][ = default]` followed by an
/// optional attribute docstring on the next line(s).
///
/// - `type_annotation`: the type after the colon, e.g. `str | None`. `None`
///   produces a bare assignment with no annotation, as used by enum members
///   (`PENDING = "pending"`) and class-level config (`model_config = ...`).
/// - `default`: the right-hand side of `=`, e.g. `None`, `Field(...)`,
///   `"pending"`. `None` means no assignment at all (a required annotated
///   field: `name: type`).
/// - `docstring`: an optional attribute docstring rendered below the line.
/// - `blank_line_after`: a layout hint to emit a blank line after this
///   attribute, used to separate a class-level config preamble (e.g. a Pydantic
///   v2 `model_config`) from the fields that follow it.
#[derive(Debug, Clone, Default)]
pub struct PythonAttribute {
    pub type_annotation: Option<String>,
    pub default: Option<String>,
    pub docstring: Option<String>,
    pub blank_line_after: bool,
}

impl PythonAttribute {
    /// A bare `name = value` assignment with no type annotation
    /// (enum members, `model_config = ...`, etc.).
    pub fn assignment(value: impl Into<String>) -> Self {
        Self {
            type_annotation: None,
            default: Some(value.into()),
            docstring: None,
            blank_line_after: false,
        }
    }

    /// An annotated field `name: type` with an optional default value.
    pub fn field(type_annotation: impl Into<String>, default: Option<String>) -> Self {
        Self {
            type_annotation: Some(type_annotation.into()),
            default,
            docstring: None,
            blank_line_after: false,
        }
    }
}

/// A Python class definition: its header, an optional docstring, nested classes,
/// and its attributes. Render it with [`PythonClassDef::render`] or `Display`.
#[derive(Debug, Clone, Default)]
pub struct PythonClassDef {
    pub name: String,
    /// Base classes, e.g. `["BaseModel"]` or `["str", "Enum"]`. Empty produces a
    /// bare `class Name:` header.
    pub base_classes: Vec<String>,
    /// Keyword arguments on the class header, rendered as `key=value` after the
    /// base classes (e.g. `class Name(BaseModel, frozen=True):`).
    pub class_kwargs: IndexMap<String, String>,
    pub docstring: Option<String>,
    pub inner_classes: Vec<PythonClassDef>,
    pub attributes: IndexMap<String, PythonAttribute>,
}

impl PythonClassDef {
    pub fn new(name: impl Into<String>, base_classes: Vec<String>) -> Self {
        Self {
            name: name.into(),
            base_classes,
            ..Default::default()
        }
    }

    /// Render the class to source text, ending with a single newline after the
    /// last body line. Callers append any further blank lines that separate
    /// top-level definitions.
    pub fn render(&self) -> String {
        let mut out = String::new();
        self.render_indented(&mut out, 0);
        out
    }

    fn render_indented(&self, out: &mut String, level: usize) {
        let pad = "    ".repeat(level);
        let body_pad = "    ".repeat(level + 1);

        // Header: `class Name(Base1, Base2, key=value):`
        let mut bases = self.base_classes.clone();
        for (key, value) in &self.class_kwargs {
            bases.push(format!("{key}={value}"));
        }
        if bases.is_empty() {
            out.push_str(&format!("{pad}class {}:\n", self.name));
        } else {
            out.push_str(&format!(
                "{pad}class {}({}):\n",
                self.name,
                bases.join(", ")
            ));
        }

        if let Some(doc) = &self.docstring {
            render_docstring(out, doc, &pad, &body_pad);
        }

        // An empty body still needs a `pass` to be valid Python, even when a
        // docstring is present (matching the existing generator's behaviour).
        if self.inner_classes.is_empty() && self.attributes.is_empty() {
            out.push_str(&format!("{body_pad}pass\n"));
            return;
        }

        // Nested classes first, each followed by a blank line separating it from
        // what follows (e.g. a Pydantic v1 `class Config:` from the fields).
        for inner in &self.inner_classes {
            inner.render_indented(out, level + 1);
            out.push('\n');
        }

        for (name, attr) in &self.attributes {
            out.push_str(&body_pad);
            out.push_str(name);
            if let Some(ty) = &attr.type_annotation {
                out.push_str(&format!(": {ty}"));
            }
            if let Some(default) = &attr.default {
                out.push_str(&format!(" = {default}"));
            }
            out.push('\n');
            if let Some(doc) = &attr.docstring {
                render_docstring(out, doc, &body_pad, &format!("{body_pad}    "));
            }
            if attr.blank_line_after {
                out.push('\n');
            }
        }
    }
}

impl fmt::Display for PythonClassDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

/// Render a `"""docstring"""`. Single-line docstrings are kept on one line;
/// multi-line docstrings are spread across opening/closing `"""` lines, with
/// blank source lines emitted truly empty (no trailing whitespace).
fn render_docstring(out: &mut String, doc: &str, base_pad: &str, content_pad: &str) {
    if doc.contains('\n') {
        out.push_str(&format!("{content_pad}\"\"\"\n"));
        for line in doc.lines() {
            if line.is_empty() {
                out.push_str(&format!("{base_pad}\n"));
            } else {
                out.push_str(&format!("{content_pad}{line}\n"));
            }
        }
        out.push_str(&format!("{content_pad}\"\"\"\n"));
    } else {
        out.push_str(&format!("{content_pad}\"\"\"{doc}\"\"\"\n"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_str_enum() {
        let mut class = PythonClassDef::new("Color", vec!["str".into(), "Enum".into()]);
        class
            .attributes
            .insert("RED".into(), PythonAttribute::assignment("\"red\""));
        class
            .attributes
            .insert("GREEN".into(), PythonAttribute::assignment("\"green\""));

        assert_eq!(
            class.render(),
            "class Color(str, Enum):\n    RED = \"red\"\n    GREEN = \"green\"\n"
        );
    }

    #[test]
    fn renders_empty_class_with_pass() {
        let class = PythonClassDef::new("Empty", vec!["BaseModel".into()]);
        assert_eq!(class.render(), "class Empty(BaseModel):\n    pass\n");
    }

    #[test]
    fn renders_fields_with_defaults() {
        let mut class = PythonClassDef::new("User", vec!["BaseModel".into()]);
        class
            .attributes
            .insert("id".into(), PythonAttribute::field("int", None));
        class.attributes.insert(
            "name".into(),
            PythonAttribute::field("str | None", Some("None".into())),
        );

        assert_eq!(
            class.render(),
            "class User(BaseModel):\n    id: int\n    name: str | None = None\n"
        );
    }

    #[test]
    fn renders_inner_class_with_trailing_blank_line() {
        let mut class = PythonClassDef::new("User", vec!["BaseModel".into()]);
        let mut config = PythonClassDef::new("Config", vec![]);
        config.attributes.insert(
            "allow_population_by_field_name".into(),
            PythonAttribute::assignment("True"),
        );
        class.inner_classes.push(config);
        class
            .attributes
            .insert("id".into(), PythonAttribute::field("int", None));

        assert_eq!(
            class.render(),
            "class User(BaseModel):\n    class Config:\n        allow_population_by_field_name = True\n\n    id: int\n"
        );
    }

    #[test]
    fn renders_config_attribute_with_blank_line_after() {
        let mut class = PythonClassDef::new("User", vec!["BaseModel".into()]);
        let mut config = PythonAttribute::assignment("ConfigDict(populate_by_name=True)");
        config.blank_line_after = true;
        class.attributes.insert("model_config".into(), config);
        class
            .attributes
            .insert("id".into(), PythonAttribute::field("int", None));

        assert_eq!(
            class.render(),
            "class User(BaseModel):\n    model_config = ConfigDict(populate_by_name=True)\n\n    id: int\n"
        );
    }

    #[test]
    fn renders_multiline_docstring() {
        let mut class = PythonClassDef::new("User", vec!["BaseModel".into()]);
        class.docstring = Some("A user.\n\nMore detail.".into());
        class
            .attributes
            .insert("id".into(), PythonAttribute::field("int", None));

        assert_eq!(
            class.render(),
            "class User(BaseModel):\n    \"\"\"\n    A user.\n\n    More detail.\n    \"\"\"\n    id: int\n"
        );
    }

    #[test]
    fn renders_class_kwargs() {
        let mut class = PythonClassDef::new("User", vec!["BaseModel".into()]);
        class.class_kwargs.insert("frozen".into(), "True".into());
        class
            .attributes
            .insert("id".into(), PythonAttribute::field("int", None));

        assert_eq!(
            class.render(),
            "class User(BaseModel, frozen=True):\n    id: int\n"
        );
    }
}
