use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::not_line_ending,
    combinator::map,
    error::context,
    sequence::{delimited, preceded},
};

use super::token::{CommentType, ParserResult, Token};

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_line_comment(input: &str) -> ParserResult<Token> {
    context(
        "line comment",
        map(
            preceded(tag("//"), not_line_ending),
            |parse_comment: &str| Token::Comment {
                content: parse_comment.trim().to_string(),
                comment_type: CommentType::Line,
            },
        ),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_block_comment(input: &str) -> ParserResult<Token> {
    context(
        "block comment",
        map(
            delimited(tag("/*"), take_until("*/"), tag("*/")),
            |content: &str| Token::Comment {
                content: content.to_string(),
                comment_type: CommentType::Block,
            },
        ),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_line_documentation_comment(input: &str) -> ParserResult<Token> {
    context(
        "line document comment",
        map(
            preceded(tag("///"), not_line_ending),
            |parse_comment: &str| Token::Comment {
                content: parse_comment.trim().to_string(),
                comment_type: CommentType::DocumentationLine,
            },
        ),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_block_documentation_comment(input: &str) -> ParserResult<Token> {
    context(
        "block document comment",
        map(
            delimited(tag("/**"), take_until("*/"), tag("*/")),
            |content: &str| Token::Comment {
                content: content.to_string(),
                comment_type: CommentType::DocumentationBlock,
            },
        ),
    )(input)
}

#[tracing::instrument(level = "debug", skip(input))]
pub fn parse_comment(input: &str) -> ParserResult<Token> {
    context(
        "comment",
        alt((
            parse_block_documentation_comment, // /** */ を次にチェック
            parse_line_documentation_comment,  // /// を最初にチェック
            parse_block_comment,               // /* */ を次にチェック
            parse_line_comment,                // // を最後にチェック
        )),
    )(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_comment() {
        let input = "// This is a line parse_comment\ncode";
        let (rest, token) = parse_comment(input).unwrap();
        assert_eq!(
            token,
            Token::Comment {
                content: "This is a line parse_comment".to_string(),
                comment_type: CommentType::Line,
            }
        );
        assert_eq!(rest, "\ncode");
    }

    #[test]
    fn test_block_comment() {
        let input = "/* This is a\n block parse_comment */code";
        let (rest, token) = parse_comment(input).unwrap();
        assert_eq!(
            token,
            Token::Comment {
                content: " This is a\n block parse_comment ".to_string(),
                comment_type: CommentType::Block,
            }
        );
        assert_eq!(rest, "code");
    }

    #[test]
    fn test_line_documentation_comment() {
        let input = "/// This is a doc parse_comment\nfn test() {}";
        let (rest, token) = parse_comment(input).unwrap();
        assert_eq!(
            token,
            Token::Comment {
                content: "This is a doc parse_comment".to_string(),
                comment_type: CommentType::DocumentationLine,
            }
        );
        assert_eq!(rest, "\nfn test() {}");
    }

    #[test]
    fn test_documentation_comment() {
        let input = "/** This is a\n * documentation parse_comment\n */code";
        let (rest, token) = parse_comment(input).unwrap();
        assert_eq!(
            token,
            Token::Comment {
                content: " This is a\n * documentation parse_comment\n ".to_string(),
                comment_type: CommentType::DocumentationBlock,
            }
        );
        assert_eq!(rest, "code");
    }

    #[test]
    fn test_nested_looking_comment() {
        let input = "/* outer /* not nested */ */";
        let (_, token) = parse_comment(input).unwrap();
        assert_eq!(
            token,
            Token::Comment {
                content: " outer /* not nested ".to_string(),
                comment_type: CommentType::Block,
            }
        );
    }
}
