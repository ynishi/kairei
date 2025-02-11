#[derive(Debug, Clone)]
pub struct FormatterConfig {
    pub indent_spaces: usize,
    pub max_width: usize,
    pub operator_spacing: bool,
    pub block_spacing: bool,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            indent_spaces: 4,
            max_width: 80,
            operator_spacing: true,
            block_spacing: true,
        }
    }
}
