use crate::ast::{MicroAgentDef, Root, WorldDef};
use crate::formatter::config::FormatterConfig;
use crate::formatter::error::FormatterError;

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

        self.dedent();
        self.write("}")?;
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
            events: vec![
                CustomEventDef {
                    name: "TestEvent".to_string(),
                    parameters: vec![Parameter {
                        name: "param1".to_string(),
                        type_info: TypeInfo::Simple("String".to_string()),
                    }],
                },
            ],
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
        visitor.format_type_info(&TypeInfo::Simple("String".to_string())).unwrap();
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
        fields.insert("name".to_string(), FieldInfo {
            type_info: Some(TypeInfo::Simple("String".to_string())),
            default_value: Some(Expression::Literal(Literal::String("test".to_string()))),
        });
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
        visitor.format_literal(&Literal::String("test".to_string())).unwrap();
        assert_eq!(visitor.output, "\"test\"");
        visitor.output.clear();

        // Test list
        visitor.format_literal(&Literal::List(vec![
            Literal::Integer(1),
            Literal::Integer(2),
        ])).unwrap();
        assert_eq!(visitor.output, "[1, 2]");
        visitor.output.clear();

        // Test map
        let mut map = HashMap::new();
        map.insert("key".to_string(), Literal::String("value".to_string()));
        visitor.format_literal(&Literal::Map(map)).unwrap();
        assert_eq!(visitor.output, "{key: \"value\"}");
    }
}
