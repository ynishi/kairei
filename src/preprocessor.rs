use crate::tokenizer::token::{TokenSpan, Tokenizer};
use regex::Regex;

/// A trait for preprocessing different types of input
pub trait Preprocessor<T> {
    /// Process the input of type T and return the processed result
    fn process(&self, input: T) -> T;
}

/// Token-specific preprocessor implementation
pub struct TokenPreprocessor {
    tokenizer: Tokenizer,
}

impl Default for TokenPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenPreprocessor {
    pub fn new() -> Self {
        Self {
            tokenizer: Tokenizer::new(),
        }
    }
}

impl Preprocessor<Vec<TokenSpan>> for TokenPreprocessor {
    fn process(&self, input: Vec<TokenSpan>) -> Vec<TokenSpan> {
        // Filter out comments and normalize whitespace
        input
            .into_iter()
            .filter(|span| !span.token.is_comment())
            .collect()
    }
}

/// String-specific preprocessor implementation
pub struct StringPreprocessor {
    re_block_comment: Regex,
    re_line_comment: Regex,
    re_empty_lines: Regex,
}

impl Default for StringPreprocessor {
    fn default() -> Self {
        Self::new()
    }
}

impl StringPreprocessor {
    pub fn new() -> Self {
        Self {
            re_block_comment: Regex::new(r"/\*(?:[^*]|\*[^/])*\*/").unwrap(),
            re_line_comment: Regex::new(r"//[^\n]*").unwrap(),
            re_empty_lines: Regex::new(r"\n\s*\n").unwrap(),
        }
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

impl Preprocessor<&str> for StringPreprocessor {
    fn process(&self, input: &str) -> String {
        let mut output = input.to_string();

        // コメントの除去
        output = self.remove_comments(&output);

        // 空白行の正規化
        output = self.normalize_empty_lines(&output);

        // 行末の空白を除去
        output = self.trim_lines(&output);

        output
    }
}
