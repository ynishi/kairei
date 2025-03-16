//! Models for DSL documentation API responses.

use kairei_core::analyzer::ParserDocumentation;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use utoipa::{IntoParams, ToSchema};

/// Response containing all DSL documentation.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentationResponse {
    /// Total number of documented parsers
    pub total_parsers: usize,
    /// Available documentation categories
    pub categories: Vec<String>,
    /// Documentation organized by category
    #[schema(value_type = Vec<CategoryDocumentation>)]
    pub by_category: HashMap<String, CategoryDocumentation>,
}

/// Documentation for a specific category of parsers.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct CategoryDocumentation {
    /// Category name
    pub name: String,
    /// Category description
    pub description: String,
    /// Number of parsers in this category
    pub parser_count: usize,
    /// Documentation for each parser in this category
    pub parsers: Vec<ParserDocumentationResponse>,
}

/// Documentation for an individual parser.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ParserDocumentationResponse {
    /// Parser name
    pub name: String,
    /// Parser description
    pub description: String,
    /// Category this parser belongs to
    pub category: String,
    /// Examples of valid syntax this parser handles
    pub examples: Vec<String>,
    /// Whether this parser is deprecated
    pub deprecated: bool,
    /// Deprecation message if any
    pub deprecation_message: Option<String>,
    /// Related parsers
    pub related_parsers: Vec<String>,
}

impl From<&ParserDocumentation> for ParserDocumentationResponse {
    fn from(doc: &ParserDocumentation) -> Self {
        Self {
            name: doc.name.clone(),
            description: doc.description.clone(),
            category: doc.category.to_string(),
            examples: doc.examples.clone(),
            deprecated: doc.deprecated.is_some(),
            deprecation_message: doc.deprecated.clone(),
            related_parsers: doc.related_parsers.clone(),
        }
    }
}

/// Documentation query parameters
#[derive(Debug, Deserialize, ToSchema, IntoParams)]
pub struct DocumentationQueryParams {
    /// Filter documentation by search query
    #[schema(example = "think")]
    pub search: Option<String>,
    /// Format to return (json, markdown)
    #[schema(example = "json", default = "json")]
    pub format: Option<String>,
}

/// Error response for documentation endpoints
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DocumentationErrorResponse {
    /// Error message
    pub error: String,
    /// Additional details about the error
    pub details: Option<String>,
}

/// Response containing a map of available documentation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct DocumentationMapResponse {
    /// API version (matches kairei-core version)
    pub version: String,
    /// Available parser categories
    pub categories: Vec<String>,
    /// Parser names by category
    pub parsers_by_category: HashMap<String, Vec<String>>,
}

/// Request to export documentation
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExportDocumentationRequest {
    /// Export format (json, markdown)
    #[schema(example = "json", default = "json")]
    pub format: String,

    /// Include version information
    #[schema(example = "true", default = "true")]
    pub include_version: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub enum ExportFormat {
    Json,
    Markdown,
}

impl Default for ExportFormat {
    fn default() -> Self {
        Self::Json
    }
}

impl TryFrom<&str> for ExportFormat {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "json" => Ok(Self::Json),
            "markdown" => Ok(Self::Markdown),
            "md" => Ok(Self::Markdown),
            _ => Err(format!("Invalid export format: {}", value)),
        }
    }
}

/// Response from documentation export
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct ExportDocumentationResponse {
    /// Export format
    pub format: String,

    /// Documentation content
    pub content: String,

    /// API version
    pub version: String,
}
