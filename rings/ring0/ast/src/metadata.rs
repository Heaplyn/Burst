//! Ring 0 · AST · **Metadata, Directives, Optimization Hints**
//!
//! Anything that decorates a layer without being part of its "shape":
//! docs values, `@inline`-style directives, optimizer hints, and the
//! observability flags used by trace folding.
//!
//! Requires: [`Expression`](crate::expr::Expression) (only inside
//! `ObservableValue::MemoryAddress`).

use std::collections::HashMap;

use crate::expr::Expression;

/// Values for metadata like doc comments.
#[derive(Debug, Clone, PartialEq)]
pub enum MetadataValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    List(Vec<MetadataValue>),
    Map(HashMap<String, MetadataValue>),
    Null,
}

/// Compiler directives like `#[inline]`.
#[derive(Debug, Clone, PartialEq)]
pub enum Directive {
    Inline,
    NoInline,
    Align(usize),
    Section(String),
    Cold,
    Hot,
    Unsafe,
    Extern,
}

/// Legacy compile-status enum (kept for downstream code; distinct from `CompilerResult<T>`).
#[derive(Debug, Clone, PartialEq)]
pub enum CompilerResult {
    Error(usize), // ErrorCode
    Success,
}

/// Hints for the optimizer to go fast.
#[derive(Debug, Clone, PartialEq)]
pub struct OptimizationHints {
    pub AggressiveLoopFolding: bool,
    pub TraceObservability: ObservabilityMode,
    pub RegisterPressure: RegisterPressureMode,
    pub InlineThreshold: Option<usize>,
}

/// How much the compiler should care about traces.
#[derive(Debug, Clone, PartialEq)]
pub enum ObservabilityMode {
    Strict,
    Relaxed,
    Aggressive,
}

/// How many registers the compiler can use.
#[derive(Debug, Clone, PartialEq)]
pub enum RegisterPressureMode {
    Low,
    High,
    Auto,
}

/// Flags for the observability boundary.
#[derive(Debug, Clone, PartialEq)]
pub struct ObservabilityFlags {
    pub ObservableValues: Vec<ObservableValue>,
    pub AffectsOutput: bool,
    pub AffectsHardware: bool,
    pub ObservableToTrace: bool,
}

/// Values that we can actually see outside the program.
#[derive(Debug, Clone, PartialEq)]
pub enum ObservableValue {
    Register(String),
    MemoryAddress(Expression),
    ReturnValue,
    SideEffect,
    Variable(String),
}
