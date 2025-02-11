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
}
