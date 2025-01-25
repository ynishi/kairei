use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::multispace1,
    combinator::recognize,
    sequence::{pair, preceded},
    IResult,
};

use super::{
    keyword::{parse_keyword, Keyword},
    symbol::{delimiter, operator, Delimiter, Operator},
    types::{parse_types, Type},
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
    // Formatting
    Whitespace(String),
    Newline,
    Comment(String),
}

#[derive(Debug, Clone)]
pub struct TokenSpan {
    pub token: Token,
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

// Parser for identifiers
fn identifier(input: &str) -> IResult<&str, Token> {
    let (input, id) = recognize(pair(
        take_while1(|c: char| c.is_alphabetic() || c == '_'),
        take_while(|c: char| c.is_alphanumeric() || c == '_'),
    ))(input)?;

    // Check if identifier is not a specials
    if let Ok(kw) = Keyword::try_from(id) {
        return Ok((input, Token::Keyword(kw)));
    }
    if let Ok(ty) = Type::try_from(id) {
        return Ok((input, Token::Type(ty)));
    }

    Ok((input, Token::Identifier(id.to_string())))
}

// Parser for whitespace
fn whitespace(input: &str) -> IResult<&str, Token> {
    let (input, ws) = multispace1(input)?;
    Ok((input, Token::Whitespace(ws.to_string())))
}

// Parser for newlines
fn newline(input: &str) -> IResult<&str, Token> {
    let (input, _) = alt((tag("\n"), tag("\r\n")))(input)?;
    Ok((input, Token::Newline))
}

// Parser for comments
fn comment(input: &str) -> IResult<&str, Token> {
    let (input, comment) = preceded(tag("//"), take_while(|c| c != '\n'))(input)?;
    Ok((input, Token::Comment(comment.trim().to_string())))
}

// Main tokenizer function
pub fn tokenize(input: &str) -> IResult<&str, Vec<TokenSpan>> {
    let mut tokens = Vec::new();
    let mut current_line = 1;
    let mut current_column = 1;
    let mut remaining = input;

    while !remaining.is_empty() {
        let result = alt((
            parse_keyword,
            parse_types,
            identifier,
            operator,
            delimiter,
            comment,
            whitespace,
            newline,
        ))(remaining);

        match result {
            Ok((rest, token)) => {
                let token_length = remaining.len() - rest.len();
                let token_span = TokenSpan {
                    token: token.clone(),
                    start: input.len() - remaining.len(),
                    end: input.len() - rest.len(),
                    line: current_line,
                    column: current_column,
                };

                // Update position tracking
                if token == Token::Newline {
                    current_line += 1;
                    current_column = 1;
                } else {
                    current_column += token_length;
                }

                tokens.push(token_span);
                remaining = rest;
            }
            Err(e) => return Err(e),
        }
    }

    Ok(("", tokens))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn test_keywords() {
        let test_cases = [
            ("micro Test", Keyword::Micro),
            ("if Test", Keyword::If),
            ("return Test", Keyword::Return),
            ("await Test", Keyword::Await),
            ("on Test", Keyword::On),
            ("with Test", Keyword::With),
        ];

        for (input, expected_keyword) in test_cases.iter() {
            let (rest, token) = parse_keyword(input).unwrap();
            assert_eq!(token, Token::Keyword(expected_keyword.clone()));
            assert_eq!(rest, " Test");
        }
    }

    // check if all keywords are parsed correctly
    #[test]
    fn test_all_keyword() {
        for keyword_string in Keyword::iter().map(|t| t.to_string()) {
            let (rest, token) = parse_keyword(&keyword_string).unwrap();
            let k = Keyword::from_str(&keyword_string).unwrap();
            assert_eq!(token, Token::Keyword(k));
            assert_eq!(rest, "");
        }
    }

    #[test]
    fn test_types() {
        let test_cases = [
            ("Ok Test", Type::Ok),
            ("Err Test", Type::Err),
            ("Int Test", Type::Int),
            ("Float Test", Type::Float),
            ("String Test", Type::String),
            ("Bool Test", Type::Bool),
            ("Any Test", Type::Any),
        ];

        for (input, expected_type) in test_cases.iter() {
            let (rest, token) = parse_types(input).unwrap();
            assert_eq!(token, Token::Type(expected_type.clone()));
            assert_eq!(rest, " Test");
        }
    }

    #[test]
    fn test_all_types() {
        for type_string in Type::iter().map(|t| t.to_string()) {
            let (rest, token) = parse_types(&type_string).unwrap();
            let t = Type::from_str(&type_string).unwrap();
            assert_eq!(token, Token::Type(t));
            assert_eq!(rest, "");
        }
    }

    #[test]
    fn test_identifier_for_keyword() {
        let input = "micro";
        let (rest, token) = identifier(input).unwrap();
        assert_eq!(token, Token::Keyword(Keyword::Micro));
        assert_eq!(rest, "");
    }

    #[test]
    fn test_identifier() {
        let input = "my_var123 other";
        let (rest, token) = identifier(input).unwrap();
        assert_eq!(token, Token::Identifier("my_var123".to_string()));
        assert_eq!(rest, " other");
    }
}
