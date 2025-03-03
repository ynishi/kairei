use thiserror::Error;

#[derive(Error, Debug)]
pub enum FormatterError {
    #[error("Formatting error: {0}")]
    Format(String),
    #[error("Invalid token: {0}")]
    Token(String),
}
