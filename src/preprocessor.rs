use regex::Regex;

pub struct Preprocessor {
    re_block_comment: Regex,
    re_line_comment: Regex,
    re_empty_lines: Regex,
}

impl Default for Preprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Preprocessor {
    pub fn new() -> Self {
        Self {
            re_block_comment: Regex::new(r"/\*(?:[^*]|\*[^/])*\*/").unwrap(),
            re_line_comment: Regex::new(r"//[^\n]*").unwrap(),
            re_empty_lines: Regex::new(r"\n\s*\n").unwrap(),
        }
    }

    pub fn process(&self, input: &str) -> String {
        let mut output = input.to_string();

        // コメントの除去
        output = self.remove_comments(&output);

        // 空白行の正規化
        output = self.normalize_empty_lines(&output);

        // 行末の空白を除去
        output = self.trim_lines(&output);

        output
    }

    fn remove_comments(&self, input: &str) -> String {
        let without_block = self.re_block_comment.replace_all(input, "");

        self.re_line_comment
            .replace_all(&without_block, "")
            .to_string()
    }

    fn normalize_empty_lines(&self, input: &str) -> String {
        self.re_empty_lines.replace_all(input, "\n").to_string()
    }

    fn trim_lines(&self, input: &str) -> String {
        input
            .lines()
            .map(|line| line.trim_end())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
