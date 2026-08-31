//! Ring 0 · AST · **Source Location**
//!
//! File + line + column position, attached to every layer through its metadata.
//! Requires: nothing.

/// Tracking where code is in the files.
#[derive(Debug, Clone, PartialEq)]
pub struct SourceLocation {
    pub File: String,
    pub Line: usize,
    pub Column: usize,
}

impl SourceLocation {
    /// For code that the compiler just knows.
    pub fn Builtin() -> Self {
        Self {
            File: "<builtin>".to_string(),
            Line: 0,
            Column: 0,
        }
    }
}
