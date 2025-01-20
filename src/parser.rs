use crate::ast::*;
use nom::{
    branch::{alt, permutation},
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0},
    combinator::{map, map_res, not, opt, peek, recognize},
    multi::{fold_many0, many0, separated_list0, separated_list1},
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};
use std::{collections::HashMap, time::Duration};
use tracing::{debug, instrument, warn};

/// Entry point of the parser.
#[instrument(level = "debug", skip(input))]
pub fn parse_root(input: &str) -> IResult<&str, Root> {
    map(
        pair(opt(ws(parse_world)), many0(ws(parse_micro_agent))),
        |(world_def, micro_agent_defs)| Root::new(world_def, micro_agent_defs),
    )(input)
}

/// MicroAgentの定義をパースする。
#[instrument(level = "debug", skip(input))]
pub fn parse_micro_agent(input: &str) -> IResult<&str, MicroAgentDef> {
    let (input, _) = ws(tag("micro"))(input)?;
    let (input, name) = ws(identifier)(input)?;
    let context = PolicyParseContext {
        scope: PolicyScope::Agent(name.to_string()),
    };
    let (input, _) = ws(char('{'))(input)?;

    let mut policies = vec![];
    let mut lifecycle = None;
    let mut state = None;
    let mut observe = None;
    let mut answer = None;
    let mut react = None;

    let (input, _) = many0(alt((
        map(|input| parse_policy(input, &context), |p| policies.push(p)),
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
            policies,
            lifecycle,
            state,
            observe,
            answer,
            react,
        },
    ))
}

// Top level blocks
#[instrument(level = "debug", skip(input))]
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

#[instrument(level = "debug", skip(input))]
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

#[instrument(level = "debug", skip(input))]
fn parse_observe(input: &str) -> IResult<&str, ObserveDef> {
    map(block("observe", many0(parse_event_handler)), |handlers| {
        ObserveDef { handlers }
    })(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_answer(input: &str) -> IResult<&str, AnswerDef> {
    map(block("answer", many0(parse_request_handler)), |handlers| {
        AnswerDef { handlers }
    })(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_react(input: &str) -> IResult<&str, ReactDef> {
    map(block("react", many0(parse_event_handler)), |handlers| {
        ReactDef { handlers }
    })(input)
}

/// Worldの定義をパースする。
#[instrument(level = "debug", skip(input))]
pub fn parse_world(input: &str) -> IResult<&str, WorldDef> {
    let (input, _) = ws(tag("world"))(input)?;
    let (input, name) = ws(identifier)(input)?;

    let context = PolicyParseContext {
        scope: PolicyScope::World(name.to_string()),
    };

    let (input, _) = ws(char('{'))(input)?;

    let mut policies = vec![];
    let mut config = None;
    let mut events = None;
    let mut handlers = None;

    let (input, _) = many0(alt((
        map(|i| parse_policy(i, &context), |p| policies.push(p)),
        map(parse_config, |c| config = Some(c)),
        map(parse_events, |e| events = Some(e)),
        map(parse_handlers, |h| handlers = Some(h)),
    )))(input)?;

    let (input, _) = ws(char('}'))(input)?;

    Ok((
        input,
        WorldDef {
            name: name.to_string(),
            policies,
            config,
            events: events.unwrap_or_default(),
            handlers: handlers.unwrap_or_default(),
        },
    ))
}

/// configセクションのパース
#[instrument(level = "debug", skip(input))]
pub fn parse_config(input: &str) -> IResult<&str, ConfigDef> {
    let (input, _) = ws(tag("config"))(input)?;
    let (input, _) = ws(char('{'))(input)?;

    let mut tick_interval = None;
    let mut max_agents = None;
    let mut event_buffer_size = None;

    let (input, _) = many0(alt((
        // Duration形式（例: 100ms, 1s)のパース
        map(parse_duration_config("tick_interval"), |d| {
            tick_interval = Some(d)
        }),
        // 整数値のパース
        map(parse_int_config("max_agents"), |n| max_agents = Some(n)),
        map(parse_int_config("event_buffer_size"), |n| {
            event_buffer_size = Some(n)
        }),
    )))(input)?;

    let (input, _) = ws(char('}'))(input)?;

    let mut config_def = ConfigDef::default();
    if let Some(tick_interval) = tick_interval {
        config_def.tick_interval = tick_interval;
    }
    if let Some(max_agents) = max_agents {
        config_def.max_agents = max_agents;
    }
    if let Some(event_buffer_size) = event_buffer_size {
        config_def.event_buffer_size = event_buffer_size;
    }

    Ok((input, config_def))
}

/// eventsセクションのパース
#[instrument(level = "debug", skip(input))]
pub fn parse_events(input: &str) -> IResult<&str, EventsDef> {
    let (input, _) = ws(tag("events"))(input)?;
    let (input, _) = ws(char('{'))(input)?;

    let (input, events) = many0(parse_event_def)(input)?;
    let (input, _) = ws(char('}'))(input)?;

    Ok((input, EventsDef { events }))
}

/// 個別のイベント定義のパース
#[instrument(level = "debug", skip(input))]
fn parse_event_def(input: &str) -> IResult<&str, CustomEventDef> {
    let (input, name) = ws(identifier)(input)?;
    let (input, parameters) = opt(delimited(
        ws(char('(')),
        separated_list0(ws(char(',')), parse_parameter),
        ws(char(')')),
    ))(input)?;

    let var_name = CustomEventDef {
        name: name.to_string(),
        parameters: parameters.unwrap_or_default(),
    };
    Ok((input, var_name))
}

/// handlersセクションのパース
#[instrument(level = "debug", skip(input))]
pub fn parse_handlers(input: &str) -> IResult<&str, HandlersDef> {
    let (input, _) = ws(tag("handlers"))(input)?;
    let (input, _) = ws(char('{'))(input)?;

    let (input, handlers) = many0(parse_handler)(input)?;
    let (input, _) = ws(char('}'))(input)?;

    Ok((input, HandlersDef { handlers }))
}

/// 個別のハンドラ定義のパース
#[instrument(level = "debug", skip(input))]
fn parse_handler(input: &str) -> IResult<&str, HandlerDef> {
    map(
        tuple((
            preceded(ws(tag("on")), map(identifier, |id| id.to_string())),
            opt(delimited(
                ws(char('(')),
                separated_list0(ws(char(',')), parse_parameter),
                ws(char(')')),
            )),
            parse_block,
        )),
        |(event_name, parameters, block)| HandlerDef {
            event_name,
            parameters: parameters.unwrap_or_default(),
            block,
        },
    )(input)
}

#[instrument(level = "debug")]
fn parse_duration_config(key: &'static str) -> impl Fn(&str) -> IResult<&str, Duration> {
    move |input: &str| {
        let (input, _) = ws(tag(key))(input)?;
        let (input, _) = ws(char(':'))(input)?;
        let (input, value) = ws(parse_duration)(input)?;
        Ok((input, value))
    }
}

/// 正の整数値設定のパース (例: max_agents: 1000)
#[instrument(level = "debug")]
fn parse_int_config(key: &'static str) -> impl Fn(&str) -> IResult<&str, usize> {
    move |input: &str| {
        let (input, _) = ws(tag(key))(input)?;
        let (input, _) = ws(char(':'))(input)?;
        let (input, value) = ws(parse_usize)(input)?;
        Ok((input, value))
    }
}

// Block contents
#[instrument(level = "debug", skip(input))]
fn parse_init_handler(input: &str) -> IResult<&str, HandlerBlock> {
    preceded(tag("onInit"), parse_block)(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_destroy_handler(input: &str) -> IResult<&str, HandlerBlock> {
    preceded(ws(tag("onDestroy")), parse_block)(input)
}

#[instrument(level = "debug", skip(input))]
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

#[instrument(level = "debug", skip(input))]
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

#[instrument(level = "debug", skip(input))]
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

#[instrument(level = "debug", skip(input))]
fn parse_parameter(input: &str) -> IResult<&str, Parameter> {
    map(
        tuple((ws(identifier), ws(char(':')), parse_type_info)),
        |(name, _, type_info)| Parameter {
            name: name.to_string(),
            type_info,
        },
    )(input)
}

#[instrument(level = "debug", skip(input))]
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
#[instrument(level = "debug", skip(input))]
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

#[instrument(level = "debug", skip(input))]
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

#[instrument(level = "debug", skip(input))]
fn parse_request_attributes(input: &str) -> IResult<&str, RequestAttributes> {
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
        |(_, _, (timeout, retry))| RequestAttributes { timeout, retry },
    )(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_constraint_item(input: &str) -> IResult<&str, (String, f64)> {
    terminated(
        tuple((
            map(terminated(identifier, ws(char(':'))), |s: &str| {
                s.to_string()
            }),
            parse_f64,
        )),
        opt(ws(char(','))),
    )(input)
}

/// Block and Statement
#[instrument(level = "debug", skip(input))]
fn parse_block(input: &str) -> IResult<&str, HandlerBlock> {
    map(parse_statements, |statements| HandlerBlock { statements })(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_statements(input: &str) -> IResult<&str, Vec<Statement>> {
    map(
        delimited(ws(char('{')), many0(parse_statement), ws(char('}'))),
        |statements| statements,
    )(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_statement(input: &str) -> IResult<&str, Statement> {
    let (input, base_statement) = alt((
        parse_assignment,
        parse_emit_statement,
        parse_if_statement,
        parse_return_statement,
    ))(input)?;
    parse_optional_error_handler(input, base_statement)
}

#[instrument(level = "debug", skip(input))]
fn parse_await(input: &str) -> IResult<&str, Expression> {
    map(preceded(ws(tag("await")), parse_expression), |s| {
        Expression::Await(vec![s])
    })(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_await_block(input: &str) -> IResult<&str, Expression> {
    map(
        preceded(
            ws(tag("await")),
            delimited(ws(char('{')), many0(parse_expression), ws(char('}'))),
        ),
        Expression::Await,
    )(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_optional_error_handler(
    input: &str,
    base_statement: Statement,
) -> IResult<&str, Statement> {
    // onFail部分はオプショナル
    let (input, error_handler) = opt(tuple((
        ws(tag("onFail")),
        // エラーバインディングは括弧付きの識別子としてオプショナル
        opt(delimited(ws(tag("(")), identifier, ws(tag(")")))),
        // エラーハンドラブロック
        parse_block,
    )))(input)?;

    // エラーハンドラが存在する場合は WithErrorHandler を生成
    match error_handler {
        Some((_, error_binding, handler_block)) => Ok((
            input,
            Statement::WithError {
                statement: Box::new(base_statement),
                error_handler_block: ErrorHandlerBlock {
                    error_binding: error_binding.map(String::from),
                    error_handler_statements: handler_block.statements,
                },
            },
        )),
        None => Ok((input, base_statement)),
    }
}

#[instrument(level = "debug", skip(input))]
fn parse_assignment(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            // 左辺: 単一の式または括弧なしカンマ区切りの式のリスト
            alt((
                map(separated_list1(ws(char(',')), parse_expression), |exprs| {
                    exprs
                }),
                delimited(
                    char('('),
                    separated_list1(ws(char(',')), parse_expression),
                    char(')'),
                ),
            )),
            ws(char('=')),
            parse_expression,
        )),
        |(target, _, value)| Statement::Assignment { target, value },
    )(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_emit_statement(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            ws(tag("emit")),
            parse_event_type,
            opt(preceded(ws(tag("to")), identifier)),
            opt(delimited(
                ws(char('(')),
                separated_list0(ws(char(',')), parse_argument),
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

#[instrument(level = "debug", skip(input))]
fn parse_request_expression(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            ws(tag("request")),
            parse_request_type,
            opt(parse_request_attributes),
            preceded(ws(tag("to")), identifier),
            parse_arguments,
        )),
        |(_, request_type, options, target, parameters)| Expression::Request {
            agent: target.to_string(),
            request_type,
            parameters,
            options,
        },
    )(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_arguments(input: &str) -> IResult<&str, Vec<Argument>> {
    delimited(
        ws(char('(')),
        separated_list0(ws(char(',')), parse_argument),
        ws(char(')')),
    )(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_argument(input: &str) -> IResult<&str, Argument> {
    alt((
        // キーワード付き引数のパース
        map(
            tuple((identifier, ws(char(':')), parse_expression)),
            |(name, _, value)| Argument::Named {
                name: name.to_string(),
                value: value.clone(),
            },
        ),
        // キーワードなし引数のパース
        map(parse_expression, Argument::Positional),
    ))(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_if_statement(input: &str) -> IResult<&str, Statement> {
    map(
        tuple((
            ws(tag("if")),
            delimited(ws(char('(')), parse_expression, ws(char(')'))),
            parse_statements,
            opt(preceded(ws(tag("else")), parse_statements)),
        )),
        |(_, condition, then_block, else_block)| Statement::If {
            condition,
            then_block,
            else_block,
        },
    )(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_return_statement(input: &str) -> IResult<&str, Statement> {
    map(
        preceded(ws(tag("return")), parse_expression),
        Statement::Return,
    )(input)
}

// Expressions
#[instrument(level = "debug", skip(input))]
fn parse_expression(input: &str) -> IResult<&str, Expression> {
    parse_logical_or(input)
}

// 論理OR (||)
#[instrument(level = "debug", skip(input))]
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
#[instrument(level = "debug", skip(input))]
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
#[instrument(level = "debug", skip(input))]
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
#[instrument(level = "debug", skip(input))]
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
#[instrument(level = "debug", skip(input))]
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

const RESERVED_KEYWORDS: [&str; 13] = [
    "think",
    "emit",
    "request",
    "if",
    "return",
    "await",
    "onFail",
    "on",
    "onInit",
    "onDestroy",
    "with",
    "Ok",
    "Err",
];

fn parse_not_reserved(input: &str) -> IResult<&str, ()> {
    not(peek(alt((
        ws(tag(RESERVED_KEYWORDS[0])),
        ws(tag(RESERVED_KEYWORDS[1])),
        ws(tag(RESERVED_KEYWORDS[2])),
        ws(tag(RESERVED_KEYWORDS[3])),
        ws(tag(RESERVED_KEYWORDS[4])),
        ws(tag(RESERVED_KEYWORDS[5])),
        ws(tag(RESERVED_KEYWORDS[6])),
        ws(tag(RESERVED_KEYWORDS[7])),
        ws(tag(RESERVED_KEYWORDS[8])),
        ws(tag(RESERVED_KEYWORDS[9])),
        ws(tag(RESERVED_KEYWORDS[10])),
        ws(tag(RESERVED_KEYWORDS[11])),
        ws(tag(RESERVED_KEYWORDS[12])),
    ))))(input)
}

// 基本式
#[instrument(level = "debug", skip(input))]
fn parse_primary(input: &str) -> IResult<&str, Expression> {
    ws(alt((
        // リテラル
        map(parse_literal, Expression::Literal),
        // LLM呼び出し
        parse_think_expression,
        // Result式
        parse_result_expression,
        // Resuest呼び出し
        parse_request_expression,
        // await式
        parse_await_block,
        parse_await,
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
        // 括弧で囲まれた式
        delimited(ws(char('(')), parse_expression, ws(char(')'))),
        // 状態アクセス
        parse_state_access,
        // 変数（最後に配置して他のパターンを先に試す）
        parse_variable,
    )))(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_state_access(input: &str) -> IResult<&str, Expression> {
    preceded(
        parse_not_reserved,
        map(
            tuple((
                identifier,
                preceded(char('.'), separated_list1(char('.'), identifier)),
            )),
            |(head, tail)| {
                let mut parts = vec![head];
                parts.extend(tail);
                Expression::StateAccess(StateAccessPath(
                    parts.into_iter().map(String::from).collect(),
                ))
            },
        ),
    )(input)
}

fn parse_variable(input: &str) -> IResult<&str, Expression> {
    preceded(
        parse_not_reserved,
        map(identifier, |id| Expression::Variable(id.to_string())),
    )(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_think_expression(input: &str) -> IResult<&str, Expression> {
    map(
        tuple((
            ws(tag("think")),
            parse_arguments,
            opt(parse_think_attributes),
        )),
        |(_, args, with_block)| Expression::Think { args, with_block },
    )(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_think_attributes(input: &str) -> IResult<&str, ThinkAttributes> {
    map(
        tuple((
            ws(tag("with")),
            delimited(
                ws(char('{')),
                separated_list0(ws(char(',')), parse_think_attribute),
                ws(char('}')),
            ),
        )),
        |(_, settings)| collect_with_settings(settings),
    )(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_think_attribute(input: &str) -> IResult<&str, ThinkAttributeKV> {
    alt((
        // ネストされたオブジェクトをパースする
        map(
            tuple((
                ws(identifier),
                ws(char(':')),
                delimited(
                    ws(char('{')),
                    separated_list0(ws(char(',')), parse_think_attribute),
                    ws(char('}')),
                ),
            )),
            |(key, _, nested_attrs)| ThinkAttributeKV {
                key: key.to_string(),
                value: Literal::Map(
                    nested_attrs
                        .into_iter()
                        .map(|attr| (attr.key, attr.value))
                        .collect(),
                ),
            },
        ),
        // 通常のキー・バリューをパースする
        map(
            tuple((ws(identifier), ws(char(':')), parse_literal)),
            |(key, _, value)| ThinkAttributeKV {
                key: key.to_string(),
                value,
            },
        ),
    ))(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_result_expression(input: &str) -> IResult<&str, Expression> {
    alt((
        // Ok(expr)のパース
        map(
            tuple((
                ws(tag("Ok")),
                preceded(ws(tag("(")), terminated(parse_expression, ws(tag(")")))),
            )),
            |(_, expr)| Expression::Ok(Box::new(expr)),
        ),
        // Err(expr)のパース
        map(
            tuple((
                ws(tag("Err")),
                preceded(ws(tag("(")), terminated(parse_expression, ws(tag(")")))),
            )),
            |(_, expr)| Expression::Err(Box::new(expr)),
        ),
    ))(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_retry(input: &str) -> IResult<&str, RetryConfig> {
    let mut max_attempts = None;
    let mut delay = None;

    let (input, _) = ws(char('{'))(input)?;
    let (input, _) = separated_list0(
        ws(tag(",")),
        alt((
            map(
                preceded(tuple((ws(tag("max_attempts")), ws(char(':')))), parse_u64),
                |n| max_attempts = Some(n),
            ),
            map(
                preceded(tuple((ws(tag("delay")), ws(char(':')))), parse_retry_delay),
                |rd| delay = Some(rd),
            ),
        )),
    )(input)?;
    let (input, _) = ws(char('}'))(input)?;

    let max_attempts = max_attempts.ok_or_else(|| new_verify_error(input))?;
    let delay = delay.ok_or_else(|| new_verify_error(input))?;

    Ok((
        input,
        RetryConfig {
            max_attempts,
            delay,
        },
    ))
}

#[instrument(level = "debug", skip(input))]
fn new_verify_error(input: &str) -> nom::Err<nom::error::Error<&str>> {
    nom::Err::Error(nom::error::Error::new(input, nom::error::ErrorKind::Verify))
}

#[instrument(level = "debug", skip(input))]
fn parse_retry_delay(input: &str) -> IResult<&str, RetryDelay> {
    alt((
        map(preceded(ws(tag("fixed")), parse_u64), RetryDelay::Fixed),
        map(
            delimited(
                preceded(ws(tag("exponential")), ws(tag("{"))),
                tuple((
                    tuple((ws(tag("initial")), ws(tag(":")), parse_u64)),
                    preceded(
                        tuple((ws(tag(",")), ws(tag("max")), ws(tag(":")))),
                        parse_u64,
                    ),
                )),
                ws(tag("}")),
            ),
            |((_, _, initial), max)| RetryDelay::Exponential { initial, max },
        ),
    ))(input)
}

// パースコンテキストを構造体として定義（ASTの構築時のみ使用）
#[derive(Debug, Clone)]
struct PolicyParseContext {
    scope: PolicyScope,
}

#[instrument(level = "debug", skip(input, context))]
fn parse_policy<'a>(input: &'a str, context: &PolicyParseContext) -> IResult<&'a str, Policy> {
    debug!("parse_policy: input={:?}", input);
    let (input, _) = ws(tag("policy"))(input)?;

    // policy <text> または policy <name> <text>
    let (input, (name, text)) = alt((
        // Named policy: policy <name> <text>
        map(tuple((identifier, ws(parse_string))), |(name, text)| {
            (Some(name), text)
        }),
        // Anonymous policy: policy <text>
        map(parse_string, |text| (None, text)),
    ))(input)?;

    Ok((
        input,
        Policy {
            text,
            scope: context.scope.clone(),
            internal_id: match name {
                Some(name) => PolicyId::builtin(name),
                None => PolicyId::new(),
            },
        },
    ))
}

// ヘルパー関数
fn collect_with_settings(settings: Vec<ThinkAttributeKV>) -> ThinkAttributes {
    let mut block = ThinkAttributes {
        provider: None,
        model: None,
        temperature: None,
        max_tokens: None,
        retry: None,
        policies: vec![],
        prompt_generator_type: None,
        plugins: HashMap::new(),
    };

    for setting in settings {
        match (setting.key.as_str(), setting.value) {
            ("provider", Literal::String(s)) => block.provider = Some(s),
            ("model", Literal::String(s)) => block.model = Some(s),
            ("temperature", Literal::Float(f)) => block.temperature = Some(f),
            ("retry", Literal::Retry(r)) => block.retry = Some(r),
            ("max_tokens", Literal::Integer(n)) => block.max_tokens = Some(n as u32),
            ("policies", Literal::List(policies)) => {
                for policy in policies {
                    if let Literal::String(text) = policy {
                        let policy = Policy {
                            text,
                            scope: PolicyScope::Think,
                            internal_id: PolicyId::new(),
                        };
                        block.policies.push(policy);
                    }
                }
            }
            // プラグイン設定の処理
            (plugin_name, Literal::Map(configs)) => {
                let mut plugin_config = HashMap::new();
                for (key, value) in configs {
                    plugin_config.insert(key, value);
                }
                block.plugins.insert(plugin_name.to_string(), plugin_config);
            }
            (key, value) => {
                warn!("Unknown think attribute: {}={:?}", key, value);
            }
        }
    }

    block
}

#[derive(Debug)]
struct ThinkAttributeKV {
    key: String,
    value: Literal,
}

/// Basic Elements
#[instrument(level = "debug", skip(input))]
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

#[instrument(level = "debug", skip(input))]
fn parse_literal(input: &str) -> IResult<&str, Literal> {
    alt((
        // number(ここは自動で決定するため`.`なしは整数とする)
        map(parse_f64_strict, Literal::Float),
        map(parse_i64, Literal::Integer),
        // 文字列
        map(
            delimited(char('"'), take_while(|c| c != '"'), char('"')),
            |s: &str| Literal::String(s.to_string()),
        ),
        // 真偽値
        map(tag("true"), |_| Literal::Boolean(true)),
        map(tag("false"), |_| Literal::Boolean(false)),
        // List型(要素はLiteralのみ)
        map(
            delimited(
                char('['),
                separated_list0(ws(char(',')), parse_literal),
                char(']'),
            ),
            Literal::List,
        ),
        // null
        map(tag("null"), |_| Literal::Null),
        map(parse_retry, Literal::Retry),
    ))(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_string(input: &str) -> IResult<&str, String> {
    map(
        delimited(char('"'), take_while(|c| c != '"'), char('"')),
        |s: &str| s.to_string(),
    )(input)
}

#[instrument(level = "debug", skip(input))]
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

#[instrument(level = "debug", skip(input))]
fn identifier(input: &str) -> IResult<&str, &str> {
    let id_chars = |c: char| c.is_alphanumeric() || c == '_';
    let start_chars = |c: char| c.is_alphabetic() || c == '_';

    take_while1(start_chars)(input).and_then(|(rest, first)| {
        let (rest, others) = take_while(id_chars)(rest)?;
        Ok((rest, &input[..first.len() + others.len()]))
    })
}

#[instrument(level = "debug", skip(input))]
fn parse_f64(input: &str) -> IResult<&str, f64> {
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

#[instrument(level = "debug", skip(input))]
fn parse_f64_strict(input: &str) -> IResult<&str, f64> {
    map_res(
        recognize(tuple((
            opt(char('-')),
            take_while1(|c: char| c.is_ascii_digit()),
            tuple((char('.'), take_while1(|c: char| c.is_ascii_digit()))),
        ))),
        |s: &str| s.parse::<f64>(),
    )(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_usize(input: &str) -> IResult<&str, usize> {
    map_res(digit1, |s: &str| s.parse::<usize>())(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_i64(input: &str) -> IResult<&str, i64> {
    map_res(digit1, |s: &str| s.parse::<i64>())(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_u32(input: &str) -> IResult<&str, u32> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), |s: &str| {
        s.parse::<u32>()
    })(input)
}

#[instrument(level = "debug", skip(input))]
fn parse_u64(input: &str) -> IResult<&str, u64> {
    map_res(take_while1(|c: char| c.is_ascii_digit()), |s: &str| {
        s.parse::<u64>()
    })(input)
}

/// キーワードブロックのパーサー
#[instrument(level = "debug", skip(inner))]
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
    fn test_parse_root_with_world_and_agents() {
        let input = r#"
               world TestWorld {
                   config { }
               }

               micro Agent1 {
                   state { }
               }

               micro Agent2 {
                   state { }
               }
           "#;

        let result = parse_root(input);
        assert!(result.is_ok());

        let (_, root) = result.unwrap();
        assert!(root.world_def.is_some());
        assert_eq!(root.micro_agent_defs.len(), 2);
        assert_eq!(root.micro_agent_defs[0].name, "Agent1");
        assert_eq!(root.micro_agent_defs[1].name, "Agent2");
    }

    #[test]
    fn test_parse_root_with_only_agents() {
        let input = r#"
               micro Agent1 {
                   state { }
               }

               micro Agent2 {
                   state { }
               }
           "#;

        let result = parse_root(input);
        assert!(result.is_ok());

        let (_, root) = result.unwrap();
        assert!(root.world_def.is_none());
        assert_eq!(root.micro_agent_defs.len(), 2);
    }

    #[test]
    fn test_parse_root_with_only_world() {
        let input = r#"
               world TestWorld {
                   config { }
               }
           "#;

        let result = parse_root(input);
        assert!(result.is_ok());

        let (_, root) = result.unwrap();
        assert!(root.world_def.is_some());
        assert_eq!(root.micro_agent_defs.len(), 0);
    }

    #[test]
    fn test_parse_empty() {
        let input = "   ";
        let result = parse_root(input);
        assert!(result.is_ok());

        let (_, root) = result.unwrap();
        assert!(root.world_def.is_none());
        assert_eq!(root.micro_agent_defs.len(), 0);
    }

    #[test]
    fn test_parse_micro_agent() {
        let input = r#"
            micro TestAgent {
                policy "policy text"
                policy named "named policy text"
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
        assert_eq!(agent.policies[0].text, "policy text".to_string());
        assert_eq!(
            agent.policies[0].scope,
            PolicyScope::Agent("TestAgent".to_string())
        );
        assert_eq!(agent.policies[1].text, "named policy text".to_string());
        assert_eq!(
            agent.policies[1].scope,
            PolicyScope::Agent("TestAgent".to_string())
        );

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
    fn test_parse_assignment_tuple() {
        let input = "(count1, count2) = count + 1";
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
            await count = count + 2
        }"#;
        let result = parse_block(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_statement() {
        let cases = [
            "count = count + 1",
            r#"c = request GetCount to counter()"#,
            "emit Updated to manager",
            "if (count > 10) { return count }",
            "await count = count + 2",
            r#"await {
                count1 = count1 + 1
                count2 = count2 + 2
            }"#,
            r#"emit SearchFlights(...).onFail(err) {
                emit Error(message: err.message)"
            }"#,
            r#"emit Test() onFail { emit Error() }"#,
            r#"emit Test() onFail(err) { emit Error() }"#,
            r#"emit Test()"#,
        ];

        for input in cases {
            let result = parse_statement(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
        }
    }

    #[tokio::test]
    async fn test_parse_await_single() {
        let input = "await think()";
        let result = parse_await(input);
        assert!(result.is_ok());
        let (_, await_expression) = result.unwrap();

        match await_expression {
            Expression::Await(expressions) => {
                assert_eq!(expressions.len(), 1);
            }
            _ => panic!("Unexpected await type"),
        }
    }

    #[tokio::test]
    async fn test_parse_await_block() {
        let input = "await {
            think(1)
            think(2)
           }";
        let result = parse_await_block(input);
        assert!(result.is_ok());
        let (_, await_expression) = result.unwrap();

        match await_expression {
            Expression::Await(expressions) => {
                assert_eq!(expressions.len(), 2);
            }
            _ => panic!("Unexpected await block"),
        }
    }

    #[tokio::test]
    async fn test_parse_await_block_assign() {
        let input = "ret = await {
                think(1)
                think(2)
               }";
        let result = parse_assignment(input);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_block_with_await() {
        let input = "{
            (a1, a2) = await {
                think(1)
                think(2)
            }
            got = think(3)
        }";
        let result = parse_block(input); // トップレベルのパーサー
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_parse_await_block_single_expression() {
        let input = "await { think(1) }";
        let result = parse_await_block(input);
        assert!(result.is_ok());
        let (_, await_statement) = result.unwrap();
        match await_statement {
            Expression::Await(expressions) => {
                assert_eq!(expressions.len(), 1);
            }
            _ => panic!("Unexpected await type"),
        }
    }

    #[tokio::test]
    async fn test_parse_await_block_with_whitespace() {
        let input = "await   {
                think(1)
                    think(2)
                         }";
        let result = parse_await_block(input);
        assert!(result.is_ok());
        let (_, await_statement) = result.unwrap();
        match await_statement {
            Expression::Await(expressions) => {
                assert_eq!(expressions.len(), 2);
            }
            _ => panic!("Unexpected await type"),
        }
    }

    #[tokio::test]
    async fn test_parse_await_block_empty() {
        let input = "await { }";
        let result = parse_await_block(input);
        assert!(result.is_ok());
        let (_, await_statement) = result.unwrap();
        match await_statement {
            Expression::Await(expressions) => {
                assert_eq!(expressions.len(), 0);
            }
            _ => panic!("Unexpected await type"),
        }
    }

    #[tokio::test]
    async fn test_parse_await_block_nested_expressions() {
        let input = "await {
                think(get_data(1))
                process(analyze(2))
            }";
        let result = parse_await_block(input);
        assert!(result.is_ok());
        let (_, await_statement) = result.unwrap();
        match await_statement {
            Expression::Await(expressions) => {
                assert_eq!(expressions.len(), 2);
            }
            _ => panic!("Unexpected await type"),
        }
    }

    #[tokio::test]
    async fn test_parse_await_block_missing_brace() {
        let input = "await {
                think(1)
                think(2)"; // 閉じ括弧がない
        let result = parse_await_block(input);
        assert!(result.is_err());
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
            let result = parse_request_attributes(input);
            assert!(result.is_ok(), "Failed to parse: {}", input);
        }
    }

    #[test]
    fn test_parse_arguments() {
        let inputs = [
            "(name: \"test\")",
            "( count: 10)",
            "(flag:true )",
            "(value)",
            "(count: count + 1)",
            "(name: \"test\", count: 10, flag: true)",
        ];

        for input in inputs {
            let result = parse_arguments(input);
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
                Literal::List(l) => format!(
                    "[{}]",
                    l.iter()
                        .map(|item| format_expression(&Expression::Literal(item.clone())))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Literal::Map(m) => format!(
                    "{{{}}}",
                    m.iter()
                        .map(|(k, v)| format!(
                            "{}: {}",
                            k,
                            format_expression(&Expression::Literal(v.clone()))
                        ))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
                Literal::Duration(d) => format!("{:?}", d),
                Literal::Retry(_) => todo!(),
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
            Expression::Think { .. } => todo!(),
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
            Expression::Ok(expression) => format!("Ok({})", format_expression(expression)),
            Expression::Err(expression) => format!("Err({})", format_expression(expression)),
            Expression::Request {
                agent,
                request_type,
                parameters,
                options,
            } => {
                format!(
                    "request {} {}{}",
                    agent,
                    request_type,
                    if parameters.is_empty() {
                        "".to_string()
                    } else {
                        format!(
                            "({})",
                            parameters
                                .iter()
                                .map(|arg| format!("{:?}", arg))
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    }
                )
            }
            Expression::Await(expressions) => {
                format!(
                    "await {}",
                    expressions
                        .iter()
                        .map(|expr| format_expression(expr))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        }
    }
    #[test]
    fn test_parse_config() {
        let input = r#"
            config {
                tick_interval: 100ms
                max_agents: 1000
                event_buffer_size: 500
            }
        "#;
        let (_, config) = parse_config(input).unwrap();
        assert_eq!(config.tick_interval, Duration::from_millis(100));
        assert_eq!(config.max_agents, 1000);
        assert_eq!(config.event_buffer_size, 500);

        // デフォルト値のテスト
        let input = r#"
            config {
                tick_interval: 1s
            }
        "#;
        let (_, config) = parse_config(input).unwrap();
        assert_eq!(config.tick_interval, Duration::from_secs(1));
        assert_eq!(config.max_agents, 1000); // デフォルト値
        assert_eq!(config.event_buffer_size, 1000); // デフォルト値
    }

    #[test]
    fn test_parse_events() {
        let input = r#"
            events {
                PlayerJoined(player_id: String)
                GameStarted
                PlayerMoved(player_id: String, x: Float, y: Float)
                ScoreUpdated(player_id: String, score: Int)
            }
        "#;
        let (_, events) = parse_events(input).unwrap();
        assert_eq!(events.events.len(), 4);

        // 詳細なイベント定義の検証
        let player_joined = &events.events[0];
        assert_eq!(player_joined.name, "PlayerJoined");
        assert_eq!(player_joined.parameters.len(), 1);
        assert_eq!(player_joined.parameters[0].name, "player_id");
        assert_eq!(
            player_joined.parameters[0].type_info,
            TypeInfo::Custom {
                name: "String".to_string(),
                constraints: HashMap::new()
            }
        );

        let game_started = &events.events[1];
        assert_eq!(game_started.name, "GameStarted");
        assert_eq!(game_started.parameters.len(), 0);

        let player_moved = &events.events[2];
        assert_eq!(player_moved.parameters.len(), 3);
    }

    #[test]
    fn test_parse_handlers() {
        let input = r#"
            handlers {
                on Tick(delta_time: Float) {
                }

                on PlayerJoined(player_id: String) {
                    emit GameStarted()
                }
            }
        "#;
        let (_, handlers) = parse_handlers(input).unwrap();
        assert_eq!(handlers.handlers.len(), 2);

        let tick_handler = &handlers.handlers[0];
        assert_eq!(tick_handler.event_name, "Tick");
        assert_eq!(tick_handler.parameters.len(), 1);
        assert_eq!(tick_handler.parameters[0].name, "delta_time");
        assert_eq!(
            tick_handler.parameters[0].type_info,
            TypeInfo::Custom {
                name: "Float".to_string(),
                constraints: Default::default(),
            }
        );

        let join_handler = &handlers.handlers[1];
        assert_eq!(join_handler.event_name, "PlayerJoined");
        assert_eq!(join_handler.parameters.len(), 1);
    }

    #[test]
    fn test_parse_world() {
        let input = r#"
            world GameWorld {
                config {
                    tick_interval: 16ms
                    max_agents: 100
                    event_buffer_size: 1000
                }

                events {
                    PlayerJoined(player_id: String)
                    GameStarted
                    PlayerMoved(player_id: String, x: Float, y: Float)
                }

                handlers {
                    on Tick(delta_time: Float) {
                        emit NextTick(delta_time)
                    }

                    on PlayerJoined(player_id: String) {
                        emit GameStarted()
                    }
                }
            }
        "#;

        let (_, world) = parse_world(input).unwrap();

        // 基本構造の検証
        assert_eq!(world.name, "GameWorld");

        // Config検証
        let config = world.config.unwrap();
        assert_eq!(config.tick_interval, Duration::from_millis(16));
        assert_eq!(config.max_agents, 100);
        assert_eq!(config.event_buffer_size, 1000);

        // Events検証
        assert_eq!(world.events.events.len(), 3);
        assert_eq!(world.events.events[0].name, "PlayerJoined");
        assert_eq!(world.events.events[1].name, "GameStarted");
        assert_eq!(world.events.events[2].name, "PlayerMoved");

        // Handlers検証
        assert_eq!(world.handlers.handlers.len(), 2);
        assert_eq!(world.handlers.handlers[0].event_name, "Tick");
        assert_eq!(world.handlers.handlers[1].event_name, "PlayerJoined");
    }

    #[test]
    fn test_parse_world_minimal() {
        let input = r#"
            world MinimalWorld {
                config {
                    tick_interval: 100ms
                }
            }
        "#;

        let (_, world) = parse_world(input).unwrap();
        let config = world.config.unwrap();
        assert_eq!(world.name, "MinimalWorld");
        assert_eq!(config.tick_interval, Duration::from_millis(100));
        assert_eq!(world.events.events.len(), 0);
        assert!(world.handlers.handlers.is_empty());
    }

    #[test]
    fn test_parse_world_with_policies() {
        let input = r#"
                world TestWorld {
                    policy "Global policy"
                    policy named_policy "Named global policy"
                }
            "#;

        let (_, world) = parse_world(input).unwrap();
        assert_eq!(world.name, "TestWorld");
        assert_eq!(world.policies.len(), 2);
        assert_eq!(
            world.policies[0].scope,
            PolicyScope::World("TestWorld".to_string())
        );
        assert_eq!(
            world.policies[1].scope,
            PolicyScope::World("TestWorld".to_string())
        );
    }

    #[test]
    fn test_parse_world_errors() {
        // Invalid duration format
        let input = r#"
            world ErrorWorld {
                config {
                    tick_interval: invalid
                }
            }
        "#;
        assert!(parse_world(input).is_err());
    }

    #[test]
    fn test_parse_policy() {
        let context = PolicyParseContext {
            scope: PolicyScope::World("TestWorld".to_string()),
        };

        let (rest, policy) = parse_policy(r#"policy "Be concise""#, &context).unwrap();
        assert!(rest.is_empty());
        assert_eq!(policy.text, "Be concise");
        assert_eq!(policy.scope, PolicyScope::World("TestWorld".to_string()));
    }

    #[test]
    fn test_parse_named_policy() {
        let context = PolicyParseContext {
            scope: PolicyScope::World("TestWorld".to_string()),
        };

        let (rest, policy) =
            parse_policy(r#"policy tech_terms "Use technical terms""#, &context).unwrap();
        assert!(rest.is_empty());
        assert_eq!(policy.text, "Use technical terms");
        assert_eq!(policy.scope, PolicyScope::World("TestWorld".to_string()));
        assert!(matches!(policy.internal_id, PolicyId(id) if id.contains("tech_terms")));
    }

    #[test]
    fn test_basic_think() {
        let input = r#"think("query")"#;
        let (rem, expr) = parse_expression(input).unwrap();
        assert!(rem.is_empty());

        match expr {
            Expression::Think { args, with_block } => {
                assert_eq!(args.len(), 1);
                let arg = args[0].clone();
                assert!(matches!(
                    arg.clone(),
                    Argument::Positional(Expression::Literal(Literal::String(_))),
                ));
                match arg {
                    Argument::Positional(Expression::Literal(Literal::String(query))) => {
                        assert_eq!(query, "query");
                    }
                    _ => panic!("Expected string literal"),
                }
                assert!(with_block.is_none());
            }
            _ => panic!("Expected Think expression"),
        }
    }

    #[test]
    fn test_think_with_multiple_args() {
        let input = r#"think("query", x + 1, true)"#;
        let (_, expr) = parse_expression(input).unwrap();

        if let Expression::Think { args, with_block } = expr {
            assert_eq!(args.len(), 3);
            assert!(matches!(
                args[0],
                Argument::Positional(Expression::Literal(Literal::String(_)))
            ));
            // 2番目の引数は加算式
            assert!(matches!(
                args[1],
                Argument::Positional(Expression::BinaryOp { .. })
            ));
            assert!(matches!(
                args[2],
                Argument::Positional(Expression::Literal(Literal::Boolean(_)))
            ));
            assert!(with_block.is_none());
        } else {
            panic!("Expected Think expression");
        }
    }

    #[test]
    fn test_think_with_basic_settings() {
        let input = r#"think("query") with {
                    provider: "gpt4",
                    temperature: 0.7,
                    max_tokens: 1000
                }"#;

        let (_, expr) = parse_expression(input).unwrap();
        if let Expression::Think {
            with_block: Some(block),
            ..
        } = expr
        {
            assert_eq!(block.provider, Some("gpt4".to_string()));
            assert_eq!(block.temperature, Some(0.7));
            assert_eq!(block.max_tokens, Some(1000));
        } else {
            panic!("Expected Think expression with settings");
        }
    }
    #[test]
    fn test_think_with_retry_config() {
        let input = r#"think("query") with {
                provider: "gpt4",
                retry: {
                    max_attempts: 3,
                    delay: fixed 1000
                }
            }"#;

        let (_, expr) = parse_expression(input).unwrap();
        if let Expression::Think {
            with_block: Some(block),
            ..
        } = expr
        {
            if let Some(retry) = block.retry {
                assert_eq!(retry.max_attempts, 3);
                assert!(matches!(retry.delay, RetryDelay::Fixed(1000)));
            } else {
                panic!("Expected retry config");
            }
        } else {
            panic!("Expected Think expression with retry");
        }
    }

    #[test]
    fn test_think_with_exponential_retry() {
        let input = r#"think("query") with {
                retry: {
                    max_attempts: 3,
                    delay: exponential {
                        initial: 1000,
                        max: 5000
                    }
                }
            }"#;

        let (_, expr) = parse_expression(input).unwrap();
        if let Expression::Think {
            with_block: Some(block),
            ..
        } = expr
        {
            if let Some(retry) = block.retry {
                assert_eq!(retry.max_attempts, 3);
                if let RetryDelay::Exponential { initial, max } = retry.delay {
                    assert_eq!(initial, 1000);
                    assert_eq!(max, 5000);
                } else {
                    panic!("Expected exponential delay");
                }
            }
        }
    }

    #[test]
    fn test_think_in_expressions() {
        // 乗算との組み合わせ
        let input = r#"2 * think("value")"#;
        let (_, expr) = parse_expression(input).unwrap();
        assert!(matches!(expr, Expression::BinaryOp { .. }));

        // 関数呼び出しの引数として
        let input = r#"format("{}", think("query"))"#;
        let (_, expr) = parse_expression(input).unwrap();
        assert!(matches!(expr, Expression::FunctionCall { .. }));

        // 括弧内での使用
        let input = r#"(think("q1") + think("q2")) / 2"#;
        let (_, expr) = parse_expression(input).unwrap();
        assert!(matches!(expr, Expression::BinaryOp { .. }));
    }
    #[test]
    fn test_think_whitespace_handling() {
        let inputs = vec![
            r#"think ( "query" )"#,
            r#"think("query")  with  {  provider:  "gpt4"  }"#,
            r#"think("query")with{provider:"gpt4"}"#,
        ];

        for input in inputs {
            assert!(parse_expression(input).is_ok());
        }
    }

    #[test]
    fn test_think_error_like_cases() {
        // 閉じていない括弧
        assert!(parse_think_expression(r#"think("query""#).is_err());

        // パーシャルなwithブロックは、エラーにならない(withが残りのinputとして残る)
        assert!(parse_think_expression(r#"think("query") with {"#).is_ok());

        // with ブロック内のリトライがエラーでも、with ブロックのエラーになるのでパースは成功する
        assert!(parse_think_expression(
            r#"think("query") with {
                retry: {
                    max_attempts: true  // should be number
                }
            }"#
        )
        .is_ok());

        // 不正なリトライの値はエラーになる
        assert!(parse_retry(
            r#"{
                    max_attempts: true  // should be number
            }"#
        )
        .is_err());
    }

    #[test]
    fn test_think_nested() {
        // Think式の入れ子
        let input = r#"think(format("Result: {}", think("inner query")))"#;
        assert!(parse_expression(input).is_ok());
    }

    #[test]
    fn test_think_with_variable_arguments() {
        let input = r#"think(query_text + " additional context")"#;
        let (_, expr) = parse_expression(input).unwrap();
        if let Expression::Think { args, .. } = expr {
            assert_eq!(args.len(), 1);
            assert!(matches!(
                args[0],
                Argument::Positional(Expression::BinaryOp { .. })
            ));
        }
    }

    #[test]
    fn test_think_with_nested_attributes() {
        let input = r#"think(query_text) with {
            provider: "openai",
            model: "gpt-4",
            my_plugin: {
                version : "v1",
                settings: {
                    timeout: 30,
                    retries: 3
                }
            }
        }"#;

        let (_, expr) = parse_expression(input).unwrap();

        if let Expression::Think {
            args, with_block, ..
        } = expr
        {
            assert_eq!(args.len(), 1);
            assert!(matches!(
                args[0],
                Argument::Positional(Expression::Variable { .. })
            ));

            // 属性のチェック
            if let Some(attrs) = with_block {
                assert_eq!(attrs.provider, Some("openai".to_string()));
                assert_eq!(attrs.model, Some("gpt-4".to_string()));

                // プラグイン設定のチェック
                let plugin_config = attrs
                    .plugins
                    .get("my_plugin")
                    .expect("my_plugin should exist");

                assert_eq!(
                    plugin_config.get("version"),
                    Some(&Literal::String("v1".to_string()))
                );

                if let Some(Literal::Map(settings)) = plugin_config.get("settings") {
                    assert_eq!(settings.get("timeout"), Some(&Literal::Integer(30)));
                    assert_eq!(settings.get("retries"), Some(&Literal::Integer(3)));
                } else {
                    panic!("settings should be a map");
                }
            } else {
                panic!("attributes should exist");
            }
        } else {
            panic!("should be a think expression");
        }
    }

    #[test]
    fn test_think_with_mixed_attributes() {
        let input = r#"think(query_text) with {
            provider: "openai",
            temperature: 0.7,
            plugin_one: {
                enabled: true
            },
            plugin_two: {
                config: {
                    value: "nested"
                }
            }
        }"#;

        let (_, expr) = parse_expression(input).unwrap();

        if let Expression::Think {
            args, with_block, ..
        } = expr
        {
            assert_eq!(args.len(), 1);

            if let Some(attrs) = with_block {
                assert_eq!(attrs.provider, Some("openai".to_string()));
                assert_eq!(attrs.temperature, Some(0.7));

                // プラグイン設定のチェック
                let plugin_one = attrs
                    .plugins
                    .get("plugin_one")
                    .expect("plugin_one should exist");
                assert_eq!(plugin_one.get("enabled"), Some(&Literal::Boolean(true)));

                let plugin_two = attrs
                    .plugins
                    .get("plugin_two")
                    .expect("plugin_two should exist");
                if let Some(Literal::Map(config)) = plugin_two.get("config") {
                    assert_eq!(
                        config.get("value"),
                        Some(&Literal::String("nested".to_string()))
                    );
                } else {
                    panic!("config should be a map");
                }
            } else {
                panic!("attributes should exist");
            }
        } else {
            panic!("should be a think expression");
        }
    }

    #[test]
    fn test_parse_request_expression() {
        let input = "request GetCount to counter(last_updated)";
        let (_, statement) = parse_request_expression(input).unwrap();
        assert_eq!(
            statement,
            Expression::Request {
                agent: "counter".to_string(),
                request_type: RequestType::Custom("GetCount".to_string()),
                parameters: vec![Argument::Positional(Expression::Variable(
                    "last_updated".to_string()
                ))],
                options: None
            }
        );
    }
}
