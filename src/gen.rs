use crate::ast::*;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::CodeGen;

impl CodeGen for MicroAgentDef {
    fn generate_rust(&self) -> TokenStream {
        let agent_name = format_ident!("{}", &self.name);
        let lifecycle = self.lifecycle.as_ref().map(|l| l.generate_rust());
        let state = self.state.as_ref().map(|s| s.generate_rust());
        let observe = self.observe.as_ref().map(|o| o.generate_rust());
        let answer = self.answer.as_ref().map(|a| a.generate_rust());
        let react = self.react.as_ref().map(|r| r.generate_rust());

        quote! {
            struct #agent_name {
                #state
            }

            impl #agent_name {
                #lifecycle
                #observe
                #answer
                #react
            }
        }
    }
}

impl CodeGen for LifecycleDef {
    fn generate_rust(&self) -> TokenStream {
        let on_init = self.on_init.as_ref().map(|b| {
            let stmts = b.generate_rust();
            quote! {
                fn on_init(&mut self) {
                    #stmts
                }
            }
        });

        let on_destroy = self.on_destroy.as_ref().map(|b| {
            let stmts = b.generate_rust();
            quote! {
                fn on_destroy(&mut self) {
                    #stmts
                }
            }
        });

        quote! {
            #on_init
            #on_destroy
        }
    }
}

impl CodeGen for StateDef {
    fn generate_rust(&self) -> TokenStream {
        let vars = self.variables.values().map(|v| v.generate_rust());

        quote! {
            #(#vars)*
        }
    }
}

impl CodeGen for StateVarDef {
    fn generate_rust(&self) -> TokenStream {
        let name = format_ident!("{}", &self.name);
        let type_info = self.type_info.generate_rust();
        // let initial_value = self.initial_value.as_ref().map(|e| e.generate_rust()); // 初期値は一旦無視

        quote! {
            #name: #type_info,
        }
    }
}

impl CodeGen for ObserveDef {
    fn generate_rust(&self) -> TokenStream {
        let handlers = self.handlers.iter().map(|h| h.generate_rust());

        quote! {
            fn handle_event(&mut self, event: &Event) {
                match event {
                    #(#handlers),*
                    _ => {}
                }
            }
        }
    }
}

impl CodeGen for AnswerDef {
    fn generate_rust(&self) -> TokenStream {
        let handlers = self.handlers.iter().map(|h| h.generate_rust());

        quote! {
            #(#handlers)*
        }
    }
}

impl CodeGen for ReactDef {
    fn generate_rust(&self) -> TokenStream {
        let handlers = self.handlers.iter().map(|h| h.generate_rust());

        quote! {
            fn handle_event(&mut self, event: &Event) {
                match event {
                    #(#handlers),*
                    _ => {}
                }
            }
        }
    }
}

impl CodeGen for EventHandler {
    fn generate_rust(&self) -> TokenStream {
        let event_type = match &self.event_type {
            EventType::Tick => quote! { Event::Tick },
            EventType::StateUpdated {
                agent_name,
                state_name,
            } => {
                quote! { Event::StateUpdated { agent: #agent_name, state: #state_name } }
            }
            EventType::Message { content_type } => {
                quote! { Event::Message { content_type: #content_type } }
            }
            EventType::Custom(event_type) => {
                quote! { Event::Custom(#event_type) }
            }
        };

        let block = self.block.generate_rust();

        quote! {
            #event_type => {
                #block
            }
        }
    }
}

impl CodeGen for RequestHandler {
    fn generate_rust(&self) -> TokenStream {
        let request_type = match &self.request_type {
            RequestType::Query { query_type } => format_ident!("{}", query_type),
            RequestType::Action { action_type } => format_ident!("{}", action_type),
            RequestType::Custom(request_type) => format_ident!("{}", request_type),
        };

        let params = self.parameters.iter().map(|p| p.generate_rust());
        let return_type = self.return_type.generate_rust();
        let constraints = self.constraints.as_ref().map(|c| c.generate_rust());
        let block = self.block.generate_rust();

        quote! {
            fn #request_type(&mut self, #(#params),*) -> #return_type {
                #constraints
                #block
            }
        }
    }
}

impl CodeGen for Parameter {
    fn generate_rust(&self) -> TokenStream {
        let name = format_ident!("{}", &self.name);
        let type_info = self.type_info.generate_rust();

        quote! {
            #name: #type_info
        }
    }
}

impl CodeGen for Constraints {
    fn generate_rust(&self) -> TokenStream {
        // 今は利用しない
        quote! {}
    }
}

impl CodeGen for TypeInfo {
    fn generate_rust(&self) -> TokenStream {
        match self {
            TypeInfo::Simple(type_name) => {
                let type_ident = format_ident!("{}", type_name);
                quote! { #type_ident }
            }
            TypeInfo::Result { ok_type, err_type } => {
                let ok_type_tokens = ok_type.generate_rust();
                let err_type_tokens = err_type.generate_rust();
                quote! { Result<#ok_type_tokens, #err_type_tokens> }
            }
            TypeInfo::Option(inner_type) => {
                let inner_type_tokens = inner_type.generate_rust();
                quote! { Option<#inner_type_tokens> }
            }
            TypeInfo::Array(item_type) => {
                let item_type_tokens = item_type.generate_rust();
                quote! { Vec<#item_type_tokens> }
            }
            TypeInfo::Map(key_type, value_type) => {
                let key_type_tokens = key_type.generate_rust();
                let value_type_tokens = value_type.generate_rust();
                quote! { HashMap<#key_type_tokens, #value_type_tokens> }
            }
            TypeInfo::Custom { name, .. } => {
                let type_ident = format_ident!("{}", name);
                // 今は利用しない
                quote! { #type_ident }
            }
        }
    }
}

impl CodeGen for HandlerBlock {
    fn generate_rust(&self) -> TokenStream {
        let stmts = self.statements.iter().map(|s| s.generate_rust());

        if self.statements.is_empty() {
            quote! {} // ステートメントが空の場合は、何も生成しない
        } else {
            quote! {
                #(#stmts;)*
            }
        }
    }
}

impl CodeGen for Statement {
    fn generate_rust(&self) -> TokenStream {
        match self {
            Statement::Await { .. } => {
                // 今は利用しない
                quote! {}
            }
            Statement::Assignment { target, value } => {
                let target_tokens = target.generate_rust();
                let value_tokens = value.generate_rust();
                quote! {
                     #target_tokens = #value_tokens
                }
            }
            Statement::Emit { .. } => {
                // 今は利用しない
                quote! {}
            }
            Statement::Request { .. } => {
                // 今は利用しない
                quote! {}
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                let condition_tokens = condition.generate_rust();
                let then_block_tokens = then_block.generate_rust();
                let else_block_tokens = else_block.as_ref().map(|b| {
                    let else_block_stmts = b.generate_rust();
                    quote! { else { #else_block_stmts } }
                });

                quote! {
                    if #condition_tokens {
                        #then_block_tokens
                    } #else_block_tokens
                }
            }
            Statement::Expression(expr) => {
                let expr_tokens = expr.generate_rust();
                quote! { #expr_tokens; }
            }
            Statement::Block(statments) => {
                let stmts_tokens = statments.iter().map(|s| s.generate_rust());
                quote! { {  #(#stmts_tokens;)* } }
            }
            Statement::Return(expr) => {
                let expr_tokens = expr.generate_rust();
                quote! {
                    return #expr_tokens;
                }
            }
            // TODO: error handling not supported
            Statement::WithError { statement, .. } => {
                let statement = statement.generate_rust();
                quote! {
                    #statement;
                }
            }
        }
    }
}

impl CodeGen for Vec<Statement> {
    fn generate_rust(&self) -> TokenStream {
        let stmts = self.iter().map(|s| s.generate_rust());
        quote! { #(#stmts;)* }
    }
}

impl CodeGen for Expression {
    fn generate_rust(&self) -> TokenStream {
        match self {
            Expression::Literal(lit) => lit.generate_rust(),
            Expression::Variable(var) => {
                let var_ident = format_ident!("{}", var);
                quote! { #var_ident }
            }
            Expression::StateAccess(path) => {
                let path_segments = path.0.iter().map(|s| format_ident!("{}", s));
                quote! { #(#path_segments).* }
            }
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                let func_ident = format_ident!("{}", function);
                let args_tokens = arguments.iter().map(|arg| arg.generate_rust());
                quote! { #func_ident(#(#args_tokens),*) }
            }
            Expression::BinaryOp { op, left, right } => {
                let left_tokens = left.generate_rust();
                let right_tokens = right.generate_rust();
                let op_tokens = op.generate_rust();
                quote! { #left_tokens #op_tokens #right_tokens }
            }
            Expression::Think { .. } => {
                todo!()
            }
        }
    }
}

impl CodeGen for Literal {
    fn generate_rust(&self) -> TokenStream {
        match self {
            Literal::Integer(i) => quote! { #i },
            Literal::Float(f) => quote! { #f },
            Literal::String(s) => quote! { #s },
            Literal::Boolean(b) => quote! { #b },
            Literal::Duration(d) => {
                let secs = d.as_secs();
                quote! { Duration::from_secs(#secs) }
            }
            Literal::List(l) => {
                let items = l.iter().map(|item| item.generate_rust());
                quote! { vec![#(#items),*] }
            }
            Literal::Map(m) => {
                let items = m.iter().map(|(k, v)| {
                    let value = v.generate_rust();
                    quote! { (#k, #value) }
                });
                quote! { vec![#(#items),*].into_iter().collect() }
            }
            Literal::Null => quote! { None },
            Literal::Retry(_) => {
                todo!()
            }
        }
    }
}

impl CodeGen for BinaryOperator {
    fn generate_rust(&self) -> TokenStream {
        match self {
            BinaryOperator::Add => quote! { + },
            BinaryOperator::Subtract => quote! { - },
            BinaryOperator::Multiply => quote! { * },
            BinaryOperator::Divide => quote! { / },
            BinaryOperator::Equal => quote! { == },
            BinaryOperator::NotEqual => quote! { != },
            BinaryOperator::LessThan => quote! { < },
            BinaryOperator::GreaterThan => quote! { > },
            BinaryOperator::LessThanEqual => quote! { <= },
            BinaryOperator::GreaterThanEqual => quote! { >= },
            BinaryOperator::And => quote! { && },
            BinaryOperator::Or => quote! { || },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::ast::*;

    #[test]
    fn test_micro_agent_def() {
        let micro_agent = MicroAgentDef {
            name: "TestAgent".to_string(),
            policies: vec![],
            lifecycle: Some(LifecycleDef {
                on_init: Some(HandlerBlock {
                    statements: vec![Statement::Assignment {
                        target: Expression::StateAccess(StateAccessPath(vec![
                            "self".to_string(),
                            "counter".to_string(),
                        ])),
                        value: Expression::Literal(Literal::Integer(0)),
                    }],
                }),
                on_destroy: None,
            }),
            state: Some(StateDef {
                variables: {
                    let mut vars = HashMap::new();
                    vars.insert(
                        "counter".to_string(),
                        StateVarDef {
                            name: "counter".to_string(),
                            type_info: TypeInfo::Simple("i64".to_string()),
                            initial_value: Some(Expression::Literal(Literal::Integer(0))),
                        },
                    );
                    vars
                },
            }),
            observe: None,
            answer: None,
            react: None,
        };

        let expected = quote! {
            struct TestAgent {
                counter: i64,
            }

            impl TestAgent {
                fn on_init(&mut self) {
                    self.counter = 0i64;
                }
            }
        };

        assert_eq!(
            micro_agent.generate_rust().to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn test_lifecycle_def() {
        let lifecycle = LifecycleDef {
            on_init: Some(HandlerBlock {
                statements: vec![Statement::Assignment {
                    target: Expression::StateAccess(StateAccessPath(vec![
                        "self".to_string(),
                        "counter".to_string(),
                    ])),
                    value: Expression::Literal(Literal::Integer(0)),
                }],
            }),
            on_destroy: Some(HandlerBlock {
                statements: vec![Statement::Emit {
                    event_type: EventType::Custom("destroy".to_string()),
                    parameters: vec![],
                    target: Some("manager".to_string()),
                }],
            }),
        };

        let expected = quote! {
            fn on_init(&mut self) {
                self.counter = 0i64;
            }
            fn on_destroy(&mut self) {
                ;
            }
        };

        assert_eq!(lifecycle.generate_rust().to_string(), expected.to_string());
    }

    #[test]
    fn test_state_def() {
        let state = StateDef {
            variables: {
                let mut vars = HashMap::new();
                vars.insert(
                    "counter".to_string(),
                    StateVarDef {
                        name: "counter".to_string(),
                        type_info: TypeInfo::Simple("i64".to_string()),
                        initial_value: Some(Expression::Literal(Literal::Integer(0))),
                    },
                );
                vars.insert(
                    "name".to_string(),
                    StateVarDef {
                        name: "name".to_string(),
                        type_info: TypeInfo::Simple("String".to_string()),
                        initial_value: Some(Expression::Literal(Literal::String(
                            "test".to_string(),
                        ))),
                    },
                );
                vars
            },
        };

        assert!(state
            .generate_rust()
            .to_string()
            .contains("counter : i64 ,"));
        assert!(state
            .generate_rust()
            .to_string()
            .contains("name : String ,"));
    }

    #[test]
    fn test_state_var_def() {
        let state_var = StateVarDef {
            name: "counter".to_string(),
            type_info: TypeInfo::Simple("i64".to_string()),
            initial_value: Some(Expression::Literal(Literal::Integer(0))),
        };

        let expected = quote! {
            counter: i64,
        };

        assert_eq!(state_var.generate_rust().to_string(), expected.to_string());
    }

    #[test]
    fn test_observe_def() {
        let observe = ObserveDef {
            handlers: vec![EventHandler {
                event_type: EventType::Tick,
                parameters: vec![],
                block: HandlerBlock {
                    statements: vec![Statement::Assignment {
                        target: Expression::StateAccess(StateAccessPath(vec![
                            "self".to_string(),
                            "counter".to_string(),
                        ])),
                        value: Expression::BinaryOp {
                            op: BinaryOperator::Add,
                            left: Box::new(Expression::StateAccess(StateAccessPath(vec![
                                "self".to_string(),
                                "counter".to_string(),
                            ]))),
                            right: Box::new(Expression::Literal(Literal::Integer(1))),
                        },
                    }],
                },
            }],
        };

        let expected = quote! {
            fn handle_event(&mut self, event: &Event) {
                match event {
                    Event::Tick => {
                        self.counter = self.counter + 1i64;
                    }
                    _ => {}
                }
            }
        };

        assert_eq!(observe.generate_rust().to_string(), expected.to_string());
    }

    #[test]
    fn test_answer_def() {
        let answer = AnswerDef {
            handlers: vec![RequestHandler {
                request_type: RequestType::Query {
                    query_type: "GetCount".to_string(),
                },
                parameters: vec![],
                return_type: TypeInfo::Result {
                    ok_type: Box::new(TypeInfo::Simple("i64".to_string())),
                    err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                },
                constraints: None,
                block: HandlerBlock {
                    statements: vec![Statement::Return(Expression::StateAccess(StateAccessPath(
                        vec!["self".to_string(), "counter".to_string()],
                    )))],
                },
            }],
        };

        let expected = quote! {
            fn GetCount(&mut self, ) -> Result<i64, Error> {
                return self.counter;
                ;
            }
        };

        assert_eq!(answer.generate_rust().to_string(), expected.to_string());
    }

    #[test]
    fn test_react_def() {
        let react = ReactDef {
            handlers: vec![EventHandler {
                event_type: EventType::Message {
                    content_type: "reset".to_string(),
                },
                parameters: vec![],
                block: HandlerBlock {
                    statements: vec![
                        Statement::Assignment {
                            target: Expression::StateAccess(StateAccessPath(vec![
                                "self".to_string(),
                                "counter".to_string(),
                            ])),
                            value: Expression::Literal(Literal::Integer(0)),
                        },
                        Statement::Emit {
                            event_type: EventType::StateUpdated {
                                agent_name: "self".to_string(),
                                state_name: "counter".to_string(),
                            },
                            parameters: vec![Argument::Positional(Expression::Literal(
                                Literal::String("counter".to_string()),
                            ))],
                            target: Some("manager".to_string()),
                        },
                    ],
                },
            }],
        };

        let expected = quote! {
            fn handle_event(&mut self, event: &Event) {
                match event {
                    Event::Message { content_type: "reset" } => {
                        self.counter = 0i64;
                        ;
                    }
                    _ => {}
                }
            }
        };

        assert_eq!(react.generate_rust().to_string(), expected.to_string());
    }

    #[test]
    fn test_event_handler() {
        let event_handler = EventHandler {
            event_type: EventType::Tick,
            parameters: vec![],
            block: HandlerBlock {
                statements: vec![Statement::Assignment {
                    target: Expression::StateAccess(StateAccessPath(vec![
                        "self".to_string(),
                        "counter".to_string(),
                    ])),
                    value: Expression::BinaryOp {
                        op: BinaryOperator::Add,
                        left: Box::new(Expression::StateAccess(StateAccessPath(vec![
                            "self".to_string(),
                            "counter".to_string(),
                        ]))),
                        right: Box::new(Expression::Literal(Literal::Integer(1))),
                    },
                }],
            },
        };

        let expected = quote! {
            Event::Tick => {
                self.counter = self.counter + 1i64;
            }
        };

        assert_eq!(
            event_handler.generate_rust().to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn test_request_handler() {
        let request_handler = RequestHandler {
            request_type: RequestType::Query {
                query_type: "GetCount".to_string(),
            },
            parameters: vec![],
            return_type: TypeInfo::Result {
                ok_type: Box::new(TypeInfo::Simple("i64".to_string())),
                err_type: Box::new(TypeInfo::Simple("Error".to_string())),
            },
            constraints: None,
            block: HandlerBlock {
                statements: vec![Statement::Return(Expression::StateAccess(StateAccessPath(
                    vec!["self".to_string(), "counter".to_string()],
                )))],
            },
        };

        let expected = quote! {
            fn GetCount(&mut self, ) -> Result<i64, Error> {
                return self.counter;
                ;
            }
        };

        assert_eq!(
            request_handler.generate_rust().to_string(),
            expected.to_string()
        );
    }

    #[test]
    fn test_parameter() {
        let parameter = Parameter {
            name: "count".to_string(),
            type_info: TypeInfo::Simple("i64".to_string()),
        };

        let expected = quote! {
            count: i64
        };

        assert_eq!(parameter.generate_rust().to_string(), expected.to_string());
    }

    #[test]
    fn test_type_info() {
        // テストデータの作成
        let simple_type = TypeInfo::Simple("i64".to_string());
        let result_type = TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("String".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        };
        let option_type = TypeInfo::Option(Box::new(TypeInfo::Simple("f64".to_string())));
        let array_type = TypeInfo::Array(Box::new(TypeInfo::Simple("bool".to_string())));

        // 期待される出力
        let expected_simple = quote! { i64 };
        let expected_result = quote! { Result<String, Error> };
        let expected_option = quote! { Option<f64> };
        let expected_array = quote! { Vec<bool> };

        // テストの実行
        assert_eq!(
            simple_type.generate_rust().to_string(),
            expected_simple.to_string()
        );
        assert_eq!(
            result_type.generate_rust().to_string(),
            expected_result.to_string()
        );
        assert_eq!(
            option_type.generate_rust().to_string(),
            expected_option.to_string()
        );
        assert_eq!(
            array_type.generate_rust().to_string(),
            expected_array.to_string()
        );
    }

    #[test]
    fn test_block() {
        let block = HandlerBlock {
            statements: vec![
                Statement::Assignment {
                    target: Expression::StateAccess(StateAccessPath(vec![
                        "self".to_string(),
                        "counter".to_string(),
                    ])),
                    value: Expression::Literal(Literal::Integer(0)),
                },
                Statement::Return(Expression::StateAccess(StateAccessPath(vec![
                    "self".to_string(),
                    "counter".to_string(),
                ]))),
            ],
        };

        let expected = quote! {
            self.counter = 0i64;
            return self.counter;
            ;
        };

        assert_eq!(block.generate_rust().to_string(), expected.to_string());
    }

    #[test]
    fn test_statement() {
        // 各ステートメントタイプのテストデータを準備
        let assignment = Statement::Assignment {
            target: Expression::StateAccess(StateAccessPath(vec![
                "self".to_string(),
                "counter".to_string(),
            ])),
            value: Expression::Literal(Literal::Integer(10)),
        };
        let if_statement = Statement::If {
            condition: Expression::BinaryOp {
                op: BinaryOperator::Equal,
                left: Box::new(Expression::StateAccess(StateAccessPath(vec![
                    "self".to_string(),
                    "counter".to_string(),
                ]))),
                right: Box::new(Expression::Literal(Literal::Integer(0))),
            },
            then_block: vec![Statement::Assignment {
                target: Expression::StateAccess(StateAccessPath(vec![
                    "self".to_string(),
                    "counter".to_string(),
                ])),
                value: Expression::Literal(Literal::Integer(1)),
            }],
            else_block: Some(vec![Statement::Assignment {
                target: Expression::StateAccess(StateAccessPath(vec![
                    "self".to_string(),
                    "counter".to_string(),
                ])),
                value: Expression::Literal(Literal::Integer(2)),
            }]),
        };
        let return_statement = Statement::Return(Expression::Literal(Literal::Integer(42)));

        // 期待される出力を準備
        let expected_assignment = quote! { self.counter = 10i64 };
        let expected_if = quote! {
            if self.counter == 0i64 {
                self.counter = 1i64;
            } else {
                self.counter = 2i64;
            }
        };
        let expected_return = quote! { return 42i64; };

        // テストを実行
        assert_eq!(
            assignment.generate_rust().to_string(),
            expected_assignment.to_string()
        );
        assert_eq!(
            if_statement.generate_rust().to_string(),
            expected_if.to_string()
        );
        assert_eq!(
            return_statement.generate_rust().to_string(),
            expected_return.to_string()
        );
    }

    #[test]
    fn test_expression() {
        let literal_expr = Expression::Literal(Literal::Integer(42));
        let variable_expr = Expression::Variable("x".to_string());
        let state_access_expr = Expression::StateAccess(StateAccessPath(vec![
            "self".to_string(),
            "counter".to_string(),
        ]));
        let function_call_expr = Expression::FunctionCall {
            function: "foo".to_string(),
            arguments: vec![
                Expression::Literal(Literal::Integer(1)),
                Expression::Literal(Literal::Integer(2)),
            ],
        };
        let binary_op_expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Literal(Literal::Integer(10))),
            right: Box::new(Expression::Literal(Literal::Integer(20))),
        };

        let expected_literal = quote! { 42i64 };
        let expected_variable = quote! { x };
        let expected_state_access = quote! { self.counter };
        let expected_function_call = quote! { foo(1i64, 2i64) };
        let expected_binary_op = quote! { 10i64 + 20i64 };

        assert_eq!(
            literal_expr.generate_rust().to_string(),
            expected_literal.to_string()
        );
        assert_eq!(
            variable_expr.generate_rust().to_string(),
            expected_variable.to_string()
        );
        assert_eq!(
            state_access_expr.generate_rust().to_string(),
            expected_state_access.to_string()
        );
        assert_eq!(
            function_call_expr.generate_rust().to_string(),
            expected_function_call.to_string()
        );
        assert_eq!(
            binary_op_expr.generate_rust().to_string(),
            expected_binary_op.to_string()
        );
    }

    #[test]
    fn test_literal() {
        let int_literal = Literal::Integer(42);
        let float_literal = Literal::Float(3.14);
        let string_literal = Literal::String("hello".to_string());
        let bool_literal = Literal::Boolean(true);
        let null_literal = Literal::Null;

        let expected_int = quote! { 42i64 };
        let expected_float = quote! { 3.14f64 };
        let expected_string = quote! { "hello" };
        let expected_bool = quote! { true };
        let expected_null = quote! { None };

        assert_eq!(
            int_literal.generate_rust().to_string(),
            expected_int.to_string()
        );
        assert_eq!(
            float_literal.generate_rust().to_string(),
            expected_float.to_string()
        );
        assert_eq!(
            string_literal.generate_rust().to_string(),
            expected_string.to_string()
        );
        assert_eq!(
            bool_literal.generate_rust().to_string(),
            expected_bool.to_string()
        );
        assert_eq!(
            null_literal.generate_rust().to_string(),
            expected_null.to_string()
        );
    }

    #[test]
    fn test_binary_operator() {
        let add_op = BinaryOperator::Add;
        let subtract_op = BinaryOperator::Subtract;
        let multiply_op = BinaryOperator::Multiply;
        let divide_op = BinaryOperator::Divide;
        let equal_op = BinaryOperator::Equal;
        let not_equal_op = BinaryOperator::NotEqual;
        let less_than_op = BinaryOperator::LessThan;
        let greater_than_op = BinaryOperator::GreaterThan;
        let less_than_equal_op = BinaryOperator::LessThanEqual;
        let greater_than_equal_op = BinaryOperator::GreaterThanEqual;
        let and_op = BinaryOperator::And;
        let or_op = BinaryOperator::Or;

        let expected_add = quote! { + };
        let expected_subtract = quote! { - };
        let expected_multiply = quote! { * };
        let expected_divide = quote! { / };
        let expected_equal = quote! { == };
        let expected_not_equal = quote! { != };
        let expected_less_than = quote! { < };
        let expected_greater_than = quote! { > };
        let expected_less_than_equal = quote! { <= };
        let expected_greater_than_equal = quote! { >= };
        let expected_and = quote! { && };
        let expected_or = quote! { || };

        assert_eq!(add_op.generate_rust().to_string(), expected_add.to_string());
        assert_eq!(
            subtract_op.generate_rust().to_string(),
            expected_subtract.to_string()
        );
        assert_eq!(
            multiply_op.generate_rust().to_string(),
            expected_multiply.to_string()
        );
        assert_eq!(
            divide_op.generate_rust().to_string(),
            expected_divide.to_string()
        );
        assert_eq!(
            equal_op.generate_rust().to_string(),
            expected_equal.to_string()
        );
        assert_eq!(
            not_equal_op.generate_rust().to_string(),
            expected_not_equal.to_string()
        );
        assert_eq!(
            less_than_op.generate_rust().to_string(),
            expected_less_than.to_string()
        );
        assert_eq!(
            greater_than_op.generate_rust().to_string(),
            expected_greater_than.to_string()
        );
        assert_eq!(
            less_than_equal_op.generate_rust().to_string(),
            expected_less_than_equal.to_string()
        );
        assert_eq!(
            greater_than_equal_op.generate_rust().to_string(),
            expected_greater_than_equal.to_string()
        );
        assert_eq!(and_op.generate_rust().to_string(), expected_and.to_string());
        assert_eq!(or_op.generate_rust().to_string(), expected_or.to_string());
    }
}
