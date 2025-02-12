use super::super::{core::*, prelude::*};
use crate::ast;
use crate::tokenizer::{
    keyword::Keyword,
    literal::{Literal, StringPart},
    symbol::Delimiter,
    token::Token,
};
use std::{collections::HashMap, time::Duration};

// 基本的なパーサー
pub fn parse_identifier() -> impl Parser<Token, String> {
    with_context(
        satisfy(|token| match token {
            Token::Identifier(s) => Some(s.clone()),
            _ => None,
        }),
        "identifier",
    )
}

pub fn parse_literal() -> impl Parser<Token, ast::Literal> {
    with_context(
        choice(vec![
            Box::new(parse_float()),
            Box::new(parse_integer()),
            Box::new(parse_string()),
            Box::new(parse_boolean()),
            Box::new(parse_duration()),
            Box::new(parse_list()),
            Box::new(parse_map()),
            Box::new(parse_retry()),
            Box::new(parse_null()),
        ]),
        "literal",
    )
}

// 区切り文字パーサー
pub fn parse_comma() -> impl Parser<Token, Token> {
    with_context(equal(Token::Delimiter(Delimiter::Comma)), "comma")
}

pub fn parse_semicolon() -> impl Parser<Token, Token> {
    with_context(equal(Token::Delimiter(Delimiter::Semicolon)), "semicolon")
}

pub fn parse_colon() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::Colon))
}

pub fn parse_equal() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::Equal))
}

pub fn parse_open_paren() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::OpenParen))
}

pub fn parse_close_paren() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::CloseParen))
}

pub fn parse_open_bracket() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::OpenBracket))
}

pub fn parse_close_bracket() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::CloseBracket))
}

pub fn parse_open_brace() -> impl Parser<Token, Token> {
    with_context(equal(Token::Delimiter(Delimiter::OpenBrace)), "open brace")
}

pub fn parse_close_brace() -> impl Parser<Token, Token> {
    equal(Token::Delimiter(Delimiter::CloseBrace))
}

// リテラルパーサー
fn parse_float() -> impl Parser<Token, ast::Literal> {
    with_context(map(parse_f64(), ast::Literal::Float), "float")
}

fn parse_integer() -> impl Parser<Token, ast::Literal> {
    with_context(map(parse_i64(), ast::Literal::Integer), "integer")
}

fn parse_string() -> impl Parser<Token, ast::Literal> {
    with_context(
        satisfy(|token| match token {
            Token::Literal(Literal::String(parts)) => {
                if parts.len() == 1 {
                    match &parts[0] {
                        StringPart::Literal(s) => Some(ast::Literal::String(s.clone())),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            _ => None,
        }),
        "string",
    )
}

fn parse_boolean() -> impl Parser<Token, ast::Literal> {
    choice(vec![Box::new(parse_true()), Box::new(parse_false())])
}

fn parse_true() -> impl Parser<Token, ast::Literal> {
    parse_bool(true)
}

fn parse_false() -> impl Parser<Token, ast::Literal> {
    parse_bool(false)
}

fn parse_bool(b: bool) -> impl Parser<Token, ast::Literal> {
    map(equal(Token::Literal(Literal::Boolean(b))), move |_| {
        ast::Literal::Boolean(b)
    })
}

pub fn parse_on_keyword() -> impl Parser<Token, Token> {
    with_context(equal(Token::Keyword(Keyword::On)), "on keyword")
}

pub fn parse_ok_ident() -> impl Parser<Token, Token> {
    with_context(equal(Token::Identifier("Ok".to_string())), "Ok")
}

pub fn parse_err_ident() -> impl Parser<Token, Token> {
    with_context(equal(Token::Identifier("Err".to_string())), "Err")
}

pub fn parse_list() -> impl Parser<Token, ast::Literal> {
    with_context(
        map(
            delimited(
                as_unit(parse_open_bracket()),
                separated_list(lazy(parse_literal), as_unit(parse_comma())),
                as_unit(parse_close_bracket()),
            ),
            ast::Literal::List,
        ),
        "list",
    )
}

fn parse_map() -> impl Parser<Token, ast::Literal> {
    with_context(
        map(
            delimited(
                as_unit(parse_open_brace()),
                separated_list(parse_map_entry(), as_unit(parse_comma())),
                as_unit(parse_close_brace()),
            ),
            |entries| {
                let mut map = HashMap::new();
                for (key, value) in entries {
                    map.insert(key, value);
                }
                ast::Literal::Map(map)
            },
        ),
        "map",
    )
}

fn parse_map_entry() -> impl Parser<Token, (String, ast::Literal)> {
    with_context(
        map(
            tuple3(
                parse_identifier(),
                as_unit(parse_colon()),
                lazy(parse_literal),
            ),
            |(key, _, value)| (key, value),
        ),
        "map entry",
    )
}

fn parse_retry() -> impl Parser<Token, ast::Literal> {
    with_context(map(parse_retry_config(), ast::Literal::Retry), "retry")
}

fn parse_retry_config() -> impl Parser<Token, ast::RetryConfig> {
    with_context(
        map(
            tuple3(
                as_unit(parse_retry_ident()),
                parse_u64(),
                parse_retry_delay(),
            ),
            |(_, max_attempts, delay)| ast::RetryConfig {
                max_attempts,
                delay,
            },
        ),
        "retry config",
    )
}

fn parse_retry_ident() -> impl Parser<Token, Token> {
    with_context(equal(Token::Identifier("Retry".to_string())), "Retry")
}

fn parse_retry_delay() -> impl Parser<Token, ast::RetryDelay> {
    with_context(
        choice(vec![
            Box::new(parse_fixed_delay()),
            Box::new(parse_exponential_delay()),
        ]),
        "retry delay",
    )
}

fn parse_fixed_delay() -> impl Parser<Token, ast::RetryDelay> {
    with_context(
        map(
            preceded(as_unit(parse_fixed_ident()), parse_u64()),
            ast::RetryDelay::Fixed,
        ),
        "Fixed",
    )
}

fn parse_fixed_ident() -> impl Parser<Token, Token> {
    with_context(equal(Token::Identifier("Fixed".to_string())), "Fixed")
}

fn parse_exponential_delay() -> impl Parser<Token, ast::RetryDelay> {
    with_context(
        map(
            preceded(
                as_unit(parse_exponential_ident()),
                tuple3(parse_u64(), as_unit(parse_comma()), parse_u64()),
            ),
            |(initial, _, max)| ast::RetryDelay::Exponential { initial, max },
        ),
        "Exponential",
    )
}

fn parse_exponential_ident() -> impl Parser<Token, Token> {
    with_context(
        equal(Token::Identifier("Exponential".to_string())),
        "Exponential",
    )
}

fn parse_null() -> impl Parser<Token, ast::Literal> {
    map(equal(Token::Literal(Literal::Null)), |_| ast::Literal::Null)
}

// 数値パーサー
pub fn parse_f64() -> impl Parser<Token, f64> {
    satisfy(|token| match token {
        Token::Literal(Literal::Float(n)) => Some(*n),
        _ => None,
    })
}

pub fn parse_i64() -> impl Parser<Token, i64> {
    satisfy(|token| match token {
        Token::Literal(Literal::Integer(n)) => Some(*n),
        _ => None,
    })
}

pub fn parse_u64() -> impl Parser<Token, u64> {
    satisfy(|token| match token {
        Token::Literal(Literal::Integer(n)) => Some(*n as u64),
        _ => None,
    })
}

pub fn parse_usize() -> impl Parser<Token, usize> {
    satisfy(|token| match token {
        Token::Literal(Literal::Integer(n)) => Some(*n as usize),
        _ => None,
    })
}

fn parse_duration() -> impl Parser<Token, ast::Literal> {
    choice(vec![
        Box::new(parse_duration_millis()),
        Box::new(parse_duration_sec()),
        Box::new(parse_duration_min()),
        Box::new(parse_duration_hour()),
    ])
}

// unitをまとめて取得するヘルパー関数
#[allow(dead_code)]
fn parse_duration_unit() -> impl Parser<Token, String> {
    choice(vec![
        Box::new(parse_ms()),
        Box::new(parse_sec()),
        Box::new(parse_min()),
        Box::new(parse_hour()),
    ])
}

fn parse_duration_millis() -> impl Parser<Token, ast::Literal> {
    map(tuple2(parse_u64(), parse_ms()), |(value, _)| {
        ast::Literal::Duration(Duration::from_millis(value))
    })
}

fn parse_duration_sec() -> impl Parser<Token, ast::Literal> {
    map(tuple2(parse_u64(), parse_sec()), |(value, _)| {
        ast::Literal::Duration(Duration::from_secs(value))
    })
}

fn parse_duration_min() -> impl Parser<Token, ast::Literal> {
    map(tuple2(parse_u64(), parse_min()), |(value, _)| {
        ast::Literal::Duration(Duration::from_secs(value * 60))
    })
}

fn parse_duration_hour() -> impl Parser<Token, ast::Literal> {
    map(tuple2(parse_u64(), parse_hour()), |(value, _)| {
        ast::Literal::Duration(Duration::from_secs(value * 60 * 60))
    })
}

fn parse_ms() -> impl Parser<Token, String> {
    expected(parse_identifier(), "ms".to_string())
}

fn parse_sec() -> impl Parser<Token, String> {
    expected(parse_identifier(), "s".to_string())
}

fn parse_min() -> impl Parser<Token, String> {
    expected(parse_identifier(), "min".to_string())
}

fn parse_hour() -> impl Parser<Token, String> {
    expected(parse_identifier(), "h".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenizer::literal::StringPart;

    #[test]
    fn test_parse_identifier() {
        let input = vec![Token::Identifier("test".to_string())];
        let (rest, result) = parse_identifier().parse(&input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_parse_string() {
        let input = vec![Token::Literal(Literal::String(vec![StringPart::Literal(
            "test string".to_string(),
        )]))];
        let (rest, result) = parse_string().parse(&input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, ast::Literal::String("test string".to_string()));
    }

    #[test]
    fn test_parse_list() {
        let input = vec![
            Token::Delimiter(Delimiter::OpenBracket),
            Token::Literal(Literal::Integer(1)),
            Token::Delimiter(Delimiter::Comma),
            Token::Literal(Literal::Integer(2)),
            Token::Delimiter(Delimiter::CloseBracket),
        ];
        let (rest, result) = parse_list().parse(&input, 0).unwrap();
        assert_eq!(rest, 5);
        assert_eq!(
            result,
            ast::Literal::List(vec![ast::Literal::Integer(1), ast::Literal::Integer(2)])
        );
    }

    #[test]
    fn test_parse_map() {
        let input = vec![
            Token::Delimiter(Delimiter::OpenBrace),
            Token::Identifier("key".to_string()),
            Token::Delimiter(Delimiter::Colon),
            Token::Literal(Literal::Integer(42)),
            Token::Delimiter(Delimiter::CloseBrace),
        ];
        let (rest, result) = parse_map().parse(&input, 0).unwrap();
        assert_eq!(rest, 5);
        let mut expected_map = HashMap::new();
        expected_map.insert("key".to_string(), ast::Literal::Integer(42));
        assert_eq!(result, ast::Literal::Map(expected_map));
    }

    #[test]
    fn test_parse_retry() {
        let input = vec![
            Token::Identifier("Retry".to_string()),
            Token::Literal(Literal::Integer(3)),
            Token::Identifier("Fixed".to_string()),
            Token::Literal(Literal::Integer(5)),
        ];
        let (rest, result) = parse_retry().parse(&input, 0).unwrap();
        assert_eq!(rest, 4);
        assert_eq!(
            result,
            ast::Literal::Retry(ast::RetryConfig {
                max_attempts: 3,
                delay: ast::RetryDelay::Fixed(5),
            })
        );
    }

    #[test]
    fn test_parse_boolean() {
        // True case
        let input = vec![Token::Literal(Literal::Boolean(true))];
        let (rest, result) = parse_boolean().parse(&input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, ast::Literal::Boolean(true));

        // False case
        let input = vec![Token::Literal(Literal::Boolean(false))];
        let (rest, result) = parse_boolean().parse(&input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, ast::Literal::Boolean(false));
    }

    #[test]
    fn test_parse_null() {
        let input = vec![Token::Literal(Literal::Null)];
        let (rest, result) = parse_null().parse(&input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, ast::Literal::Null);
    }

    #[test]
    fn test_parse_numbers() {
        // Integer
        let input = vec![Token::Literal(Literal::Integer(42))];
        let (rest, result) = parse_i64().parse(&input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 42);

        // Float
        let input = vec![Token::Literal(Literal::Float(3.14))];
        let (rest, result) = parse_f64().parse(&input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 3.14);

        // u64
        let input = vec![Token::Literal(Literal::Integer(42))];
        let (rest, result) = parse_u64().parse(&input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 42);

        // usize
        let input = vec![Token::Literal(Literal::Integer(42))];
        let (rest, result) = parse_usize().parse(&input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_parse_delimiters() {
        // Comma
        let input = vec![Token::Delimiter(Delimiter::Comma)];
        assert!(parse_comma().parse(&input, 0).is_ok());

        // Semicolon
        let input = vec![Token::Delimiter(Delimiter::Semicolon)];
        assert!(parse_semicolon().parse(&input, 0).is_ok());

        // Colon
        let input = vec![Token::Delimiter(Delimiter::Colon)];
        assert!(parse_colon().parse(&input, 0).is_ok());

        // Equal
        let input = vec![Token::Delimiter(Delimiter::Equal)];
        assert!(parse_equal().parse(&input, 0).is_ok());

        // Parentheses
        let input = vec![Token::Delimiter(Delimiter::OpenParen)];
        assert!(parse_open_paren().parse(&input, 0).is_ok());
        let input = vec![Token::Delimiter(Delimiter::CloseParen)];
        assert!(parse_close_paren().parse(&input, 0).is_ok());

        // Brackets
        let input = vec![Token::Delimiter(Delimiter::OpenBracket)];
        assert!(parse_open_bracket().parse(&input, 0).is_ok());
        let input = vec![Token::Delimiter(Delimiter::CloseBracket)];
        assert!(parse_close_bracket().parse(&input, 0).is_ok());

        // Braces
        let input = vec![Token::Delimiter(Delimiter::OpenBrace)];
        assert!(parse_open_brace().parse(&input, 0).is_ok());
        let input = vec![Token::Delimiter(Delimiter::CloseBrace)];
        assert!(parse_close_brace().parse(&input, 0).is_ok());
    }

    #[test]
    fn test_parse_string_literal() {
        // 単純な文字列リテラル
        let input = &[Token::Literal(Literal::String(vec![StringPart::Literal(
            "test string".to_string(),
        )]))];

        let (pos, result) = parse_string().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(result, ast::Literal::String("test string".to_string()));
    }

    #[test]
    fn test_parse_string_with_interpolation() {
        // 補間を含む文字列
        let input = &[Token::Literal(Literal::String(vec![
            StringPart::Literal("Hello ".to_string()),
            StringPart::Interpolation("name".to_string()),
            StringPart::Literal("!".to_string()),
        ]))];

        // 現時点では未サポート
        assert!(parse_string().parse(input, 0).is_err());
    }

    #[test]
    fn test_parse_string_with_newline() {
        // 改行を含む文字列
        let input = &[Token::Literal(Literal::String(vec![
            StringPart::Literal("line 1".to_string()),
            StringPart::NewLine,
            StringPart::Literal("line 2".to_string()),
        ]))];

        // 現時点では未サポート
        assert!(parse_string().parse(input, 0).is_err());
    }

    #[test]
    fn test_separated_list() {
        // シンプルな数値のカンマ区切りリスト
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Delimiter(Delimiter::Comma),
            Token::Literal(Literal::Integer(2)),
        ];

        let number_parser = choice(vec![
            Box::new(map(equal(Token::Literal(Literal::Integer(1))), |_| 1)),
            Box::new(map(equal(Token::Literal(Literal::Integer(2))), |_| 2)),
        ]);
        let comma_parser = as_unit(equal(Token::Delimiter(Delimiter::Comma)));

        let list_parser = separated_list(number_parser, comma_parser);

        let (pos, result) = list_parser.parse(input, 0).unwrap();
        assert_eq!(pos, 3);
        assert_eq!(result, vec![1, 2]);
    }

    #[test]
    fn test_separated_list_single() {
        // 単一要素のリスト
        let input = &[Token::Literal(Literal::Integer(1))];

        let number_parser = map(equal(Token::Literal(Literal::Integer(1))), |_| 1);
        let comma_parser = as_unit(equal(Token::Delimiter(Delimiter::Comma)));

        let list_parser = separated_list(number_parser, comma_parser);

        let (pos, result) = list_parser.parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(result, vec![1]);
    }

    #[test]
    fn test_parse_literal() {
        // 整数
        let input = &[Token::Literal(Literal::Integer(42))];
        let (pos, result) = parse_literal().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(result, ast::Literal::Integer(42));

        // 真偽値
        let input = &[Token::Literal(Literal::Boolean(true))];
        let (pos, result) = parse_literal().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(result, ast::Literal::Boolean(true));

        // null
        let input = &[Token::Literal(Literal::Null)];
        let (pos, result) = parse_literal().parse(input, 0).unwrap();
        assert_eq!(pos, 1);
        assert_eq!(result, ast::Literal::Null);
    }

    #[test]
    fn test_parse_duration() {
        // all pattern for duration
        // 1ms, 1s, 1min, 1h
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("ms".to_string()),
        ];
        let (rest, result) = super::parse_duration().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            super::ast::Literal::Duration(std::time::Duration::from_millis(1))
        );
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("s".to_string()),
        ];
        let (rest, result) = super::parse_duration().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            super::ast::Literal::Duration(std::time::Duration::from_secs(1))
        );
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("min".to_string()),
        ];
        let (rest, result) = super::parse_duration().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            super::ast::Literal::Duration(std::time::Duration::from_secs(60))
        );
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("h".to_string()),
        ];
        let (rest, result) = super::parse_duration().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            super::ast::Literal::Duration(std::time::Duration::from_secs(3600))
        );
    }

    #[test]
    fn test_parse_duration_millis() {
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("ms".to_string()),
        ];
        let (rest, result) = parse_duration_millis().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            ast::Literal::Duration(std::time::Duration::from_millis(1))
        );
    }

    #[test]
    fn test_parse_duration_sec() {
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("s".to_string()),
        ];
        let (rest, result) = parse_duration_sec().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            ast::Literal::Duration(std::time::Duration::from_secs(1))
        );
    }

    #[test]
    fn test_parse_duration_min() {
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("min".to_string()),
        ];
        let (rest, result) = parse_duration_min().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            ast::Literal::Duration(std::time::Duration::from_secs(60))
        );
    }

    #[test]
    fn test_parse_duration_hour() {
        let input = &[
            Token::Literal(Literal::Integer(1)),
            Token::Identifier("h".to_string()),
        ];
        let (rest, result) = parse_duration_hour().parse(input, 0).unwrap();
        assert_eq!(rest, 2);
        assert_eq!(
            result,
            ast::Literal::Duration(std::time::Duration::from_secs(3600))
        );
    }

    #[test]
    fn test_parse_ms() {
        let input = &[Token::Identifier("ms".to_string())];
        let (rest, result) = parse_ms().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, "ms".to_string());
    }
    #[test]
    fn test_parse_sec() {
        let input = &[Token::Identifier("s".to_string())];
        let (rest, result) = parse_sec().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, "s".to_string());
    }
    #[test]
    fn test_parse_min() {
        let input = &[Token::Identifier("min".to_string())];
        let (rest, result) = parse_min().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, "min".to_string());
    }
    #[test]
    fn test_parse_hour() {
        let input = &[Token::Identifier("h".to_string())];
        let (rest, result) = parse_hour().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, "h".to_string());
    }

    #[test]
    fn test_identifier() {
        let input = &[Token::Identifier("foo".to_string())];
        let (rest, result) = parse_identifier().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, "foo".to_string());
    }

    #[test]
    fn test_parse_f64() {
        let input = &[Token::Literal(Literal::Float(3.14))];
        let (rest, result) = parse_f64().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 3.14);
    }

    #[test]
    fn test_parse_i64() {
        let input = &[Token::Literal(Literal::Integer(42))];
        let (rest, result) = parse_i64().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_parse_u64() {
        let input = &[Token::Literal(Literal::Integer(42))];
        let (rest, result) = parse_u64().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_parse_usize() {
        let input = &[Token::Literal(Literal::Integer(42))];
        let (rest, result) = parse_usize().parse(input, 0).unwrap();
        assert_eq!(rest, 1);
        assert_eq!(result, 42);
    }
}
