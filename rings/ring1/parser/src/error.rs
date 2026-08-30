//! Error and result types shared across the runner.

/// The crate-wide result alias: every fallible runner operation returns this.
pub type CompilerResult<T> = Result<T, CompilerError>;

/// Every category of failure the runner (and its neighbours) can produce.
#[derive(Debug, Clone, PartialEq)]
pub enum CompilerError {
    LexerError(String),
    ParserError(String),
    ElaborationError(String),
    TypeError(String),
    RuntimeError(String),
    InternalError(String),
}
