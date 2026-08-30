#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

//! The LayerScript tree-walking interpreter (Ring 3).
//!
//! Modules:
//! - [`error`]   — `CompilerError` / `CompilerResult`
//! - [`config`]  — `CompilerConfig`
//! - [`value`]   — runtime `Value` + `FormatValue`
//! - [`context`] — variable storage and call frames
//! - [`trace`]   — execution trace + layer snapshot
//! - [`builtins`]— built-in functions and their registry
//! - [`runner`]  — the `CodeRunner`, entry point, resolution & helpers
//! - [`eval`]    — expression evaluation (impl on `CodeRunner`)
//! - [`exec`]    — layer execution (impl on `CodeRunner`)

pub mod builtins;
pub mod config;
pub mod context;
pub mod runner;
pub mod trace;
pub mod value;

mod eval;
mod exec;

// Flat re-exports so downstream crates keep using `code_runner::{CodeRunner, ...}`.
pub use builtins::{AddBuiltins, BuiltinFn};
pub use config::CompilerConfig;
pub use context::{ExecutionContext, Frame, VariableEntry};
pub use parser::error::{CompilerError, CompilerResult};
pub use runner::CodeRunner;
pub use trace::{LayerTrace, TraceEvent};
pub use value::{FormatValue, Value};

// ============================================
// Tests
// ============================================

#[cfg(test)]
mod tests {
    use super::*;
    use ast::Expression;

    #[test]
    fn test_evaluate_literal_int() {
        let mut Runner = CodeRunner::New(CompilerConfig::default());
        let Expr = Expression::LiteralInt(42);
        let Result = Runner.EvaluateExpression(&Expr).unwrap();
        assert_eq!(Result, Value::Int(42));
    }

    #[test]
    fn test_evaluate_binary_add() {
        let mut Runner = CodeRunner::New(CompilerConfig::default());
        let Expr = Expression::BinaryOp {
            Op: "+".to_string(),
            Lhs: Box::new(Expression::LiteralInt(5)),
            Rhs: Box::new(Expression::LiteralInt(3)),
        };
        let Result = Runner.EvaluateExpression(&Expr).unwrap();
        assert_eq!(Result, Value::Int(8));
    }
}
