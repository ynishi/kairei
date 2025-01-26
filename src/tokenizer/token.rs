use nom::{
    branch::alt,
    bytes::complete::{take_while, take_while1},
    combinator::recognize,
    error::{context, VerboseError},
    sequence::pair,
    IResult,
};
use thiserror::Error;

use super::{
    comment::parse_comment,
    keyword::{parse_keyword, Keyword},
    literal::{parse_literal, Literal},
    symbol::{parse_delimiter, parse_operator, Delimiter, Operator},
    types::{parse_types, Type},
    whitespace::{parse_newline, parse_whitespace},
};

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Keyword(Keyword),
    // Identifiers
    Identifier(String),
    // Types
    Type(Type),
    // Symbols
    Operator(Operator),
    Delimiter(Delimiter),
    // Literals
    Literal(Literal),
    // Formatting
    Whitespace(String),
    Newline,
    Comment {
        content: String,
        comment_type: CommentType,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentType {
    Line,               // //
    Block,              // /* */
    DocumentationLine,  // ///
    DocumentationBlock, // /** */
}

#[derive(Debug, Clone)]
pub struct Tokenizer {
    current_position: usize,
    current_line: usize,
    current_column: usize,
}

impl Tokenizer {
    pub fn new() -> Self {
        Self {
            current_position: 0,
            current_line: 1,   // 1-based
            current_column: 1, // 1-based
        }
    }

    #[tracing::instrument(level = "debug", skip(input))]
    pub fn tokenize(&mut self, input: &str) -> TokenizerResult<Vec<TokenSpan>> {
        let mut tokens = Vec::new();
        let mut remaining = input;

        while !remaining.is_empty() {
            let start_position = self.current_position;
            let start_line = self.current_line;
            let start_column = self.current_column;

            let result = alt((
                // Formatting
                parse_whitespace,
                parse_newline,
                // Literals
                parse_literal,
                // Comments
                parse_comment,
                // Code elements
                parse_keyword,
                parse_operator,
                parse_delimiter,
                parse_types,
                parse_identifier,
            ))(remaining);

            match result {
                Ok((new_remaining, token)) => {
                    let consumed = &remaining[..(remaining.len() - new_remaining.len())];
                    self.update_position(consumed);

                    tokens.push(TokenSpan {
                        token,
                        start: start_position,
                        end: self.current_position,
                        line: start_line,
                        column: start_column,
                    });

                    remaining = new_remaining;
                }
                Err(e) => {
                    let found = remaining.chars().take(20).collect::<String>();
                    let span = Span {
                        start: self.current_position,
                        end: self.current_position + 1,
                        line: self.current_line,
                        column: self.current_column,
                    };
                    let error = match e {
                        nom::Err::Incomplete(e) => TokenizerError::ParseError {
                            message: format!("Incomplete input, {:?}", e),
                            found,
                            span,
                        },
                        nom::Err::Error(e) | nom::Err::Failure(e) => TokenizerError::ParseError {
                            message: nom::error::convert_error(remaining, e).to_string(),
                            found,
                            span,
                        },
                    };
                    tracing::error!("{}", error);
                    return Err(error);
                }
            }
        }

        Ok(tokens)
    }

    fn update_position(&mut self, text: &str) {
        for c in text.chars() {
            self.current_position += c.len_utf8();
            if c == '\n' {
                self.current_line += 1;
                self.current_column = 1;
            } else {
                self.current_column += 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenSpan {
    pub token: Token,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "line: {}, column: {}, start: {}, end: {}",
            self.line, self.column, self.start, self.end
        )
    }
}

#[tracing::instrument(level = "debug", skip(input))]
fn parse_identifier(input: &str) -> ParserResult<Token> {
    let (input, id) = context(
        "identifier",
        recognize(pair(
            take_while1(|c: char| c.is_alphabetic() || c == '_'),
            take_while(|c: char| c.is_alphanumeric() || c == '_'),
        )),
    )(input)?;

    // Check if identifier is not a specials
    if let Ok(kw) = Keyword::try_from(id) {
        return Ok((input, Token::Keyword(kw)));
    }
    if let Ok(ty) = Type::try_from(id) {
        return Ok((input, Token::Type(ty)));
    }

    Ok((input, Token::Identifier(id.to_string())))
}

pub type ParserResult<'a, T> = IResult<&'a str, T, VerboseError<&'a str>>;

pub type TokenizerResult<'a, T> = Result<T, TokenizerError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum TokenizerError {
    #[error("Parse error: {message} at position {span}")]
    ParseError {
        message: String,
        found: String,
        span: Span,
    },
}

#[cfg(test)]
mod tests {

    use crate::tokenizer::literal::StringPart;

    use super::*;

    #[test]
    fn test_identifier_for_keyword() {
        let input = "micro";
        let (rest, token) = parse_identifier(input).unwrap();
        assert_eq!(token, Token::Keyword(Keyword::Micro));
        assert_eq!(rest, "");
    }

    #[test]
    fn test_identifier() {
        let input = "my_var123 other";
        let (rest, token) = parse_identifier(input).unwrap();
        assert_eq!(token, Token::Identifier("my_var123".to_string()));
        assert_eq!(rest, " other");
    }

    #[test]
    fn test_tokenizer_with_position() {
        let mut tokenizer = Tokenizer::new();
        let input = "x\nother";
        let tokens = tokenizer.tokenize(input).unwrap();

        assert_eq!(tokens[0].line, 1);
        assert_eq!(tokens[0].column, 1);
        assert_eq!(tokens[0].token, Token::Identifier("x".to_string()));

        // 2行目のtokenを確認
        let print_token = &tokens[2];
        assert_eq!(print_token.line, 2);
        assert_eq!(print_token.column, 1);
    }

    #[test]
    fn test_world_block() {
        let mut tokenizer = Tokenizer::new();
        let input = r#"world TravelPlanning {
               policy "Consider budget constraints and optimize value for money"
           }"#;

        let tokens = tokenizer.tokenize(input).unwrap();

        // 期待されるトークンの確認
        let important_tokens: Vec<_> = tokens
            .iter()
            .filter(|t| !matches!(t.token, Token::Whitespace(_) | Token::Newline))
            .collect();

        assert!(matches!(
            important_tokens[0].token,
            Token::Keyword(Keyword::World)
        ));
        assert!(
            matches!(important_tokens[1].token, Token::Identifier(ref s) if s == "TravelPlanning")
        );
        assert!(matches!(
            important_tokens[2].token,
            Token::Delimiter(Delimiter::OpenBrace)
        ));
        assert!(matches!(
            important_tokens[3].token,
            Token::Keyword(Keyword::Policy)
        ));
        assert!(matches!(important_tokens[4].token,
            Token::Literal(Literal::String(ref parts))
            if parts.len() == 1
            && matches!(parts[0], StringPart::Literal(ref s)
                if s == "Consider budget constraints and optimize value for money")));
    }

    #[test]
    fn test_micro_block() {
        let mut tokenizer = Tokenizer::new();
        let input = r#"micro TravelPlanner {
               state {
                   current_plan: String = "none",
                   planning_stage: String = "none"
               }
           }"#;

        let tokens = tokenizer.tokenize(input).unwrap();

        let micro_tokens = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::Micro)))
            .count();
        assert_eq!(micro_tokens, 1);

        let state_tokens = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::State)))
            .count();
        assert_eq!(state_tokens, 1);

        let identifiers = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Identifier(_)))
            .collect::<Vec<_>>();
        assert!(identifiers
            .iter()
            .any(|t| matches!(t.token, Token::Identifier(ref s) if s == "current_plan")));
        assert!(identifiers
            .iter()
            .any(|t| matches!(t.token, Token::Identifier(ref s) if s == "planning_stage")));
    }

    #[test]
    fn test_answer_block() {
        let mut tokenizer = Tokenizer::new();
        let input = r#"answer {
               on request PlanTrip(destination: String, budget: Float) -> Result<String, Error> {
                   return Ok(plan)
               }
           }"#;

        let tokens = tokenizer.tokenize(input).unwrap();

        let answer_tokens = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::Answer)))
            .count();
        assert_eq!(answer_tokens, 1);

        let request_tokens = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::Request)))
            .count();
        assert_eq!(request_tokens, 1);

        let plantrip_tokens = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Identifier(ref s) if s == "PlanTrip"))
            .count();
        assert_eq!(plantrip_tokens, 1);
    }

    #[test]
    fn test_complete_dsl() {
        let mut tokenizer = Tokenizer::new();
        let input = r#"
           world TravelPlanning {
               policy "Consider budget constraints"
               policy "Ensure traveler safety"
           }

           micro TravelPlanner {
               state {
                   current_plan: String = "none"
               }
               answer {
                   on request PlanTrip(destination: String) -> Result<String, Error> {
                       return Ok(plan)
                   }
               }
           }"#;

        let result = tokenizer.tokenize(input);
        assert!(result.is_ok());

        let tokens = result.unwrap();
        let world_count = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::World)))
            .count();
        assert_eq!(world_count, 1);

        let micro_count = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::Micro)))
            .count();
        assert_eq!(micro_count, 1);

        let state_count = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::State)))
            .count();
        assert_eq!(state_count, 1);

        let answer_count = tokens
            .iter()
            .filter(|t| matches!(t.token, Token::Keyword(Keyword::Answer)))
            .count();
        assert_eq!(answer_count, 1);
    }
}
