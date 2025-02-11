use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    combinator::map,
    error::context,
};

use super::token::{ParserResult, Token};

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_whitespace(input: &str) -> ParserResult<Token> {
    context(
        "whitespace expected",
        map(take_while1(|c| c == ' ' || c == '\t'), |ws: &str| {
            Token::Whitespace(ws.to_string())
        }),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_newline(input: &str) -> ParserResult<Token> {
    context(
        "newline expected",
        map(alt((tag("\r\n"), tag("\n"))), |_| Token::Newline),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_whitespace() {
        let input = "   hello";
        let (rest, token) = parse_whitespace(input).unwrap();
        assert_eq!(token, Token::Whitespace("   ".to_string()));
        assert_eq!(rest, "hello");

        let input = "\t\t  hello";
        let (rest, token) = parse_whitespace(input).unwrap();
        assert_eq!(token, Token::Whitespace("\t\t  ".to_string()));
        assert_eq!(rest, "hello");
    }

    #[test]
    fn test_newline() {
        let input = "\nhello";
        let (rest, token) = parse_newline(input).unwrap();
        assert_eq!(token, Token::Newline);
        assert_eq!(rest, "hello");

        let input = "\r\nworld";
        let (rest, token) = parse_newline(input).unwrap();
        assert_eq!(token, Token::Newline);
        assert_eq!(rest, "world");
    }

    #[test]
    fn test_error() {
        let input = "hello";
        let result = parse_whitespace(input);
        assert!(result.is_err());

        let input = "hello";
        let result = parse_newline(input);
        assert!(result.is_err());
    }
}
