use serde::{Deserialize, Serialize};
use indexmap::IndexMap;

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
pub enum Schema {
    Reference {
        #[serde(rename = "$ref")]
        reference: String,
    },
    Object {
        #[serde(rename = "type")]
        schema_type: Option<String>,
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
    pub fn get_type(&self) -> Option<&str> {
        match self {
            Schema::Object { schema_type, .. } => schema_type.as_deref(),
            Schema::Reference { .. } => None,
        }
    }
    
    pub fn is_nullable(&self) -> bool {
        match self {
            Schema::Object { nullable, .. } => nullable.unwrap_or(false),
            Schema::Reference { .. } => false,
        }
    }
    
    pub fn get_reference(&self) -> Option<&str> {
        match self {
            Schema::Reference { reference } => Some(reference),
            Schema::Object { .. } => None,
        }
    }
} 