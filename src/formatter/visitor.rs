use crate::ast::{MicroAgentDef, Root, WorldDef, TypeInfo, Expression, Literal, BinaryOperator, RetryDelay, Argument, ThinkAttributes, PromptGeneratorType, RequestAttributes};
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
            self.write(&format!("tick_interval: {}", config.tick_interval.as_secs()))?;
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
            Expression::FunctionCall { function, arguments } => {
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
            Expression::Request { agent, request_type, parameters, options } => {
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

    fn format_request_attributes(&mut self, attrs: &RequestAttributes) -> Result<(), FormatterError> {
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
    use super::*;
    use crate::ast::{Policy, PolicyScope, PolicyId};

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
        let config = create_test_config();
        let mut visitor = FormatterVisitor::new(config);
        let agent = MicroAgentDef {
            name: "TestAgent".to_string(),
            policies: vec![Policy {
                text: "Agent policy".to_string(),
                scope: PolicyScope::Agent("TestAgent".to_string()),
                internal_id: PolicyId::new(),
            }],
            lifecycle: None,
            state: None,
            observe: None,
            answer: None,
            react: None,
        };

        visitor.format_micro_agent(&agent).unwrap();
        let output = visitor.output;
        assert!(output.contains("micro TestAgent {"));
        assert!(output.contains("    policy \"Agent policy\""));
        assert!(output.ends_with("}"));
    }

    #[test]
    fn test_format_root() {
        let config = create_test_config();
        let mut visitor = FormatterVisitor::new(config);
        let root = Root::new(None, vec![MicroAgentDef {
            name: "TestAgent".to_string(),
            policies: vec![],
            lifecycle: None,
            state: None,
            observe: None,
            answer: None,
            react: None,
        }]);

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
        assert!(output.contains("\n  policy"));  // Check 2-space indentation
    }
}
