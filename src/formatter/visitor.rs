use crate::ast::*;
use crate::formatter::config::FormatterConfig;
use crate::formatter::error::FormatterError;
use std::time::Duration;

pub struct FormatterVisitor {
    config: FormatterConfig,
    indent_level: usize,
    output: String,
}

impl FormatterVisitor {
    pub fn new(config: FormatterConfig) -> Self {
        Self {
            config,
            indent_level: 0,
            output: String::new(),
        }
    }

    pub fn format_root(&mut self, root: &Root) -> Result<String, FormatterError> {
        // Format world definition if exists
        if let Some(world) = &root.world_def {
            self.format_world(world)?;
            self.newline()?;
        }

        // Format micro agents
        for agent in &root.micro_agent_defs {
            self.format_micro_agent(agent)?;
            self.newline()?;
        }

        Ok(self.output.clone())
    }

    fn format_world(&mut self, world: &WorldDef) -> Result<(), FormatterError> {
        self.write("world ")?;
        self.write(&world.name)?;
        self.write(" {")?;
        self.indent();
        self.newline()?;

        // Format policies
        for policy in &world.policies {
            self.write("policy ")?;
            self.write(&format!("\"{}\"", policy.text))?;
            self.newline()?;
        }

        // Format config if present
        if let Some(config) = &world.config {
            self.format_world_config(config)?;
            self.newline()?;
        }

        // Format events
        self.format_events(&world.events)?;

        self.dedent();
        self.write("}")?;
        Ok(())
    }

    fn format_world_config(&mut self, config: &ConfigDef) -> Result<(), FormatterError> {
        self.write("config {")?;
        self.indent();
        self.newline()?;

        if config.tick_interval != Duration::from_secs(1) {
            self.write(&format!(
                "tick_interval: {}",
                config.tick_interval.as_secs()
            ))?;
            self.newline()?;
        }
        if config.max_agents != 1000 {
            self.write(&format!("max_agents: {}", config.max_agents))?;
            self.newline()?;
        }
        if config.event_buffer_size != 1000 {
            self.write(&format!("event_buffer_size: {}", config.event_buffer_size))?;
            self.newline()?;
        }

        self.dedent();
        self.write("}")?;
        Ok(())
    }

    fn format_events(&mut self, events: &EventsDef) -> Result<(), FormatterError> {
        if !events.events.is_empty() {
            self.write("events {")?;
            self.indent();
            self.newline()?;

            for event in &events.events {
                self.format_custom_event(event)?;
                self.newline()?;
            }

            self.dedent();
            self.write("}")?;
            self.newline()?;
        }
        Ok(())
    }

    fn format_custom_event(&mut self, event: &CustomEventDef) -> Result<(), FormatterError> {
        self.write(&event.name)?;
        self.write("(")?;
        for (i, param) in event.parameters.iter().enumerate() {
            if i > 0 {
                self.write(", ")?;
            }
            self.format_parameter(param)?;
        }
        self.write(")")?;
        Ok(())
    }

    fn format_parameter(&mut self, param: &Parameter) -> Result<(), FormatterError> {
        self.write(&param.name)?;
        self.write(": ")?;
        self.format_type_info(&param.type_info)?;
        Ok(())
    }

    fn format_expression(&mut self, expr: &Expression) -> Result<(), FormatterError> {
        match expr {
            Expression::Literal(lit) => self.format_literal(lit)?,
            Expression::Variable(name) => self.write(name)?,
            Expression::StateAccess(path) => self.write(&path.0.join("."))?,
            Expression::FunctionCall {
                function,
                arguments,
            } => {
                self.write(function)?;
                self.write("(")?;
                for (i, arg) in arguments.iter().enumerate() {
                    if i > 0 {
                        self.write(", ")?;
                    }
                    self.format_expression(arg)?;
                }
                self.write(")")?;
            }
            Expression::Think { args, with_block } => {
                self.write("think(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.write(", ")?;
                    }
                    self.format_argument(arg)?;
                }
                self.write(")")?;
                if let Some(attrs) = with_block {
                    self.write(" with ")?;
                    self.format_think_attributes(attrs)?;
                }
            }
            Expression::Request {
                agent,
                request_type,
                parameters,
                options,
            } => {
                self.write(&format!("{}.{}", agent, request_type))?;
                self.write("(")?;
                for (i, param) in parameters.iter().enumerate() {
                    if i > 0 {
                        self.write(", ")?;
                    }
                    self.format_argument(param)?;
                }
                self.write(")")?;
                if let Some(attrs) = options {
                    self.write(" with ")?;
                    self.format_request_attributes(attrs)?;
                }
            }
            Expression::Await(exprs) => {
                self.write("await ")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        self.write(", ")?;
                    }
                    self.format_expression(expr)?;
                }
            }
            Expression::BinaryOp { op, left, right } => {
                self.format_expression(left)?;
                self.write(" ")?;
                self.write(match op {
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
                })?;
                self.write(" ")?;
                self.format_expression(right)?;
            }
            Expression::Ok(expr) => {
                self.write("Ok(")?;
                self.format_expression(expr)?;
                self.write(")")?;
            }
            Expression::Err(expr) => {
                self.write("Err(")?;
                self.format_expression(expr)?;
                self.write(")")?;
            }
        }
        Ok(())
    }

    fn format_literal(&mut self, lit: &Literal) -> Result<(), FormatterError> {
        match lit {
            Literal::Integer(i) => self.write(&i.to_string())?,
            Literal::Float(f) => self.write(&f.to_string())?,
            Literal::String(s) => self.write(&format!("\"{}\"", s))?,
            Literal::Boolean(b) => self.write(if *b { "true" } else { "false" })?,
            Literal::Duration(d) => self.write(&format!("{}s", d.as_secs()))?,
            Literal::List(items) => {
                self.write("[")?;
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        self.write(", ")?;
                    }
                    self.format_literal(item)?;
                }
                self.write("]")?;
            }
            Literal::Map(items) => {
                self.write("{")?;
                for (i, (key, value)) in items.iter().enumerate() {
                    if i > 0 {
                        self.write(", ")?;
                    }
                    self.write(key)?;
                    self.write(": ")?;
                    self.format_literal(value)?;
                }
                self.write("}")?;
            }
            Literal::Retry(config) => {
                self.write("retry(")?;
                self.write(&format!("max_attempts: {}", config.max_attempts))?;
                match &config.delay {
                    RetryDelay::Fixed(ms) => {
                        self.write(&format!(", delay: {}ms", ms))?;
                    }
                    RetryDelay::Exponential { initial, max } => {
                        self.write(&format!(", delay: exponential({}ms, {}ms)", initial, max))?;
                    }
                }
                self.write(")")?;
            }
            Literal::Null => self.write("null")?,
        }
        Ok(())
    }

    fn format_argument(&mut self, arg: &Argument) -> Result<(), FormatterError> {
        match arg {
            Argument::Named { name, value } => {
                self.write(name)?;
                self.write(": ")?;
                self.format_expression(value)?;
            }
            Argument::Positional(expr) => {
                self.format_expression(expr)?;
            }
        }
        Ok(())
    }

    fn format_think_attributes(&mut self, attrs: &ThinkAttributes) -> Result<(), FormatterError> {
        self.write("{")?;
        self.indent();
        self.newline()?;

        if let Some(provider) = &attrs.provider {
            self.write(&format!("provider: \"{}\"", provider))?;
            self.newline()?;
        }
        if let Some(gen_type) = &attrs.prompt_generator_type {
            self.write("generator: ")?;
            match gen_type {
                PromptGeneratorType::Standard => self.write("standard")?,
            }
            self.newline()?;
        }
        if !attrs.policies.is_empty() {
            self.write("policies: [")?;
            for (i, policy) in attrs.policies.iter().enumerate() {
                if i > 0 {
                    self.write(", ")?;
                }
                self.write(&format!("\"{}\"", policy.text))?;
            }
            self.write("]")?;
            self.newline()?;
        }
        if let Some(model) = &attrs.model {
            self.write(&format!("model: \"{}\"", model))?;
            self.newline()?;
        }
        if let Some(temp) = attrs.temperature {
            self.write(&format!("temperature: {}", temp))?;
            self.newline()?;
        }
        if let Some(tokens) = attrs.max_tokens {
            self.write(&format!("max_tokens: {}", tokens))?;
            self.newline()?;
        }
        if let Some(retry) = &attrs.retry {
            self.write("retry: ")?;
            self.format_literal(&Literal::Retry(retry.clone()))?;
            self.newline()?;
        }
        if !attrs.plugins.is_empty() {
            self.write("plugins: {")?;
            self.indent();
            self.newline()?;
            for (name, config) in &attrs.plugins {
                self.write(&format!("{}: {{", name))?;
                self.indent();
                self.newline()?;
                for (key, value) in config {
                    self.write(key)?;
                    self.write(": ")?;
                    self.format_literal(value)?;
                    self.newline()?;
                }
                self.dedent();
                self.write("}")?;
                self.newline()?;
            }
            self.dedent();
            self.write("}")?;
            self.newline()?;
        }

        self.dedent();
        self.write("}")?;
        Ok(())
    }

    fn format_request_attributes(
        &mut self,
        attrs: &RequestAttributes,
    ) -> Result<(), FormatterError> {
        self.write("{")?;
        self.indent();
        self.newline()?;

        if let Some(timeout) = attrs.timeout {
            self.write(&format!("timeout: {}s", timeout.as_secs()))?;
            self.newline()?;
        }
        if let Some(retry) = attrs.retry {
            self.write(&format!("retry: {}", retry))?;
            self.newline()?;
        }

        self.dedent();
        self.write("}")?;
        Ok(())
    }

    fn format_type_info(&mut self, type_info: &TypeInfo) -> Result<(), FormatterError> {
        match type_info {
            TypeInfo::Simple(name) => self.write(name)?,
            TypeInfo::Result { ok_type, err_type } => {
                self.write("Result{")?;
                self.format_type_info(ok_type)?;
                self.write(", ")?;
                self.format_type_info(err_type)?;
                self.write("}")?;
            }
            TypeInfo::Option(inner) => {
                self.write("Option{")?;
                self.format_type_info(inner)?;
                self.write("}")?;
            }
            TypeInfo::Array(inner) => {
                self.write("Array{")?;
                self.format_type_info(inner)?;
                self.write("}")?;
            }
            TypeInfo::Map(key, value) => {
                self.write("Map{")?;
                self.format_type_info(key)?;
                self.write(", ")?;
                self.format_type_info(value)?;
                self.write("}")?;
            }
            TypeInfo::Custom { name, fields } => {
                self.write(name)?;
                if !fields.is_empty() {
                    self.write(" {")?;
                    self.indent();
                    self.newline()?;

                    for (name, field) in fields {
                        self.write(name)?;
                        if let Some(type_info) = &field.type_info {
                            self.write(": ")?;
                            self.format_type_info(type_info)?;
                        }
                        if let Some(default) = &field.default_value {
                            self.write(" = ")?;
                            self.format_expression(default)?;
                        }
                        self.newline()?;
                    }

                    self.dedent();
                    self.write("}")?;
                }
            }
        }
        Ok(())
    }

    fn format_lifecycle(&mut self, lifecycle: &LifecycleDef) -> Result<(), FormatterError> {
        self.write("lifecycle {")?;
        self.indent();
        self.newline()?;

        if let Some(on_init) = &lifecycle.on_init {
            self.write("on_init ")?;
            self.format_handler_block(on_init)?;
            self.newline()?;
        }

        if let Some(on_destroy) = &lifecycle.on_destroy {
            self.write("on_destroy ")?;
            self.format_handler_block(on_destroy)?;
            self.newline()?;
        }

        self.dedent();
        self.write("}")?;
        Ok(())
    }

    fn format_state(&mut self, state: &StateDef) -> Result<(), FormatterError> {
        self.write("state {")?;
        self.indent();
        self.newline()?;

        let mut first = true;
        for (name, var) in &state.variables {
            if !first {
                self.write(",")?;
                self.newline()?;
            }
            first = false;

            self.write(name)?;
            self.write(": ")?;
            self.format_type_info(&var.type_info)?;
            if let Some(initial_value) = &var.initial_value {
                self.write(" = ")?;
                self.format_expression(initial_value)?;
            }
        }

        self.newline()?;
        self.dedent();
        self.write("}")?;
        Ok(())
    }

    fn format_observe(&mut self, observe: &ObserveDef) -> Result<(), FormatterError> {
        self.write("observe {")?;
        self.indent();
        self.newline()?;

        for handler in &observe.handlers {
            self.format_event_handler(handler)?;
            self.newline()?;
        }

        self.dedent();
        self.write("}")?;
        Ok(())
    }

    fn format_answer(&mut self, answer: &AnswerDef) -> Result<(), FormatterError> {
        self.write("answer {")?;
        self.indent();
        self.newline()?;

        for handler in &answer.handlers {
            self.format_request_handler(handler)?;
            self.newline()?;
        }

        self.dedent();
        self.write("}")?;
        Ok(())
    }

    fn format_react(&mut self, react: &ReactDef) -> Result<(), FormatterError> {
        self.write("react {")?;
        self.indent();
        self.newline()?;

        for handler in &react.handlers {
            self.format_event_handler(handler)?;
            self.newline()?;
        }

        self.dedent();
        self.write("}")?;
        Ok(())
    }

    fn format_event_handler(&mut self, handler: &EventHandler) -> Result<(), FormatterError> {
        self.write("on ")?;
        match &handler.event_type {
            EventType::Tick => self.write("tick")?,
            EventType::StateUpdated {
                agent_name,
                state_name,
            } => {
                self.write(&format!("state_updated({}.{})", agent_name, state_name))?;
            }
            EventType::Message { content_type } => self.write(content_type)?,
            EventType::Custom(name) => self.write(name)?,
        }
        self.write("(")?;

        for (i, param) in handler.parameters.iter().enumerate() {
            if i > 0 {
                self.write(", ")?;
            }
            self.format_parameter(param)?;
        }
        self.write(") ")?;

        self.format_handler_block(&handler.block)?;
        Ok(())
    }

    fn format_request_handler(&mut self, handler: &RequestHandler) -> Result<(), FormatterError> {
        self.write("on request ")?;
        match &handler.request_type {
            RequestType::Query { query_type } => self.write(query_type)?,
            RequestType::Action { action_type } => self.write(action_type)?,
            RequestType::Custom(name) => self.write(name)?,
        }
        self.write("(")?;

        for (i, param) in handler.parameters.iter().enumerate() {
            if i > 0 {
                self.write(", ")?;
            }
            self.format_parameter(param)?;
        }
        self.write(") -> ")?;
        self.format_type_info(&handler.return_type)?;

        if let Some(constraints) = &handler.constraints {
            self.write(" with {")?;
            self.indent();
            self.newline()?;

            if let Some(strictness) = constraints.strictness {
                self.write(&format!("strictness: {}", strictness))?;
                self.newline()?;
            }
            if let Some(stability) = constraints.stability {
                self.write(&format!("stability: {}", stability))?;
                self.newline()?;
            }
            if let Some(latency) = constraints.latency {
                self.write(&format!("latency: {}", latency))?;
                self.newline()?;
            }

            self.dedent();
            self.write("} ")?;
        }

        self.format_handler_block(&handler.block)?;
        Ok(())
    }

    fn format_handler_block(&mut self, block: &HandlerBlock) -> Result<(), FormatterError> {
        self.write("{")?;
        self.indent();
        self.newline()?;

        for stmt in &block.statements {
            self.format_statement(stmt)?;
            self.newline()?;
        }

        self.dedent();
        self.write("}")?;
        Ok(())
    }

    fn format_statement(&mut self, stmt: &Statement) -> Result<(), FormatterError> {
        match stmt {
            Statement::Expression(expr) => self.format_expression(expr)?,
            Statement::Assignment { target, value } => {
                for (i, expr) in target.iter().enumerate() {
                    if i > 0 {
                        self.write(", ")?;
                    }
                    self.format_expression(expr)?;
                }
                self.write(" = ")?;
                self.format_expression(value)?;
            }
            Statement::Return(expr) => {
                self.write("return ")?;
                self.format_expression(expr)?;
            }
            Statement::Emit {
                event_type,
                parameters,
                target,
            } => {
                self.write("emit ")?;
                if let Some(t) = target {
                    self.write(&format!("to {} ", t))?;
                }
                self.write(&event_type.to_string())?;
                self.write("(")?;
                for (i, arg) in parameters.iter().enumerate() {
                    if i > 0 {
                        self.write(", ")?;
                    }
                    self.format_argument(arg)?;
                }
                self.write(")")?;
            }
            Statement::Block(statements) => {
                self.write("{")?;
                self.indent();
                self.newline()?;
                for stmt in statements {
                    self.format_statement(stmt)?;
                    self.newline()?;
                }
                self.dedent();
                self.write("}")?;
            }
            Statement::WithError {
                statement,
                error_handler_block,
            } => {
                self.format_statement(statement)?;
                self.write(" on_fail ")?;
                if let Some(binding) = &error_handler_block.error_binding {
                    self.write(&format!("bind {} ", binding))?;
                }
                self.write("{")?;
                self.indent();
                self.newline()?;
                for stmt in &error_handler_block.error_handler_statements {
                    self.format_statement(stmt)?;
                    self.newline()?;
                }
                self.dedent();
                self.write("}")?;
            }
            Statement::If {
                condition,
                then_block,
                else_block,
            } => {
                self.write("if ")?;
                self.format_expression(condition)?;
                self.write(" {")?;
                self.indent();
                self.newline()?;

                for stmt in then_block {
                    self.format_statement(stmt)?;
                    self.newline()?;
                }

                self.dedent();
                self.write("}")?;

                if let Some(else_stmts) = else_block {
                    self.write(" else {")?;
                    self.indent();
                    self.newline()?;

                    for stmt in else_stmts {
                        self.format_statement(stmt)?;
                        self.newline()?;
                    }

                    self.dedent();
                    self.write("}")?;
                }
            }
        }
        Ok(())
    }

    fn format_micro_agent(&mut self, agent: &MicroAgentDef) -> Result<(), FormatterError> {
        self.write("micro ")?;
        self.write(&agent.name)?;
        self.write(" {")?;
        self.indent();
        self.newline()?;

        // Format policies
        for policy in &agent.policies {
            self.write("policy ")?;
            self.write(&format!("\"{}\"", policy.text))?;
            self.newline()?;
        }

        // Add newline after policies if there are other components
        if agent.state.is_some()
            || agent.lifecycle.is_some()
            || agent.observe.is_some()
            || agent.answer.is_some()
            || agent.react.is_some()
        {
            self.newline()?;
        }

        // Format lifecycle if present
        if let Some(lifecycle) = &agent.lifecycle {
            self.format_lifecycle(lifecycle)?;
            self.newline()?;
        }

        // Format state if present
        if let Some(state) = &agent.state {
            self.format_state(state)?;
            self.newline()?;
        }

        // Format observe if present
        if let Some(observe) = &agent.observe {
            self.format_observe(observe)?;
            self.newline()?;
        }

        // Format answer if present
        if let Some(answer) = &agent.answer {
            self.format_answer(answer)?;
            self.newline()?;
        }

        // Format react if present
        if let Some(react) = &agent.react {
            self.format_react(react)?;
            self.newline()?;
        }

        self.dedent();
        self.write("}")?;
        Ok(())
    }

    fn write(&mut self, text: &str) -> Result<(), FormatterError> {
        self.output.push_str(text);
        Ok(())
    }

    fn indent(&mut self) {
        self.indent_level += 1;
    }

    fn dedent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }

    fn newline(&mut self) -> Result<(), FormatterError> {
        self.output.push('\n');
        self.write(&" ".repeat(self.indent_level * self.config.indent_spaces))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::ast::*;

    fn create_test_config() -> FormatterConfig {
        FormatterConfig {
            indent_spaces: 4,
            max_width: 80,
            operator_spacing: true,
            block_spacing: true,
        }
    }

    #[test]
    fn test_format_world() {
        let config = create_test_config();
        let mut visitor = FormatterVisitor::new(config);
        let world = WorldDef {
            name: "TestWorld".to_string(),
            policies: vec![Policy {
                text: "Test policy".to_string(),
                scope: PolicyScope::World("TestWorld".to_string()),
                internal_id: PolicyId::new(),
            }],
            config: None,
            events: Default::default(),
            handlers: Default::default(),
        };

        visitor.format_world(&world).unwrap();
        let output = visitor.output;
        assert!(output.contains("world TestWorld {"));
        assert!(output.contains("    policy \"Test policy\""));
        assert!(output.ends_with("}"));
    }

    #[test]
    fn test_format_micro_agent() {
        let mut visitor = FormatterVisitor::new(create_test_config());
        let agent = MicroAgentDef {
            name: "TravelPlanner".to_string(),
            policies: vec![Policy {
                text: "Create balanced itineraries with appropriate time allocation".to_string(),
                scope: PolicyScope::Agent("TravelPlanner".to_string()),
                internal_id: PolicyId::new(),
            }],
            state: Some(StateDef {
                variables: {
                    let mut map = HashMap::new();
                    map.insert(
                        "current_plan".to_string(),
                        StateVarDef {
                            name: "current_plan".to_string(),
                            type_info: TypeInfo::Simple("String".to_string()),
                            initial_value: Some(Expression::Literal(Literal::String(
                                "none".to_string(),
                            ))),
                        },
                    );
                    map.insert(
                        "planning_stage".to_string(),
                        StateVarDef {
                            name: "planning_stage".to_string(),
                            type_info: TypeInfo::Simple("String".to_string()),
                            initial_value: Some(Expression::Literal(Literal::String(
                                "none".to_string(),
                            ))),
                        },
                    );
                    map
                },
            }),
            lifecycle: None,
            observe: None,
            answer: Some(AnswerDef {
                handlers: vec![RequestHandler {
                    request_type: RequestType::Custom("PlanTrip".to_string()),
                    parameters: vec![
                        Parameter {
                            name: "destination".to_string(),
                            type_info: TypeInfo::Simple("String".to_string()),
                        },
                        Parameter {
                            name: "start".to_string(),
                            type_info: TypeInfo::Simple("String".to_string()),
                        },
                        Parameter {
                            name: "end".to_string(),
                            type_info: TypeInfo::Simple("String".to_string()),
                        },
                        Parameter {
                            name: "budget".to_string(),
                            type_info: TypeInfo::Simple("Float".to_string()),
                        },
                        Parameter {
                            name: "interests".to_string(),
                            type_info: TypeInfo::Simple("String".to_string()),
                        },
                    ],
                    return_type: TypeInfo::Result {
                        ok_type: Box::new(TypeInfo::Simple("String".to_string())),
                        err_type: Box::new(TypeInfo::Simple("Error".to_string())),
                    },
                    constraints: None,
                    block: HandlerBlock { statements: vec![] },
                }],
            }),
            react: None,
        };

        visitor.format_micro_agent(&agent).unwrap();
        let output = visitor.output;
        assert!(output.contains("micro TravelPlanner {"));
        assert!(output.contains("    policy \"Create balanced itineraries"));
        assert!(output.contains("state {"));
        assert!(output.contains("current_plan: String = \"none\""));
        assert!(output.contains("planning_stage: String = \"none\""));
        assert!(output.contains("answer {"));
        assert!(output.contains("on request PlanTrip("));
        assert!(output.contains("destination: String"));
        assert!(output.contains("-> Result{String, Error}"));
    }

    #[test]
    fn test_format_lifecycle() {
        let mut visitor = FormatterVisitor::new(create_test_config());
        let lifecycle = LifecycleDef {
            on_init: Some(HandlerBlock {
                statements: vec![Statement::Expression(Expression::FunctionCall {
                    function: "initialize".to_string(),
                    arguments: vec![],
                })],
            }),
            on_destroy: Some(HandlerBlock {
                statements: vec![Statement::Expression(Expression::FunctionCall {
                    function: "cleanup".to_string(),
                    arguments: vec![],
                })],
            }),
        };

        visitor.format_lifecycle(&lifecycle).unwrap();
        let output = visitor.output;
        assert!(output.contains("lifecycle {"));
        assert!(output.contains("on_init {"));
        assert!(output.contains("initialize()"));
        assert!(output.contains("on_destroy {"));
        assert!(output.contains("cleanup()"));
    }

    #[test]
    fn test_format_state() {
        let mut visitor = FormatterVisitor::new(create_test_config());
        let state = StateDef {
            variables: {
                let mut map = HashMap::new();
                map.insert(
                    "counter".to_string(),
                    StateVarDef {
                        name: "counter".to_string(),
                        type_info: TypeInfo::Simple("Int".to_string()),
                        initial_value: Some(Expression::Literal(Literal::Integer(0))),
                    },
                );
                map.insert(
                    "name".to_string(),
                    StateVarDef {
                        name: "name".to_string(),
                        type_info: TypeInfo::Simple("String".to_string()),
                        initial_value: None,
                    },
                );
                map
            },
        };

        visitor.format_state(&state).unwrap();
        let output = visitor.output;
        assert!(output.contains("state {"));
        assert!(output.contains("counter: Int = 0"));
        assert!(output.contains("name: String"));
    }

    #[test]
    fn test_format_observe() {
        let mut visitor = FormatterVisitor::new(create_test_config());
        let observe = ObserveDef {
            handlers: vec![EventHandler {
                event_type: EventType::Tick,
                parameters: vec![],
                block: HandlerBlock {
                    statements: vec![Statement::Expression(Expression::FunctionCall {
                        function: "update".to_string(),
                        arguments: vec![],
                    })],
                },
            }],
        };

        visitor.format_observe(&observe).unwrap();
        let output = visitor.output;
        assert!(output.contains("observe {"));
        assert!(output.contains("on tick()"));
        assert!(output.contains("update()"));
    }

    #[test]
    fn test_format_react() {
        let mut visitor = FormatterVisitor::new(create_test_config());
        let react = ReactDef {
            handlers: vec![EventHandler {
                event_type: EventType::StateUpdated {
                    agent_name: "other".to_string(),
                    state_name: "status".to_string(),
                },
                parameters: vec![],
                block: HandlerBlock {
                    statements: vec![Statement::Expression(Expression::FunctionCall {
                        function: "react".to_string(),
                        arguments: vec![],
                    })],
                },
            }],
        };

        visitor.format_react(&react).unwrap();
        let output = visitor.output;
        assert!(output.contains("react {"));
        assert!(output.contains("on state_updated(other.status)"));
        assert!(output.contains("react()"));
    }

    #[test]
    fn test_format_root() {
        let config = create_test_config();
        let mut visitor = FormatterVisitor::new(config);
        let root = Root::new(
            None,
            vec![MicroAgentDef {
                name: "TestAgent".to_string(),
                policies: vec![],
                lifecycle: None,
                state: None,
                observe: None,
                answer: None,
                react: None,
            }],
        );

        let output = visitor.format_root(&root).unwrap();
        assert!(output.contains("micro TestAgent {"));
        assert!(output.ends_with("}\n"));
    }

    #[test]
    fn test_indentation() {
        let config = FormatterConfig {
            indent_spaces: 2,
            max_width: 80,
            operator_spacing: true,
            block_spacing: true,
        };
        let mut visitor = FormatterVisitor::new(config);
        let world = WorldDef {
            name: "TestWorld".to_string(),
            policies: vec![Policy {
                text: "Test policy".to_string(),
                scope: PolicyScope::World("TestWorld".to_string()),
                internal_id: PolicyId::new(),
            }],
            config: None,
            events: Default::default(),
            handlers: Default::default(),
        };

        visitor.format_world(&world).unwrap();
        let output = visitor.output;
        assert!(output.contains("\n  policy")); // Check 2-space indentation
    }

    #[test]
    fn test_format_world_config() {
        let mut visitor = FormatterVisitor::new(create_test_config());
        let config = ConfigDef {
            tick_interval: Duration::from_secs(2),
            max_agents: 500,
            event_buffer_size: 2000,
        };

        visitor.format_world_config(&config).unwrap();
        let output = visitor.output;
        assert!(output.contains("config {"));
        assert!(output.contains("tick_interval: 2"));
        assert!(output.contains("max_agents: 500"));
        assert!(output.contains("event_buffer_size: 2000"));
        assert!(output.ends_with("}"));
    }

    #[test]
    fn test_format_events() {
        let mut visitor = FormatterVisitor::new(create_test_config());
        let events = EventsDef {
            events: vec![CustomEventDef {
                name: "TestEvent".to_string(),
                parameters: vec![Parameter {
                    name: "param1".to_string(),
                    type_info: TypeInfo::Simple("String".to_string()),
                }],
            }],
        };

        visitor.format_events(&events).unwrap();
        let output = visitor.output;
        assert!(output.contains("events {"));
        assert!(output.contains("TestEvent(param1: String)"));
        assert!(output.ends_with("}\n"));
    }

    #[test]
    fn test_format_type_info() {
        let mut visitor = FormatterVisitor::new(create_test_config());

        // Test simple type
        visitor
            .format_type_info(&TypeInfo::Simple("String".to_string()))
            .unwrap();
        assert_eq!(visitor.output, "String");
        visitor.output.clear();

        // Test Result type
        let result_type = TypeInfo::Result {
            ok_type: Box::new(TypeInfo::Simple("String".to_string())),
            err_type: Box::new(TypeInfo::Simple("Error".to_string())),
        };
        visitor.format_type_info(&result_type).unwrap();
        assert_eq!(visitor.output, "Result{String, Error}");
        visitor.output.clear();

        // Test Option type
        let option_type = TypeInfo::Option(Box::new(TypeInfo::Simple("Int".to_string())));
        visitor.format_type_info(&option_type).unwrap();
        assert_eq!(visitor.output, "Option{Int}");
        visitor.output.clear();

        // Test Custom type
        let mut fields = HashMap::new();
        fields.insert(
            "name".to_string(),
            FieldInfo {
                type_info: Some(TypeInfo::Simple("String".to_string())),
                default_value: Some(Expression::Literal(Literal::String("test".to_string()))),
            },
        );
        let custom_type = TypeInfo::Custom {
            name: "Person".to_string(),
            fields,
        };
        visitor.format_type_info(&custom_type).unwrap();
        let output = visitor.output;
        assert!(output.contains("Person {"));
        assert!(output.contains("name: String = \"test\""));
        assert!(output.ends_with("}"));
    }

    #[test]
    fn test_format_expression() {
        let mut visitor = FormatterVisitor::new(create_test_config());

        // Test literal
        let literal_expr = Expression::Literal(Literal::Integer(42));
        visitor.format_expression(&literal_expr).unwrap();
        assert_eq!(visitor.output, "42");
        visitor.output.clear();

        // Test variable
        let var_expr = Expression::Variable("x".to_string());
        visitor.format_expression(&var_expr).unwrap();
        assert_eq!(visitor.output, "x");
        visitor.output.clear();

        // Test function call
        let func_expr = Expression::FunctionCall {
            function: "test".to_string(),
            arguments: vec![Expression::Literal(Literal::String("hello".to_string()))],
        };
        visitor.format_expression(&func_expr).unwrap();
        assert_eq!(visitor.output, "test(\"hello\")");
        visitor.output.clear();

        // Test binary op
        let binary_expr = Expression::BinaryOp {
            op: BinaryOperator::Add,
            left: Box::new(Expression::Literal(Literal::Integer(1))),
            right: Box::new(Expression::Literal(Literal::Integer(2))),
        };
        visitor.format_expression(&binary_expr).unwrap();
        assert_eq!(visitor.output, "1 + 2");
    }

    #[test]
    fn test_format_literal() {
        let mut visitor = FormatterVisitor::new(create_test_config());

        // Test integer
        visitor.format_literal(&Literal::Integer(42)).unwrap();
        assert_eq!(visitor.output, "42");
        visitor.output.clear();

        // Test string
        visitor
            .format_literal(&Literal::String("test".to_string()))
            .unwrap();
        assert_eq!(visitor.output, "\"test\"");
        visitor.output.clear();

        // Test list
        visitor
            .format_literal(&Literal::List(vec![
                Literal::Integer(1),
                Literal::Integer(2),
            ]))
            .unwrap();
        assert_eq!(visitor.output, "[1, 2]");
        visitor.output.clear();

        // Test map
        let mut map = HashMap::new();
        map.insert("key".to_string(), Literal::String("value".to_string()));
        visitor.format_literal(&Literal::Map(map)).unwrap();
        assert_eq!(visitor.output, "{key: \"value\"}");
    }
}
