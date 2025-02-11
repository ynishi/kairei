use thiserror::Error;

// パーサートレイト

pub trait Parser<I, O> {
    fn parse(&self, input: &[I], pos: usize) -> ParseResult<O>;
}

pub type ParseResult<O> = Result<(usize, O), ParseError>;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("Parse error: {message}")]
    ParseError {
        message: String,
        found: String,
        position: (usize, usize),
    },
    #[error("Unexpected EOF")]
    UnexpectedEOF,
    #[error("EOF")]
    EOF,
    #[error("Unexpected")]
    Unexpected,
    #[error("No alternative")]
    NoAlternative,
    // fail
    #[error("Fail: {0}")]
    Fail(String),
    #[error("PredicateError")]
    PredicateError,
    #[error("WithContext: {message}, {inner}")]
    WithContext {
        message: String,
        inner: Box<ParseError>,
    },
}
