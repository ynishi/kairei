use crate::ast::*;
use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{map, map_res, opt, recognize},
    multi::{fold_many0, many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};
use std::{collections::HashMap, time::Duration};

/// MicroAgentの定義をパースする。
/// Entry point of the parser.
pub fn parse_micro_agent(input: &str) -> IResult<&str, MicroAgentDef> {
    let (input, _) = ws(tag("micro"))(input)?;
    let (input, name) = ws(identifier)(input)?;
    let (input, _) = ws(char('{'))(input)?;

    let mut lifecycle = None;
    let mut state = None;
    let mut observe = None;
    let mut answer = None;
    let mut react = None;

    let (input, _) = many0(alt((
        map(parse_lifecycle, |l| lifecycle = Some(l)),
        map(parse_state, |s| state = Some(s)),
        map(parse_observe, |o| observe = Some(o)),
        map(parse_answer, |a| answer = Some(a)),
        map(parse_react, |r| react = Some(r)),
    )))(input)?;

    let (input, _) = ws(char('}'))(input)?;

    Ok((
        input,
        MicroAgentDef {
            name: name.to_string(),
            lifecycle,
            state,
            observe,
            answer,
            react,
        },
    ))
}

// Top level blocks
fn parse_lifecycle(input: &str) -> IResult<&str, LifecycleDef> {
    map(
        block(
            "lifecycle",
            permutation((opt(ws(parse_init_handler)), opt(ws(parse_destroy_handler)))),
        ),
        |(on_init, on_destroy)| LifecycleDef {
            on_init,
            on_destroy,
        },
    )(input)
}

fn parse_state(input: &str) -> IResult<&str, StateDef> {
    map(
        block("state", separated_list0(ws(char(',')), parse_state_var)),
        |vars| {
            let mut variables = HashMap::new();
            for var in vars {
                variables.insert(var.name.clone(), var);
            }
            StateDef { variables }
        },
    )(input)
}

fn parse_observe(input: &str) -> IResult<&str, ObserveDef> {
    map(block("observe", many0(parse_event_handler)), |handlers| {
        ObserveDef { handlers }
    })(input)
}

fn parse_answer(input: &str) -> IResult<&str, AnswerDef> {
    map(block("answer", many0(parse_request_handler)), |handlers| {
        AnswerDef { handlers }
    })(input)
}

fn parse_react(input: &str) -> IResult<&str, ReactDef> {
    map(block("react", many0(parse_event_handler)), |handlers| {
        ReactDef { handlers }
    })(input)
}

// Block contents
fn parse_init_handler(input: &str) -> IResult<&str, Block> {
    preceded(tag("onInit"), parse_block)(input)
}

fn parse_destroy_handler(input: &str) -> IResult<&str, Block> {
    preceded(ws(tag("onDestroy")), parse_block)(input)
}

fn parse_event_handler(input: &str) -> IResult<&str, EventHandler> {
    map(
        tuple((
            preceded(ws(tag("on")), parse_event_type),
            opt(delimited(
                ws(char('(')),
                separated_list0(ws(char(',')), parse_parameter),
                ws(char(')')),
            )),
            parse_block,
        )),
        |(event_type, parameters, block)| EventHandler {
            event_type,
            parameters: parameters.unwrap_or_default(),
            block,
        },
    )(input)
}

fn parse_request_handler(input: &str) -> IResult<&str, RequestHandler> {
    map(
        tuple((
            preceded(ws(tag("on request")), parse_request_type),
            delimited(
                ws(char('(')),
                separated_list0(ws(char(',')), parse_parameter),
                ws(char(')')),
            ),
            opt(preceded(ws(tag("->")), ws(parse_type_info))),
            opt(ws(parse_constraints)),
            parse_block,
        )),
        |(request_type, parameters, return_type, constraints, block)| RequestHandler {
            request_type,
            parameters,
            return_type: return_type.unwrap_or_else(|| TypeInfo::Simple("Unit".to_string())),
            constraints,
            block,
        },
    )(input)
}

fn parse_state_var(input: &str) -> IResult<&str, StateVarDef> {
    map(
        tuple((
            ws(identifier),
            ws(char(':')),
            parse_type_info,
            opt(preceded(ws(char('=')), parse_expression)),
        )),
        |(name, _, type_info, initial_value)| StateVarDef {
            name: name.to_string(),
            type_info,
            initial_value,
        },
    )(input)
}

fn parse_parameter(input: &str) -> IResult<&str, Parameter> {
    map(
        tuple((ws(identifier), ws(char(':')), parse_type_info)),
        |(name, _, type_info)| Parameter {
            name: name.to_string(),
            type_info,
        },
    )(input)
}

fn parse_constraints(input: &str) -> IResult<&str, Constraints> {
    map(
        block("with constraints", many0(parse_constraint_item)),
        |items| {
            let mut constraints = Constraints {
                strictness: None,
                stability: None,
                latency: None,
            };

            for (key, value) in items {
                match key.as_str() {
                    "strictness" => constraints.strictness = Some(value),
                    "stability" => constraints.stability = Some(value),
                    "latency" => constraints.latency = Some(value as u32),
                    _ => {} // 未知の制約は無視
                }
            }

            constraints
        },
    )(input)
}

// Type Definitions
fn parse_event_type(input: &str) -> IResult<&str, EventType> {
    alt((
        map(tag("Tick"), |_| EventType::Tick),
        map(
            tuple((
                ws(tag("StateUpdated")),
                delimited(
                    ws(char('{')),
                    permutation((
                        terminated(
                            preceded(ws(tag("agent:")), parse_string),
                            opt(ws(char(','))),
                        ),
                        terminated(
                            preceded(ws(tag("state:")), parse_string),
                            opt(ws(char(','))),
                        ),
                    )),
                    ws(char('}')),
                ),
            )),
            |(_, (agent, state))| EventType::StateUpdated {
                agent_name: agent,
                state_name: state,
            },
        ),
        map(
            tuple((
                ws(tag("Message")),
                delimited(
                    ws(char('{')),
                    preceded(ws(tag("content:")), parse_string),
                    ws(char('}')),
                ),
            )),
            |(_, content)| EventType::Message {
                content_type: content,
            },
        ),
        map(identifier, |id| EventType::Custom(id.to_string())),
    ))(input)
}

fn parse_request_type(input: &str) -> IResult<&str, RequestType> {
    alt((
        map(
            tuple((
                tag("Query"),
                delimited(char('{'), preceded(tag("type:"), ws(identifier)), char('}')),
            )),
            |(_, query_type)| RequestType::Query {
                query_type: query_type.to_string(),
            },
        ),
        map(
            tuple((
                tag("Action"),
                delimited(char('{'), preceded(tag("type:"), ws(identifier)), char('}')),
            )),
            |(_, action_type)| RequestType::Action {
                action_type: action_type.to_string(),
            },
        ),
        map(identifier, |id| RequestType::Custom(id.to_string())),
    ))(input)
}

fn parse_request_options(input: &str) -> IResult<&str, RequestOptions> {
    map(
        tuple((
            ws(char('.')),
            ws(tag("with")),
            delimited(
                ws(char('{')),
                tuple((
                    opt(terminated(
                        preceded(
                            ws(pair(tag("timeout:"), multispace0)),
                            parse_duration, // Durationを返す
                        ),
                        opt(ws(char(','))),
                    )),
                    opt(terminated(
                        preceded(
                            ws(pair(tag("retry:"), multispace0)),
                            parse_u32, // u32を返す
                        ),
                        opt(ws(char(','))),
                    )),
                )),
                ws(char('}')),
            ),
        )),
        |(_, _, (timeout, retry))| RequestOptions { timeout, retry },
    )(input)
}

fn parse_constraint_item(input: &str) -> IResult<&str, (String, f64)> {
    terminated(
        tuple((
            map(terminated(identifier, ws(char(':'))), |s: &str| {
                s.to_string()
            }),
            parse_float,
        )),
        opt(ws(char(','))),
    )(input)
}

/// Block and Statement
fn parse_block(input: &str) -> IResult<&str, Block> {
    map(
        delimited(ws(char('{')), many0(parse_statement), ws(char('}'))),
        |statements| Block { statements },
    )(input)
}

fn parse_statement(input: &str) -> IResult<&str, Statement> {
    alt((
        parse_assignment,
        parse_emit_statement,
        parse_request_statement,
        parse_if_statement,
        parse_return_statement,
    ))(input)
}

fn parse_assignment(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((ws(parse_expression), char('='), parse_expression)),
        |(value, _, target)| Statement::Assignment { target, value },
    )(input)
}

fn parse_emit_statement(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            ws(tag("emit")),
            parse_event_type,
            opt(preceded(ws(tag("to")), identifier)),
            opt(delimited(
                ws(char('(')),
                separated_list0(ws(char(',')), parse_expression),
                ws(char(')')),
            )),
        )),
        |(_, event_type, target, parameters)| Statement::Emit {
            event_type,
            parameters: parameters.unwrap_or_default(),
            target: target.map(String::from),
        },
    )(input)
}

fn parse_request_statement(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            ws(tag("request")),
            parse_request_type,
            opt(parse_request_options),
            delimited(
                ws(char('(')),
                separated_list0(ws(char(',')), parse_expression),
                ws(char(')')),
            ),
        )),
        |(_, request_type, options, parameters)| Statement::Request {
            agent: "".to_string(), // TODO: Parse agent
            request_type,
            parameters,
            options,
        },
    )(input)
}

fn parse_if_statement(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            ws(tag("if")),
            delimited(ws(char('(')), parse_expression, ws(char(')'))),
            parse_block,
            opt(preceded(ws(tag("else")), parse_block)),
        )),
        |(_, condition, then_block, else_block)| Statement::If {
            condition,
            then_block,
            else_block,
        },
    )(input)
}

fn parse_return_statement(input: &str) -> IResult<&str, Statement> {
    map(
        preceded(ws(tag("return")), parse_expression),
        Statement::Return,
    )(input)
}

// Expressions
fn parse_expression(input: &str) -> IResult<&str, Expression> {
    parse_await(input)
}

fn parse_await(input: &str) -> IResult<&str, Expression> {
    alt((
        // await式
        map(
            tuple((
                ws(tag("await")),
                parse_logical_or, // または適切な優先順位の式パーサー
            )),
            |(_, expr)| Expression::Await(Box::new(expr)),
        ),
        // await以外の式
        parse_logical_or,
    ))(input)
}

// 論理OR (||)
fn parse_logical_or(input: &str) -> IResult<&str, Expression> {
    let (input, first) = parse_logical_and(input)?;
    let (input, rest) = many0(preceded(ws(tag("||")), parse_logical_and))(input)?;

    let result = rest
        .into_iter()
        .fold(first, |left, right| Expression::BinaryOp {
            op: BinaryOperator::Or,
            left: Box::new(left),
            right: Box::new(right),
        });

    Ok((input, result))
}

// 論理AND (&&)
fn parse_logical_and(input: &str) -> IResult<&str, Expression> {
    let (input, first) = parse_comparison(input)?;
    let (input, rest) = many0(preceded(ws(tag("&&")), parse_comparison))(input)?;

    let result = rest
        .into_iter()
        .fold(first, |left, right| Expression::BinaryOp {
            op: BinaryOperator::And,
            left: Box::new(left),
            right: Box::new(right),
        });

    Ok((input, result))
}

// 比較演算子 (==, !=, <, >, <=, >=)
fn parse_comparison(input: &str) -> IResult<&str, Expression> {
    let (input, first) = parse_additive(input)?;
    let (input, rest) = opt(tuple((
        ws(alt((
            tag("=="),
            tag("!="),
            tag("<="),
            tag(">="),
            tag("<"),
            tag(">"),
        ))),
        parse_additive,
    )))(input)?;

    match rest {
        Some((op, right)) => {
            let op = match op {
                "==" => BinaryOperator::Equal,
                "!=" => BinaryOperator::NotEqual,
                "<" => BinaryOperator::LessThan,
                ">" => BinaryOperator::GreaterThan,
                "<=" => BinaryOperator::LessThanEqual,
                ">=" => BinaryOperator::GreaterThanEqual,
                _ => unreachable!(),
            };
            Ok((
                input,
                Expression::BinaryOp {
                    op,
                    left: Box::new(first),
                    right: Box::new(right),
                },
            ))
        }
        None => Ok((input, first)),
    }
}

// 加減算 (+, -)
fn parse_additive(input: &str) -> IResult<&str, Expression> {
    let (input, first) = parse_multiplicative(input)?;
    let first = first.clone(); // 先にクローンを作成
    fold_many0::<_, _, _, _, _, _, Expression>(
        tuple((ws(alt((tag("+"), tag("-")))), parse_multiplicative)),
        move || first.clone(),
        |left, (op, right)| Expression::BinaryOp {
            op: match op {
                "+" => BinaryOperator::Add,
                "-" => BinaryOperator::Subtract,
                _ => unreachable!(),
            },
            left: Box::new(left),
            right: Box::new(right),
        },
    )(input)
}

// 乗除算 (*, /)
fn parse_multiplicative(input: &str) -> IResult<&str, Expression> {
    let (input, first) = parse_primary(input)?;
    let first = first.clone(); // 先にクローンを作成
    fold_many0::<_, _, _, _, _, _, Expression>(
        tuple((ws(alt((tag("*"), tag("/")))), parse_primary)),
        move || first.clone(),
        |left, (op, right)| Expression::BinaryOp {
            op: match op {
                "*" => BinaryOperator::Multiply,
                "/" => BinaryOperator::Divide,
                _ => unreachable!(),
            },
            left: Box::new(left),
            right: Box::new(right),
        },
    )(input)
}

// 基本式
fn parse_primary(input: &str) -> IResult<&str, Expression> {
    ws(alt((
        // リテラル
        map(parse_literal, Expression::Literal),
        // 関数呼び出し
        map(
            tuple((
                identifier,
                delimited(
                    char('('),
                    separated_list0(ws(char(',')), parse_expression),
                    char(')'),
                ),
            )),
            |(func, args)| Expression::FunctionCall {
                function: func.to_string(),
                arguments: args,
            },
        ),
        // 状態アクセス
        map(separated_list1(char('.'), identifier), |parts| {
            if parts.len() == 1 {
                Expression::Variable(parts[0].to_string())
            } else {
                Expression::StateAccess(StateAccessPath(
                    parts.into_iter().map(String::from).collect(),
                ))
            }
        }),
        // 括弧で囲まれた式
        delimited(ws(char('(')), parse_expression, ws(char(')'))),
        // 変数（最後に配置して他のパターンを先に試す）
        map(identifier, |id| Expression::Variable(id.to_string())),
    )))(input)
}

/// Basic Elements
fn parse_type_info(input: &str) -> IResult<&str, TypeInfo> {
    alt((
        // Result型
        map(
            tuple((
                ws(tag("Result")),
                ws(char('<')),
                parse_type_info,
                ws(char(',')),
                parse_type_info,
                ws(char('>')),
            )),
            |(_, _, ok_type, _, err_type, _)| TypeInfo::Result {
                ok_type: Box::new(ok_type),
                err_type: Box::new(err_type),
            },
        ),
        // Option型
        map(
            tuple((
                ws(tag("Option")),
                ws(char('<')),
                parse_type_info,
                ws(char('>')),
            )),
            |(_, _, inner_type, _)| TypeInfo::Option(Box::new(inner_type)),
        ),
        // 配列型
        map(
            tuple((
                ws(tag("Array")),
                ws(char('<')),
                parse_type_info,
                ws(char('>')),
            )),
            |(_, _, inner_type, _)| TypeInfo::Array(Box::new(inner_type)),
        ),
        // カスタム型（制約付き）
        map(
            tuple((
                ws(identifier),
                opt(delimited(
                    ws(char('{')),
                    separated_list0(
                        ws(char(',')),
                        tuple((ws(identifier), ws(char(':')), parse_expression)),
                    ),
                    ws(char('}')),
                )),
            )),
            |(name, constraints)| {
                let mut constraint_map = HashMap::new();
                if let Some(constraints) = constraints {
                    for (key, _, value) in constraints {
                        constraint_map.insert(key.to_string(), value);
                    }
                }
                TypeInfo::Custom {
                    name: name.to_string(),
                    constraints: constraint_map,
                }
            },
        ),
        // 基本型
        map(ws(identifier), |type_name: &str| {
            TypeInfo::Simple(type_name.to_string())
        }),
    ))(input)
}

fn parse_literal(input: &str) -> IResult<&str, Literal> {
    alt((
        // 整数
        map(parse_integer, Literal::Integer),
        // 文字列
        map(
            delimited(char('"'), take_while(|c| c != '"'), char('"')),
            |s: &str| Literal::String(s.to_string()),
        ),
        // 真偽値
        map(tag("true"), |_| Literal::Boolean(true)),
        map(tag("false"), |_| Literal::Boolean(false)),
        // null
        map(tag("null"), |_| Literal::Null),
    ))(input)
}

fn parse_string(input: &str) -> IResult<&str, String> {
    map(
        delimited(char('"'), take_while(|c| c != '"'), char('"')),
        |s: &str| s.to_string(),
    )(input)
}

fn parse_duration(input: &str) -> IResult<&str, Duration> {
    let (input, value) = parse_u64(input)?;
    let (input, unit) = opt(alt((tag("ms"), tag("s"), tag("m"), tag("h"))))(input)?;

    let duration = match unit {
        Some("ms") => Duration::from_millis(value),
        Some("s") => Duration::from_secs(value),
        Some("m") => Duration::from_secs(value * 60),
        Some("h") => Duration::from_secs(value * 3600),
        _ => Duration::from_millis(value), // デフォルトはミリ秒
    };

    Ok((input, duration))
}

fn identifier(input: &str) -> IResult<&str, &str> {
    let id_chars = |c: char| c.is_alphanumeric() || c == '_';
    let start_chars = |c: char| c.is_alphabetic() || c == '_';

    take_while1(start_chars)(input).and_then(|(rest, first)| {
        let (rest, others) = take_while(id_chars)(rest)?;
        Ok((rest, &input[..first.len() + others.len()]))
    })
}

fn parse_float(input: &str) -> IResult<&str, f64> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            take_while1(|c: char| c.is_ascii_digit()),
            opt(tuple((
                char('.'),
                take_while1(|c: char| c.is_ascii_digit()),
            ))),
        ))),
        |s: &str| s.parse::<f64>(),
    )(input)
}

fn parse_integer(input: &str) -> IResult<&str, i64> {
    map_res(digit1, |s: &str| s.parse::<i64>())(input)
}

fn parse_u32(input: &str) -> IResult<&str, u32> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), |s: &str| {
        s.parse::<u32>()
    })(input)
}

fn parse_u64(input: &str) -> IResult<&str, u64> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), |s: &str| {
        s.parse::<u64>()
    })(input)
}

/// キーワードブロックのパーサー
fn block<'a, F, O>(keyword: &'static str, inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    preceded(
        ws(tag(keyword)),
        delimited(ws(char('{')), inner, ws(char('}'))),
    )
}

/// 空白文字とコメントのスキップ
fn ws<'a, F, O>(inner: F) -> impl FnMut(&'a str) -> IResult<&'a str, O>
where
    F: FnMut(&'a str) -> IResult<&'a str, O>,
{
    delimited(multispace0, inner, multispace0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_micro_agent() {
        let input = r#"
            micro TestAgent {
                lifecycle {
                    onInit {
                        counter = 0
                    }
                    onDestroy {
                        emit Shutdown to manager
                    }
                }
                state {
                    counter: Int = 0,
                    name: String = "test",
                    active: Bool = true
                }
                observe {
                    on Tick {
                        counter = counter + 1
                    }
                    on StateUpdated { agent: "other", state: "value" } {
                        name = "updated"
                    }
                }
                answer {
                    on request GetCount() -> Result<Int, Error> {
                        return Ok(counter)
                    }

                    on request SetName(newName: String) -> Result<Bool, Error>
                    with constraints { strictness: 0.9, stability: 0.95 }
                    {
                        name = newName
                        return Ok(true)
                    }
                }
                react {
                    on Message { content: "reset" } {
                        counter = 0
                        emit StateUpdated { agent: "self", state: "counter" } to manager
                    }
                }
            }
       "#;
        let result = parse_micro_agent(input);
        assert!(result.is_ok());
        let agent = result.unwrap().1;
        assert_eq!(agent.name, "TestAgent");
        assert!(agent.lifecycle.is_some());
        assert!(agent.state.is_some());
        assert!(agent.observe.is_some());
        assert!(agent.answer.is_some());
        assert!(agent.react.is_some());
    }

    #[test]
    fn test_parse_identifier() {
        assert_eq!(identifier("abc123"), Ok(("", "abc123")));
        assert_eq!(identifier("_abc"), Ok(("", "_abc")));
        assert!(identifier("123abc").is_err());
    }

    #[test]
    fn test_parse_event_types() {
        let cases = [
            ("Tick", EventType::Tick),
            (
                "Message{content:\"update\"}",
                EventType::Message {
                    content_type: "update".to_string(),
                },
            ),
            (
                "StateUpdated{agent:\"counter\", state:\"value\"}",
                EventType::StateUpdated {
                    agent_name: "counter".to_string(),
                    state_name: "value".to_string(),
                },
            ),
            (
                "StateUpdated{state:\"value\", agent:\"counter\"}",
                EventType::StateUpdated {
                    agent_name: "counter".to_string(),
                    state_name: "value".to_string(),
                },
            ),
            ("CustomEvent", EventType::Custom("CustomEvent".to_string())),
        ];

        for (input, expected) in cases {
            let result = parse_event_type(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            let (_, event_type) = result.unwrap();
            assert_eq!(expected, event_type);
        }
    }

    #[test]
    fn test_parse_assignment() {
        let input = "count = count + 1";
        let result = parse_assignment(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_emit() {
        let cases = [
            "emit Tick",
            "emit Tick()",
            "emit StateUpdated{agent:\"counter\", state:\"value\"} to manager",
            "emit Message{content:\"update\"}",
            "emit CustomEvent to handler(param1, param2)",
        ];

        for input in cases {
            let result = parse_emit_statement(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
        }
    }

    #[test]
    fn test_parse_block() {
        let input = r#"{
            count = count + 1
            emit Updated to manager
            if (count > 10) {
                return count
            }
        }"#;
        let result = parse_block(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_state_var() {
        assert!(parse_state_var("count: Int = 0").is_ok());
        assert!(parse_state_var("name: String = \"test\"").is_ok());
        assert!(parse_state_var("flag: Bool").is_ok());
    }

    #[test]
    fn test_parse_type_info() {
        assert!(parse_type_info("Int").is_ok());
        assert!(parse_type_info("Result<Int, Error>").is_ok());
        assert!(parse_type_info("Option<String>").is_ok());
        assert!(parse_type_info("Array<Int>").is_ok());
        assert!(parse_type_info("UserName{minLength: 3, maxLength: 20}").is_ok());
    }

    #[test]
    fn test_parse_constraints() {
        let input = "with constraints { strictness: 0.9, stability: 0.95, latency: 1000 }";
        let result = parse_constraints(input);
        assert!(result.is_ok());
        let (_, constraints) = result.unwrap();
        assert_eq!(constraints.strictness, Some(0.9));
        assert_eq!(constraints.stability, Some(0.95));
        assert_eq!(constraints.latency, Some(1000));
    }

    #[test]
    fn test_parse_constraint_item() {
        let result = parse_constraint_item("strictness: 0.9,");
        assert!(result.is_ok());
        let (_, (key, value)) = result.unwrap();
        assert_eq!(key, "strictness");
        assert!((value - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_parse_request_options() {
        let inputs = [
            ".with { }",
            ".with { timeout: 2s }",
            ".with { retry: 5 }",
            ".with { timeout: 1000ms, retry: 3 }",
        ];

        for input in inputs {
            let result = parse_request_options(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
        }
    }

    #[test]
    fn test_basic_expressions() {
        let cases = [
            ("42", "Literal(Integer(42))"),
            ("\"hello\"", "Literal(String(\"hello\"))"),
            ("true", "Literal(Boolean(true))"),
            ("false", "Literal(Boolean(false))"),
            ("null", "Literal(Null)"),
            ("variable", "Variable(\"variable\")"),
            (
                "state.value",
                "StateAccess(StateAccessPath([\"state\", \"value\"]))",
            ),
        ];

        for (input, expected) in cases {
            let result = parse_expression(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            assert_eq!(expected, format!("{:?}", result.unwrap().1));
        }
    }

    #[test]
    fn test_arithmetic_expressions() {
        let cases = [
            ("1 + 2", "(1 + 2)"),
            ("1 - 2", "(1 - 2)"),
            ("2 * 3", "(2 * 3)"),
            ("6 / 2", "(6 / 2)"),
            ("1 + 2 * 3", "(1 + (2 * 3))"),   // 演算子の優先順位
            ("(1 + 2) * 3", "((1 + 2) * 3)"), // 括弧による優先順位の変更
            ("1 + 2 + 3", "((1 + 2) + 3)"),   // 左結合
            ("1 * 2 + 3 * 4", "((1 * 2) + (3 * 4))"),
        ];

        for (input, expected) in cases {
            let result = parse_expression(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            let expr = result.unwrap().1;
            assert_eq!(format_expression(&expr), expected);
        }
    }

    #[test]
    fn test_logical_expressions() {
        let cases = [
            ("a && b", "(a && b)"),
            ("a || b", "(a || b)"),
            ("a && b || c", "((a && b) || c)"),
            ("a || b && c", "(a || (b && c))"), // &&の優先順位が高い
            ("(a || b) && c", "((a || b) && c)"),
        ];

        for (input, expected) in cases {
            let result = parse_expression(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            let expr = result.unwrap().1;
            assert_eq!(format_expression(&expr), expected);
        }
    }

    #[test]
    fn test_comparison_expressions() {
        let cases = [
            ("a == b", "(a == b)"),
            ("a != b", "(a != b)"),
            ("a < b", "(a < b)"),
            ("a <= b", "(a <= b)"),
            ("a > b", "(a > b)"),
            ("a >= b", "(a >= b)"),
            ("a == b && c != d", "((a == b) && (c != d))"),
        ];

        for (input, expected) in cases {
            let result = parse_expression(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            let expr = result.unwrap().1;
            assert_eq!(expected, format_expression(&expr));
        }
    }

    #[test]
    fn test_complex_expressions() {
        let cases = [
            ("a.b.c + func(x, y) * 2", "(a.b.c + (func(x, y) * 2))"),
            ("(a || b) && (c + d > e)", "((a || b) && ((c + d) > e))"),
            (
                "1 + 2 * 3 == 4 && a || b",
                "((((1 + (2 * 3)) == 4) && a) || b)",
            ),
            ("await someFunc(x + y)", "await(someFunc((x + y)))"),
        ];

        for (input, expected) in cases {
            let result = parse_expression(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
            let expr = result.unwrap().1;
            assert_eq!(expected, format_expression(&expr));
        }
    }

    // Helper function to format expressions for testing
    fn format_expression(expr: &Expression) -> String {
        match expr {
            Expression::Literal(lit) => match lit {
                Literal::Integer(n) => n.to_string(),
                Literal::String(s) => s.clone(),
                Literal::Boolean(b) => b.to_string(),
                Literal::Null => "null".to_string(),
                Literal::Float(f) => f.to_string(),
            },
            Expression::Variable(name) => name.clone(),
            Expression::StateAccess(path) => path.0.join("."),
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                format!(
                    "{}({})",
                    function,
                    arguments
                        .iter()
                        .map(|arg| format_expression(arg))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expression::BinaryOp { op, left, right } => {
                let op_str = match op {
                    BinaryOperator::Add => "+",
                    BinaryOperator::Subtract => "-",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Equal => "==",
                    BinaryOperator::NotEqual => "!=",
                    BinaryOperator::LessThan => "<",
                    BinaryOperator::GreaterThan => ">",
                    BinaryOperator::LessThanEqual => "<=",
                    BinaryOperator::GreaterThanEqual => ">=",
                    BinaryOperator::And => "&&",
                    BinaryOperator::Or => "||",
                };
                format!(
                    "({} {} {})",
                    format_expression(left),
                    op_str,
                    format_expression(right)
                )
            }
            Expression::Await(expr) => format!("await({})", format_expression(expr)),
        }
    }
}
