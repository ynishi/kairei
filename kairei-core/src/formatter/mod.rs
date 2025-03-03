pub mod config;
pub mod error;
pub mod visitor;

use crate::ast::Root;
use config::FormatterConfig;
use error::FormatterError;
use visitor::FormatterVisitor;

pub struct Formatter {
    config: FormatterConfig,
}

impl Formatter {
    pub fn new(config: FormatterConfig) -> Self {
        Self { config }
    }

    pub fn format(&self, ast: &Root) -> Result<String, FormatterError> {
        let mut visitor = FormatterVisitor::new(self.config.clone());
        visitor.format_root(ast)
    }
}
