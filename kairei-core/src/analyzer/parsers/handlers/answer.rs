use super::super::{
    super::{core::*, prelude::*},
    *,
};
use crate::analyzer::parsers::handlers::parse_parameters;
use crate::ast;
use crate::{
    analyzer::parsers::{
        expression::{parse_dot, parse_with_keyword},
        statement::parse_statements,
        types::parse_type_info,
    },
    tokenizer::{keyword::Keyword, symbol::Operator, token::Token},
};

/// Answer Block Handler Implementation
///
/// The answer block defines how an agent responds to explicit requests.
/// It enforces read-only access to state and provides type-safe responses.
///
/// # Features
/// - Type-safe request handling
/// - Read-only state access
/// - Quality constraints specification
/// - Error handling with Result type
///
/// # Example
/// ```text
/// answer {
///     on request GetProfile(user_id: String) -> Result<Profile> {
///         with {
///             strictness: 0.8,
///             stability: 0.9,
///             latency: 1000
///         }
///         // Handler implementation
///     }
/// }
/// ```
///
/// # Quality Constraints
/// - `strictness`: Accuracy requirement (0.0-1.0)
/// - `stability`: Consistency requirement (0.0-1.0)
/// - `latency`: Response time limit in milliseconds
pub fn parse_answer() -> impl Parser<Token, ast::AnswerDef> {
    with_context(
        map(
            tuple4(
                as_unit(parse_answer_keyword()),
                as_unit(parse_open_brace()),
                many(parse_request_handler()),
                as_unit(parse_close_brace()),
            ),
            |(_, _, handlers, _)| ast::AnswerDef { handlers },
        ),
        "answer",
    )
}

/// Request Handler Parser
///
/// Parses individual request handlers within an answer block.
/// Each handler defines a specific request type, parameters, return type,
/// and optional quality constraints.
///
/// # Handler Structure
/// - Request type (query, action, or custom)
/// - Parameters with types
/// - Return type (must be Result)
/// - Optional quality constraints
/// - Handler implementation block
///
/// # Example
/// ```text
/// on request GetData(id: String) -> Result<Data> {
///     with {
///         strictness: 0.9
///     }
///     // Handler implementation
/// }
/// ```
pub fn parse_request_handler() -> impl Parser<Token, ast::RequestHandler> {
    with_context(
        map(
            tuple6(
                as_unit(parse_on_keyword()),
                parse_request_type(),
                parse_parameters(),
                preceded(as_unit(parse_arrow()), parse_type_info()),
                optional(parse_constraints()),
                parse_statements(),
            ),
            |(_, request_type, parameters, return_type, constraints, block)| ast::RequestHandler {
                request_type,
                parameters,
                return_type,
                constraints,
                block: ast::HandlerBlock { statements: block },
            },
        ),
        "request handler",
    )
}

/// Request Type Parser
///
/// Parses the type of request being handled. Supports three types:
/// - Query: For data retrieval operations
/// - Action: For state-changing operations
/// - Custom: For user-defined request types
///
/// # Examples
/// ```text
/// query.GetUserData
/// action.UpdateProfile
/// request CustomOperation
/// ```
pub fn parse_request_type() -> impl Parser<Token, ast::RequestType> {
    with_context(
        choice(vec![
            Box::new(map(
                preceded(
                    as_unit(parse_query_keyword()),
                    preceded(as_unit(parse_dot()), parse_identifier()),
                ),
                |query_type| ast::RequestType::Query { query_type },
            )),
            Box::new(map(
                preceded(
                    as_unit(parse_action_keyword()),
                    preceded(as_unit(parse_dot()), parse_identifier()),
                ),
                |action_type| ast::RequestType::Action { action_type },
            )),
            Box::new(map(
                preceded(as_unit(parse_request()), parse_identifier()),
                ast::RequestType::Custom,
            )),
        ]),
        "request type",
    )
}

fn parse_request() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Request)), "request keyword")
}

/// Quality Constraints Parser
///
/// Parses quality constraints for request handlers.
/// These constraints define requirements for:
/// - Response accuracy (strictness)
/// - Response consistency (stability)
/// - Response time (latency)
///
/// # Example
/// ```text
/// with {
///     strictness: 0.9,  // 90% accuracy requirement
///     stability: 0.8,   // 80% consistency requirement
///     latency: 1000     // 1 second response time limit
/// }
/// ```
pub fn parse_constraints() -> impl Parser<Token, ast::Constraints> {
    with_context(
        map(
            preceded(
                as_unit(parse_with_keyword()),
                map(
                    delimited(
                        as_unit(parse_open_brace()),
                        separated_list(parse_constraint_item(), as_unit(parse_comma())),
                        as_unit(parse_close_brace()),
                    ),
                    |items| {
                        let mut constraints = ast::Constraints {
                            strictness: None,
                            stability: None,
                            latency: None,
                        };
                        for (key, value) in items {
                            match (key.as_str(), value) {
                                ("strictness", ast::Literal::Float(v)) => {
                                    constraints.strictness = Some(v)
                                }
                                ("stability", ast::Literal::Float(v)) => {
                                    constraints.stability = Some(v)
                                }
                                ("latency", ast::Literal::Integer(v)) => {
                                    constraints.latency = Some(v as u32)
                                }
                                _ => {}
                            }
                        }
                        constraints
                    },
                ),
            ),
            |constraints| constraints,
        ),
        "constraints",
    )
}

fn parse_constraint_item() -> impl Parser<Token, (String, ast::Literal)> {
    with_context(
        map(
            tuple3(parse_identifier(), as_unit(parse_colon()), parse_literal()),
            |(key, _, value)| (key, value),
        ),
        "constraint item",
    )
}

fn parse_answer_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Answer)), "answer keyword")
}

fn parse_query_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Query)), "query keyword")
}

fn parse_action_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::Action)), "action keyword")
}

// Support thin and fat arrow
fn parse_arrow() -> impl Parser<Token, Token> {
    with_context(
        choice(vec![
            Box::new(equal(Token::Operator(Operator::Arrow))),
            Box::new(equal(Token::Operator(Operator::ThinArrow))),
        ]),
        "arrow operator",
    )
}
