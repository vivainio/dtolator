use std::fmt;

/// Structured representation of Zod schemas
/// This allows building schemas as Rust types first, then converting to strings

#[derive(Debug, Clone)]
pub enum ZodValue {
    String(StringConstraints),
    Number(NumberConstraints),
    Boolean,
    Array(Box<ZodValue>),
    Object(Vec<(String, ZodValue, bool)>), // (name, schema, required)
    Reference(String),
    Union(Vec<ZodValue>),
    Intersection(Vec<ZodValue>),
    Enum(Vec<String>),
    Nullable(Box<ZodValue>),
    Unknown,
}

#[derive(Debug, Clone)]
pub struct StringConstraints {
    pub format: Option<String>, // "uuid", "email", "uri", "date", "date-time"
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NumberConstraints {
    pub is_integer: bool,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
}

impl fmt::Display for ZodValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.render())
    }
}

impl ZodValue {
    /// Convert a ZodValue to its string representation
    fn render(&self) -> String {
        match self {
            ZodValue::String(constraints) => Self::string_to_zod(constraints),
            ZodValue::Number(constraints) => Self::number_to_zod(constraints),
            ZodValue::Boolean => "z.boolean()".to_string(),
            ZodValue::Array(inner) => format!("z.array({})", inner.render()),
            ZodValue::Object(props) => Self::object_to_zod(props),
            ZodValue::Reference(name) => format!("{name}Schema"),
            ZodValue::Union(values) => {
                let value_strs: Vec<String> = values.iter().map(|v| v.render()).collect();
                format!("z.union([{}])", value_strs.join(", "))
            }
            ZodValue::Intersection(values) => {
                let value_strs: Vec<String> = values.iter().map(|v| v.render()).collect();
                format!("z.intersection({})", value_strs.join(", z.intersection("))
            }
            ZodValue::Enum(values) => {
                let value_strs: Vec<String> = values.iter().map(|v| format!("\"{v}\"")).collect();
                format!("z.enum([{}])", value_strs.join(", "))
            }
            ZodValue::Nullable(inner) => format!("{}.nullable()", inner.render()),
            ZodValue::Unknown => "z.unknown()".to_string(),
        }
    }

    fn string_to_zod(constraints: &StringConstraints) -> String {
        let mut s = match constraints.format.as_deref() {
            Some("uuid") => "z.uuid()".to_string(),
            Some("email") => "z.email()".to_string(),
            Some("uri") => "z.url()".to_string(),
            Some("date") => "z.iso.date()".to_string(),
            Some("date-time") => "z.iso.datetime()".to_string(),
            _ => "z.string()".to_string(),
        };

        if let Some(min) = constraints.min_length {
            s.push_str(&format!(".min({min})"));
        }
        if let Some(max) = constraints.max_length {
            s.push_str(&format!(".max({max})"));
        }
        if let Some(pattern) = &constraints.pattern {
            let escaped = Self::escape_regex_pattern(pattern);
            s.push_str(&format!(".regex(/{escaped}/)"));
        }

        s
    }

    fn number_to_zod(constraints: &NumberConstraints) -> String {
        let mut s = "z.number()".to_string();

        if let Some(min) = constraints.minimum {
            s.push_str(&format!(".min({min})"));
        }
        if let Some(max) = constraints.maximum {
            s.push_str(&format!(".max({max})"));
        }
        if constraints.is_integer {
            s.push_str(".int()");
        }

        s
    }

    fn object_to_zod(props: &[(String, ZodValue, bool)]) -> String {
        let prop_strs: Vec<String> = props
            .iter()
            .map(|(name, zod_val, required)| {
                let val_str = zod_val.render();
                let constraint = if *required {
                    val_str
                } else {
                    format!("{val_str}.optional()")
                };
                format!("  {name}: {constraint}")
            })
            .collect();

        format!("z.object({{\n{}\n}})", prop_strs.join(",\n"))
    }

    fn escape_regex_pattern(pattern: &str) -> String {
        pattern.replace('\\', "\\\\").replace('/', "\\/")
    }
}
