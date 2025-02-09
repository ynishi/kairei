use super::{super::{core::*, prelude::*}, expression::*, *};
use crate::{ast, tokenizer::token::Token};
use std::collections::HashMap;

pub fn parse_type_info() -> impl Parser<Token, ast::TypeInfo> {
    with_context(
        lazy(|| {
            choice(vec![
                Box::new(parse_result_type()),
                Box::new(parse_option_type()),
                Box::new(parse_array_type()),
                Box::new(parse_simple_type()),
            ])
        }),
        "type info",
    )
}

fn parse_field() -> impl Parser<Token, (String, ast::FieldInfo)> {
    with_context(
        tuple2(
            parse_identifier(),
            choice(vec![
                Box::new(parse_field_typed_with_default()),
                Box::new(parse_field_typed()),
                Box::new(parse_field_inferred()),
            ]),
        ),
        "field",
    )
}

pub fn parse_field_typed_with_default() -> impl Parser<Token, ast::FieldInfo> {
    with_context(
        map(
            tuple4(
                parse_colon(),
                parse_identifier(),
                parse_equal(),
                parse_expression(),
            ),
            |(_, type_info, _, value)|ast::FieldInfo {
                type_info: Some(ast::TypeInfo::Simple(type_info)),
                default_value: Some(value),
            },
        ),
        "typed field with default value",
    )
}

fn parse_field_typed() -> impl Parser<Token,ast::FieldInfo> {
    with_context(
        map(
            preceded(as_unit(parse_colon()), parse_type_reference()),
            |type_info|ast::FieldInfo {
                type_info: Some(type_info),
                default_value: None,
            },
        ),
        "typed field",
    )
}

fn parse_field_inferred() -> impl Parser<Token,ast::FieldInfo> {
    with_context(
        map(preceded(as_unit(parse_equal()), parse_expression()), |value| {
           ast::FieldInfo {
                type_info: None,
                default_value: Some(value),
            }
        }),
        "inferred field",
    )
}

// 型参照のみを許可（インライン定義は不可）
fn parse_type_reference() -> impl Parser<Token, ast::TypeInfo> {
    lazy(|| {
        choice(vec![
            Box::new(parse_simple_type()), // String, Intなど
            Box::new(parse_result_type()), // Result<T, E>
            Box::new(parse_option_type()), // Option<T>
            Box::new(parse_array_type()),  // Array<T>
                                           // カスタム型の参照はOK、定義は不可
        ])
    })
}

pub fn parse_custom_type() -> impl Parser<Token,ast::TypeInfo> {
    with_context(
        map(
            tuple2(
                parse_identifier(),
                delimited(
                    as_unit(parse_open_brace()),
                    separated_list(lazy(parse_field), as_unit(parse_comma())),
                    as_unit(parse_close_brace()),
                ),
            ),
            |(name, fields)| {
                let field_map = fields.into_iter().collect::<HashMap<_, _>>();
               ast::TypeInfo::Custom {
                    name,
                    fields: field_map,
                }
            },
        ),
        "custom type",
    )
}

pub fn parse_option_type() -> impl Parser<Token, ast::TypeInfo> {
    with_context(
        map(parse_generic_single_arg("Option"), |inner_type| {
            ast::TypeInfo::Option(inner_type)
        }),
        "Option type",
    )
}

fn parse_array_type() -> impl Parser<Token, ast::TypeInfo> {
    with_context(
        map(parse_generic_single_arg("Array"), |element_type| {
            ast::TypeInfo::Array(element_type)
        }),
        "Array type",
    )
}

fn parse_generic_single_arg(type_name: &'static str) -> impl Parser<Token, Box<ast::TypeInfo>> {
    map(
        tuple2(
            expected(parse_identifier(), type_name.to_string()),
            delimited(
                as_unit(parse_open_brace()),
                map(parse_type_info(), Box::new),
                as_unit(parse_close_brace()),
            ),
        ),
        |(_, inner)| inner,
    )
}

pub fn parse_result_type() -> impl Parser<Token, ast::TypeInfo> {
    with_context(
        map(
            tuple6(
                expected(parse_identifier(), "Result".to_string()),
                parse_open_brace(),
                parse_identifier(),
                parse_comma(),
                parse_identifier(),
                parse_close_brace(),
            ),
            |(_, _, ok, _, err, _)| ast::TypeInfo::Result {
                ok_type: Box::new(ast::TypeInfo::Simple(ok)),
                err_type: Box::new(ast::TypeInfo::Simple(err)),
            },
        ),
        "Result type",
    )
}

fn parse_simple_type() -> impl Parser<Token, ast::TypeInfo> {
    with_context(
        map(parse_identifier(), ast::TypeInfo::Simple),
        "simple type",
    )
}
