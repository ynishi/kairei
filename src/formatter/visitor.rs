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
