use nom::error::ErrorKind;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KaireiError {
    #[error("Parse error: {message} at line {line}, column {column}, kind: {kind:?}")]
    Parse {
        message: String,
        line: usize,
        column: usize,
        kind: ErrorKind,
    },

    #[error("Type error: {0}")]
    Type(String),

    #[error("Validation error: {0}")]
    Validation(String),
}

pub type Result<T> = std::result::Result<T, KaireiError>;
