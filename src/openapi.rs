use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// Schema type that supports both OpenAPI 3.0 (single string) and 3.1 (array of strings).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SchemaType {
    Single(String),
    Multiple(Vec<String>),
}

impl SchemaType {
    /// Returns the primary (non-null) type string.
    pub fn primary_type(&self) -> Option<&str> {
        match self {
            SchemaType::Single(s) => Some(s.as_str()),
            SchemaType::Multiple(types) => types
                .iter()
                .find(|t| t.as_str() != "null")
                .map(|s| s.as_str()),
        }
    }

    /// Returns true if "null" is included in the type array.
    pub fn has_null(&self) -> bool {
        match self {
            SchemaType::Single(s) => s == "null",
            SchemaType::Multiple(types) => types.iter().any(|t| t == "null"),
        }
    }
}

/// Helper to get the primary type string from an `Option<SchemaType>`.
/// Mirrors the old `Option<String>::as_deref()` pattern.
pub fn schema_type_str(schema_type: &Option<SchemaType>) -> Option<&str> {
    schema_type.as_ref().and_then(|t| t.primary_type())
}

/// Check nullability from both the OpenAPI 3.0 `nullable` field and
/// the OpenAPI 3.1 type array containing "null".
pub fn is_schema_nullable(nullable: &Option<bool>, schema_type: &Option<SchemaType>) -> bool {
    nullable.unwrap_or(false) || schema_type.as_ref().is_some_and(|t| t.has_null())
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenApiSchema {
    pub openapi: Option<String>,
    pub info: Option<Info>,
    pub components: Option<Components>,
    pub paths: Option<IndexMap<String, PathItem>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Components {
    pub schemas: Option<IndexMap<String, Schema>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PathItem {
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub delete: Option<Operation>,
    pub patch: Option<Operation>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Operation {
    #[serde(rename = "operationId")]
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub parameters: Option<Vec<Parameter>>,
    #[serde(rename = "requestBody")]
    pub request_body: Option<RequestBody>,
    pub responses: Option<IndexMap<String, Response>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: String,
    pub required: Option<bool>,
    pub schema: Option<Schema>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub content: Option<IndexMap<String, MediaType>>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    pub description: String,
    pub content: Option<IndexMap<String, MediaType>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MediaType {
    pub schema: Option<Schema>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AdditionalProperties {
    Boolean(bool),
    Schema(Box<Schema>),
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
#[allow(clippy::large_enum_variant)]
pub enum Schema {
    Reference {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Object {
        #[serde(rename = "type")]
        schema_type: Option<SchemaType>,
        properties: Option<IndexMap<String, Schema>>,
        required: Option<Vec<String>>,
        #[serde(rename = "additionalProperties")]
        additional_properties: Option<AdditionalProperties>,
        items: Option<Box<Schema>>,
        #[serde(rename = "enum")]
        enum_values: Option<Vec<serde_json::Value>>,
        format: Option<String>,
        description: Option<String>,
        example: Option<serde_json::Value>,
        #[serde(rename = "allOf")]
        all_of: Option<Vec<Schema>>,
        #[serde(rename = "oneOf")]
        one_of: Option<Vec<Schema>>,
        #[serde(rename = "anyOf")]
        any_of: Option<Vec<Schema>>,
        minimum: Option<f64>,
        maximum: Option<f64>,
        #[serde(rename = "minLength")]
        min_length: Option<usize>,
        #[serde(rename = "maxLength")]
        max_length: Option<usize>,
        pattern: Option<String>,
        nullable: Option<bool>,
    },
}

impl Schema {
    /// Returns the primary (non-null) type string.
    /// For OpenAPI 3.1 array types like `["string", "null"]`, returns `"string"`.
    pub fn get_type(&self) -> Option<&str> {
        match self {
            Schema::Object { schema_type, .. } => match schema_type {
                Some(SchemaType::Single(s)) => Some(s.as_str()),
                Some(SchemaType::Multiple(types)) => types
                    .iter()
                    .find(|t| t.as_str() != "null")
                    .map(|s| s.as_str()),
                None => None,
            },
            Schema::Reference { .. } => None,
        }
    }

    /// Returns true if this schema is nullable, checking both the OpenAPI 3.0
    /// `nullable` field and the OpenAPI 3.1 type array containing "null".
    pub fn is_nullable(&self) -> bool {
        match self {
            Schema::Object {
                schema_type,
                nullable,
                ..
            } => {
                if nullable == &Some(true) {
                    return true;
                }
                if let Some(SchemaType::Multiple(types)) = schema_type {
                    return types.iter().any(|t| t == "null");
                }
                false
            }
            Schema::Reference { .. } => false,
        }
    }

    /// Create a new builder for Schema::Object
    pub fn object() -> SchemaObjectBuilder {
        SchemaObjectBuilder::new()
    }

    /// Create a reference schema
    pub fn reference(reference: impl Into<String>) -> Self {
        Schema::Reference {
            reference: reference.into(),
        }
    }

    // Convenience methods for common schema types
    pub fn string() -> Self {
        Schema::object().schema_type("string").build()
    }

    pub fn integer() -> Self {
        Schema::object().schema_type("integer").build()
    }

    pub fn number() -> Self {
        Schema::object().schema_type("number").build()
    }

    pub fn boolean() -> Self {
        Schema::object().schema_type("boolean").build()
    }

    pub fn null() -> Self {
        Schema::object().schema_type("null").nullable(true).build()
    }

    pub fn array(items: Schema) -> Self {
        Schema::object()
            .schema_type("array")
            .items(Box::new(items))
            .build()
    }
}

/// Builder for Schema::Object
#[derive(Debug, Clone, Default)]
pub struct SchemaObjectBuilder {
    schema_type: Option<SchemaType>,
    properties: Option<IndexMap<String, Schema>>,
    required: Option<Vec<String>>,
    additional_properties: Option<AdditionalProperties>,
    items: Option<Box<Schema>>,
    enum_values: Option<Vec<serde_json::Value>>,
    format: Option<String>,
    description: Option<String>,
    example: Option<serde_json::Value>,
    all_of: Option<Vec<Schema>>,
    one_of: Option<Vec<Schema>>,
    any_of: Option<Vec<Schema>>,
    minimum: Option<f64>,
    maximum: Option<f64>,
    min_length: Option<usize>,
    max_length: Option<usize>,
    pattern: Option<String>,
    nullable: Option<bool>,
}

impl SchemaObjectBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the schema type (accepts a single type string)
    pub fn schema_type(mut self, schema_type: impl Into<String>) -> Self {
        self.schema_type = Some(SchemaType::Single(schema_type.into()));
        self
    }

    /// Set properties
    pub fn properties(mut self, properties: IndexMap<String, Schema>) -> Self {
        self.properties = Some(properties);
        self
    }

    /// Set required fields
    pub fn required(mut self, required: Vec<String>) -> Self {
        self.required = Some(required);
        self
    }

    /// Set additional properties
    pub fn additional_properties(mut self, additional_properties: AdditionalProperties) -> Self {
        self.additional_properties = Some(additional_properties);
        self
    }

    /// Set items for array types
    pub fn items(mut self, items: Box<Schema>) -> Self {
        self.items = Some(items);
        self
    }

    /// Set enum values
    pub fn enum_values(mut self, enum_values: Vec<serde_json::Value>) -> Self {
        self.enum_values = Some(enum_values);
        self
    }

    /// Set format
    pub fn format(mut self, format: impl Into<String>) -> Self {
        self.format = Some(format.into());
        self
    }

    /// Set description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set example
    pub fn example(mut self, example: serde_json::Value) -> Self {
        self.example = Some(example);
        self
    }

    /// Set allOf schemas
    pub fn all_of(mut self, all_of: Vec<Schema>) -> Self {
        self.all_of = Some(all_of);
        self
    }

    /// Set oneOf schemas
    pub fn one_of(mut self, one_of: Vec<Schema>) -> Self {
        self.one_of = Some(one_of);
        self
    }

    /// Set anyOf schemas
    pub fn any_of(mut self, any_of: Vec<Schema>) -> Self {
        self.any_of = Some(any_of);
        self
    }

    /// Set minimum value
    pub fn minimum(mut self, minimum: f64) -> Self {
        self.minimum = Some(minimum);
        self
    }

    /// Set maximum value
    pub fn maximum(mut self, maximum: f64) -> Self {
        self.maximum = Some(maximum);
        self
    }

    /// Set minimum length
    pub fn min_length(mut self, min_length: usize) -> Self {
        self.min_length = Some(min_length);
        self
    }

    /// Set maximum length
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set pattern
    pub fn pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Set nullable
    pub fn nullable(mut self, nullable: bool) -> Self {
        self.nullable = Some(nullable);
        self
    }

    /// Build the Schema::Object
    pub fn build(self) -> Schema {
        Schema::Object {
            schema_type: self.schema_type,
            properties: self.properties,
            required: self.required,
            additional_properties: self.additional_properties,
            items: self.items,
            enum_values: self.enum_values,
            format: self.format,
            description: self.description,
            example: self.example,
            all_of: self.all_of,
            one_of: self.one_of,
            any_of: self.any_of,
            minimum: self.minimum,
            maximum: self.maximum,
            min_length: self.min_length,
            max_length: self.max_length,
            pattern: self.pattern,
            nullable: self.nullable,
        }
    }
}
