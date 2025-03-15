//! Handlers for DSL documentation endpoints.

use crate::auth::AuthUser;
use crate::models::docs::{
    CategoryDocumentation, DocumentationErrorResponse, DocumentationQueryParams,
    DocumentationResponse, ParserDocumentationResponse,
};
use crate::server::AppState;
use axum::Json;
use axum::extract::{Path, Query, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use kairei_core::analyzer::{DocumentationCollection, ParserCategory};
use std::collections::HashMap;
// use std::str::FromStr;
use tracing::{debug, warn};

/// Get all DSL documentation
///
/// Returns documentation for all parsers organized by category.
#[utoipa::path(
    get,
    path = "/docs/dsl",
    responses(
        (status = 200, description = "Documentation retrieved successfully", body = DocumentationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error", body = DocumentationErrorResponse)
    ),
    params(
        DocumentationQueryParams
    )
)]
pub async fn get_all_documentation(
    State(state): State<AppState>,
    _auth: AuthUser,
    headers: HeaderMap,
    Query(params): Query<DocumentationQueryParams>,
) -> Result<Response, StatusCode> {
    debug!("Getting all DSL documentation");

    // In a real implementation, this would access the system's documentation collection
    // For now, we'll create a sample collection
    match get_documentation_from_system(&state).await {
        Ok(doc_collection) => {
            let accept_markdown = is_markdown_requested(&headers);

            // Filter by search query if provided
            let filtered_collection = if let Some(search) = params.search {
                filter_documentation_by_search(&doc_collection, &search)
            } else {
                doc_collection
            };

            // Return response in requested format
            if accept_markdown || params.format == Some("markdown".to_string()) {
                let markdown = generate_markdown_documentation(&filtered_collection);

                Response::builder()
                    .header(header::CONTENT_TYPE, "text/markdown")
                    .body(markdown)
                    .map(IntoResponse::into_response)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                // Default to JSON
                let response = create_documentation_response(&filtered_collection);
                Ok(Json(response).into_response())
            }
        }
        Err(e) => {
            warn!("Failed to get documentation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get documentation for a specific category
///
/// Returns documentation for parsers in the specified category.
#[utoipa::path(
    get,
    path = "/docs/dsl/{category}",
    responses(
        (status = 200, description = "Category documentation retrieved successfully", body = CategoryDocumentation),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Category not found", body = DocumentationErrorResponse),
        (status = 500, description = "Internal server error", body = DocumentationErrorResponse)
    ),
    params(
        ("category" = String, Path, description = "Category name (Expression, Statement, Handler, Type, or Definition)"),
        DocumentationQueryParams
    )
)]
pub async fn get_category_documentation(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path(category): Path<String>,
    headers: HeaderMap,
    Query(params): Query<DocumentationQueryParams>,
) -> Result<Response, StatusCode> {
    debug!("Getting documentation for category: {}", category);

    // Parse category
    let parsed_category = match parse_category(&category) {
        Some(cat) => cat,
        None => {
            return Err(StatusCode::NOT_FOUND);
        }
    };

    // Get documentation from the system
    match get_documentation_from_system(&state).await {
        Ok(doc_collection) => {
            let category_docs = doc_collection.get_by_category(&parsed_category);

            if category_docs.is_empty() {
                return Err(StatusCode::NOT_FOUND);
            }

            // Filter by search query if provided
            let filtered_docs = if let Some(search) = params.search {
                category_docs
                    .into_iter()
                    .filter(|doc| {
                        doc.name.contains(&search)
                            || doc.description.contains(&search)
                            || doc.examples.iter().any(|ex| ex.contains(&search))
                    })
                    .collect()
            } else {
                category_docs
            };

            // Return response in requested format
            let accept_markdown = is_markdown_requested(&headers);
            if accept_markdown || params.format == Some("markdown".to_string()) {
                let markdown = generate_markdown_category(&category, &filtered_docs);

                Response::builder()
                    .header(header::CONTENT_TYPE, "text/markdown")
                    .body(markdown)
                    .map(IntoResponse::into_response)
                    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
            } else {
                // Default to JSON
                let response = CategoryDocumentation {
                    name: category.clone(),
                    description: get_category_description(&parsed_category),
                    parser_count: filtered_docs.len(),
                    parsers: filtered_docs
                        .iter()
                        .map(|doc| ParserDocumentationResponse::from(*doc))
                        .collect(),
                };

                Ok(Json(response).into_response())
            }
        }
        Err(e) => {
            warn!("Failed to get documentation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Get documentation for a specific parser
///
/// Returns documentation for a specific parser identified by category and name.
#[utoipa::path(
    get,
    path = "/docs/dsl/{category}/{name}",
    responses(
        (status = 200, description = "Parser documentation retrieved successfully", body = ParserDocumentationResponse),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "Parser not found", body = DocumentationErrorResponse),
        (status = 500, description = "Internal server error", body = DocumentationErrorResponse)
    ),
    params(
        ("category" = String, Path, description = "Category name (Expression, Statement, Handler, Type, or Definition)"),
        ("name" = String, Path, description = "Parser name"),
        DocumentationQueryParams
    )
)]
pub async fn get_parser_documentation(
    State(state): State<AppState>,
    _auth: AuthUser,
    Path((category, name)): Path<(String, String)>,
    headers: HeaderMap,
    Query(params): Query<DocumentationQueryParams>,
) -> Result<Response, StatusCode> {
    debug!(
        "Getting documentation for parser: {} in category {}",
        name, category
    );

    // Parse category
    let parsed_category = match parse_category(&category) {
        Some(cat) => cat,
        None => {
            return Err(StatusCode::NOT_FOUND);
        }
    };

    // Get documentation from the system
    match get_documentation_from_system(&state).await {
        Ok(doc_collection) => {
            // Find the parser in the specified category
            let category_docs = doc_collection.get_by_category(&parsed_category);
            let parser_doc = category_docs.iter().find(|doc| doc.name == name);

            match parser_doc {
                Some(doc) => {
                    // Return response in requested format
                    let accept_markdown = is_markdown_requested(&headers);
                    if accept_markdown || params.format == Some("markdown".to_string()) {
                        let markdown = generate_markdown_parser(doc);

                        Response::builder()
                            .header(header::CONTENT_TYPE, "text/markdown")
                            .body(markdown)
                            .map(IntoResponse::into_response)
                            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
                    } else {
                        // Default to JSON
                        let response = ParserDocumentationResponse::from(*doc);
                        Ok(Json(response).into_response())
                    }
                }
                None => Err(StatusCode::NOT_FOUND),
            }
        }
        Err(e) => {
            warn!("Failed to get documentation: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Helper function to get documentation from the system
async fn get_documentation_from_system(
    _state: &AppState,
) -> Result<DocumentationCollection, String> {
    // TODO: In a full implementation, we would access the System's documentation collection
    // For now, we'll return a mock collection for development

    // This is a temporary mock implementation
    let mut collection = DocumentationCollection::new();

    // Add some sample documentation for development
    use kairei_core::analyzer::doc_parser::DocBuilder;

    // Expression parsers
    let expr1 = DocBuilder::new("parse_binary_expression", ParserCategory::Expression)
        .description("Parses binary operations between expressions")
        .example("a + b")
        .example("x * (y - z)")
        .related_parser("parse_expression")
        .build();

    let expr2 = DocBuilder::new("parse_think", ParserCategory::Expression)
        .description("Parses a think expression for LLM invocation")
        .example("think(\"What is the capital of France?\")")
        .example("think(prompt: \"Generate ideas\", context: data) with { model: \"gpt-4\" }")
        .related_parser("parse_expression")
        .build();

    // Statement parsers
    let stmt1 = DocBuilder::new("parse_if_statement", ParserCategory::Statement)
        .description("Parses an if/else conditional statement")
        .example("if (condition) { /* code */ }")
        .example("if (x > 0) { return true; } else { return false; }")
        .related_parser("parse_statement")
        .build();

    // Handler parsers
    let handler1 = DocBuilder::new("parse_observe_handler", ParserCategory::Handler)
        .description("Parses an observe handler for event observations")
        .example("on UserMessage(text: String) { /* code */ }")
        .example("on Timer(interval: Duration) { /* code */ }")
        .related_parser("parse_handler")
        .build();

    // Add them to the collection
    collection.add(expr1);
    collection.add(expr2);
    collection.add(stmt1);
    collection.add(handler1);

    Ok(collection)
}

// Helper function to check if markdown is requested
fn is_markdown_requested(headers: &HeaderMap) -> bool {
    if let Some(accept) = headers.get(header::ACCEPT) {
        if let Ok(accept_str) = accept.to_str() {
            return accept_str.contains("text/markdown") || accept_str.contains("text/x-markdown");
        }
    }
    false
}

// Helper function to parse category string to ParserCategory
fn parse_category(category: &str) -> Option<ParserCategory> {
    match category.to_lowercase().as_str() {
        "expression" => Some(ParserCategory::Expression),
        "statement" => Some(ParserCategory::Statement),
        "handler" => Some(ParserCategory::Handler),
        "type" => Some(ParserCategory::Type),
        "definition" => Some(ParserCategory::Definition),
        other => Some(ParserCategory::Other(other.to_string())),
    }
}

// Helper function to get category description
fn get_category_description(category: &ParserCategory) -> String {
    match category {
        ParserCategory::Expression => {
            "Expression parsers handle values and operations in the KAIREI DSL".to_string()
        }
        ParserCategory::Statement => {
            "Statement parsers handle control flow, assignments, and other statements".to_string()
        }
        ParserCategory::Handler => {
            "Handler parsers handle event handlers like answer, observe, and react".to_string()
        }
        ParserCategory::Type => {
            "Type parsers handle type definitions and type checking".to_string()
        }
        ParserCategory::Definition => {
            "Definition parsers handle top-level constructs like world, agent, and sistence"
                .to_string()
        }
        ParserCategory::Other(name) => format!("Other parsers: {}", name),
    }
}

// Helper function to generate a documentation response
fn create_documentation_response(collection: &DocumentationCollection) -> DocumentationResponse {
    let mut by_category = HashMap::new();
    let mut categories = Vec::new();

    for category in collection.get_categories() {
        let category_name = category.to_string();
        categories.push(category_name.clone());

        let parsers = collection.get_by_category(category);
        let category_doc = CategoryDocumentation {
            name: category_name.clone(),
            description: get_category_description(category),
            parser_count: parsers.len(),
            parsers: parsers
                .iter()
                .map(|doc| ParserDocumentationResponse::from(*doc))
                .collect(),
        };

        by_category.insert(category_name, category_doc);
    }

    DocumentationResponse {
        total_parsers: collection.get_all().len(),
        categories,
        by_category,
    }
}

// Helper function to filter documentation by search query
fn filter_documentation_by_search(
    collection: &DocumentationCollection,
    query: &str,
) -> DocumentationCollection {
    let mut filtered = DocumentationCollection::new();

    for doc in collection.get_all() {
        if doc.name.contains(query)
            || doc.description.contains(query)
            || doc.examples.iter().any(|ex| ex.contains(query))
        {
            filtered.add(doc.clone());
        }
    }

    filtered
}

// Helper function to generate markdown for all documentation
fn generate_markdown_documentation(collection: &DocumentationCollection) -> String {
    let mut markdown = String::new();

    markdown.push_str("# KAIREI DSL Documentation\n\n");

    // Table of contents
    markdown.push_str("## Table of Contents\n\n");
    for category in collection.get_categories() {
        let category_name = category.to_string();
        let anchor = category_name.to_lowercase().replace(' ', "-");
        markdown.push_str(&format!("- [{}](#{})\n", category_name, anchor));
    }
    markdown.push('\n');

    // Categories
    for category in collection.get_categories() {
        let category_name = category.to_string();
        markdown.push_str(&format!("## {}\n\n", category_name));
        markdown.push_str(&format!("{}\n\n", get_category_description(category)));

        let parsers = collection.get_by_category(category);
        for parser in parsers {
            markdown.push_str(&format!("### {}\n\n", parser.name));
            markdown.push_str(&format!("{}\n\n", parser.description));

            if !parser.examples.is_empty() {
                markdown.push_str("#### Examples\n\n");
                for example in &parser.examples {
                    markdown.push_str(&format!("```kairei\n{}\n```\n\n", example));
                }
            }

            if !parser.related_parsers.is_empty() {
                markdown.push_str("#### Related Parsers\n\n");
                for related in &parser.related_parsers {
                    markdown.push_str(&format!("- {}\n", related));
                }
                markdown.push('\n');
            }

            if let Some(deprecated) = &parser.deprecated {
                markdown.push_str(&format!("> **Deprecated:** {}\n\n", deprecated));
            }
        }
    }

    markdown
}

// Helper function to generate markdown for a category
fn generate_markdown_category(
    category_name: &str,
    parsers: &[&kairei_core::analyzer::ParserDocumentation],
) -> String {
    let mut markdown = String::new();

    markdown.push_str(&format!("# {} Parsers\n\n", category_name));

    // Table of contents
    markdown.push_str("## Parsers\n\n");
    for parser in parsers {
        let anchor = parser.name.to_lowercase().replace(' ', "-");
        markdown.push_str(&format!("- [{}](#{})\n", parser.name, anchor));
    }
    markdown.push('\n');

    // Parsers
    for parser in parsers {
        markdown.push_str(&format!("## {}\n\n", parser.name));
        markdown.push_str(&format!("{}\n\n", parser.description));

        if !parser.examples.is_empty() {
            markdown.push_str("### Examples\n\n");
            for example in &parser.examples {
                markdown.push_str(&format!("```kairei\n{}\n```\n\n", example));
            }
        }

        if !parser.related_parsers.is_empty() {
            markdown.push_str("### Related Parsers\n\n");
            for related in &parser.related_parsers {
                markdown.push_str(&format!("- {}\n", related));
            }
            markdown.push('\n');
        }

        if let Some(deprecated) = &parser.deprecated {
            markdown.push_str(&format!("> **Deprecated:** {}\n\n", deprecated));
        }
    }

    markdown
}

// Helper function to generate markdown for a parser
fn generate_markdown_parser(parser: &kairei_core::analyzer::ParserDocumentation) -> String {
    let mut markdown = String::new();

    markdown.push_str(&format!("# {}\n\n", parser.name));
    markdown.push_str(&format!("**Category:** {}\n\n", parser.category));
    markdown.push_str(&format!("{}\n\n", parser.description));

    if !parser.examples.is_empty() {
        markdown.push_str("## Examples\n\n");
        for example in &parser.examples {
            markdown.push_str(&format!("```kairei\n{}\n```\n\n", example));
        }
    }

    if !parser.related_parsers.is_empty() {
        markdown.push_str("## Related Parsers\n\n");
        for related in &parser.related_parsers {
            markdown.push_str(&format!("- {}\n", related));
        }
        markdown.push('\n');
    }

    if let Some(deprecated) = &parser.deprecated {
        markdown.push_str(&format!("> **Deprecated:** {}\n\n", deprecated));
    }

    markdown
}
