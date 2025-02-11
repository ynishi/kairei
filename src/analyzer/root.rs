use std::collections::HashMap;
use std::marker::PhantomData;
use thiserror::Error;

use crate::tokenizer::keyword::Keyword;
use crate::tokenizer::token::Token;
use crate::LifecycleDef;

use super::parser::{ParseResult, ParserError};

// lifecycle パーサー
fn parse_lifecycle_rust(input: &[Token], pos: usize) -> ParseResult<LifecycleDef> {
    let keyword_lifecycle = expect(
        Token::Keyword(Keyword::Lifecycle),
        ParserError::ExpectedKeyword(Keyword::Lifecycle),
    );
    let block_parser = block(
        keyword_lifecycle,
        ParserCombinator::map(
            ParserCombinator::sequence(
                opt(ws(parse_init_handler_rust)),
                opt(ws(parse_destroy_handler_rust)),
            ),
            |(on_init, on_destroy)| LifecycleDef {
                on_init,
                on_destroy,
            },
        ),
    );
    block_parser.parse(input, pos)
}

// state パーサー
fn parse_state_rust(input: &[Token], pos: usize) -> ParseResult<StateDef> {
    let keyword_state = expect(
        Token::Keyword(Keyword::State),
        ParserError::ExpectedKeyword(Keyword::State),
    );
    let block_parser = block(
        keyword_state,
        ParserCombinator::map(
            separated_list0(
                ws(expect(
                    Token::Delimiter(Delimiter::Comma),
                    ParserError::ExpectedDelimiter(Delimiter::Comma),
                )),
                parse_state_var_rust,
            ),
            |vars| {
                let mut variables = HashMap::new();
                for var in vars {
                    variables.insert(var.name.clone(), var);
                }
                StateDef { variables }
            },
        ),
    );
    block_parser.parse(input, pos)
}

// init ハンドラー パーサー
fn parse_init_handler_rust(input: &[Token], pos: usize) -> ParseResult<InitHandler> {
    let keyword_on = expect(
        Token::Keyword(Keyword::On),
        ParserError::ExpectedKeyword(Keyword::On),
    );
    let keyword_init = expect(
        Token::Keyword(Keyword::Init),
        ParserError::ExpectedKeyword(Keyword::Init),
    );
    let on_init_sequence = ParserCombinator::sequence(keyword_on, keyword_init);

    let block_parser = block(
        on_init_sequence,
        ParserCombinator::many(parse_statement_rust), // ステートメントのパース
    );

    ParserCombinator::map(block_parser, |body| InitHandler { body }).parse(input, pos)
}

// destroy ハンドラー パーサー
fn parse_destroy_handler_rust(input: &[Token], pos: usize) -> ParseResult<DestroyHandler> {
    let keyword_on = expect(
        Token::Keyword(Keyword::On),
        ParserError::ExpectedKeyword(Keyword::On),
    );
    let keyword_destroy = expect(
        Token::Keyword(Keyword::Destroy),
        ParserError::ExpectedKeyword(Keyword::Destroy),
    );
    let on_destroy_sequence = ParserCombinator::sequence(keyword_on, keyword_destroy);

    let block_parser = block(
        on_destroy_sequence,
        ParserCombinator::many(parse_statement_rust), // ステートメントのパース
    );

    ParserCombinator::map(block_parser, |body| DestroyHandler { body }).parse(input, pos)
}

// state 変数 パーサー
fn parse_state_var_rust(input: &[Token], pos: usize) -> ParseResult<StateVar> {
    // 識別子をパースする
    let identifier_parser = ParserCombinator::map(
        expect(
            Token::Identifier("".to_string()),
            ParserError::ExpectedIdentifier,
        ),
        |token| {
            if let Token::Identifier(name) = token {
                name
            } else {
                unreachable!()
            }
        },
    );

    // オプションの初期値をパースする
    let initial_value_parser = opt(ParserCombinator::map(
        ParserCombinator::sequence(
            expect(
                Token::Operator(Operator::Assignment),
                ParserError::ExpectedOperator,
            ), // = を期待
            expect(
                Token::Literal(Literal::StringLiteral("".to_string())),
                ParserError::InvalidStateVarInit,
            ), // 文字列リテラルを期待
        ),
        |(_, value)| {
            if let Token::Literal(Literal::StringLiteral(v)) = value {
                Some(v)
            } else {
                None // unreachable!()
            }
        },
    ));

    // 識別子とオプションの初期値を組み合わせて StateVar を作成する
    ParserCombinator::map(
        ParserCombinator::sequence(identifier_parser, initial_value_parser),
        |(name, initial_value)| StateVar {
            name,
            initial_value,
        },
    )
    .parse(input, pos)
}
