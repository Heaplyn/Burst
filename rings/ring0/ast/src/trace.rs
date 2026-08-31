//! Ring 0 · AST · **Trace Info**
//!
//! The per-layer bookkeeping the `layertrace` runtime reads for introspection.
//! Requires: [`TypeStorage`](crate::storage::TypeStorage).

use crate::storage::TypeStorage;

/// Info for the layertrace runtime.
#[derive(Debug, Clone, PartialEq)]
pub struct TraceInfo {
    pub TraceId: String,
    pub Depth: usize,
    pub Context: TraceContext,
    pub TypeEnv: TypeStorage,
}

impl TraceInfo {
    pub fn default() -> Self {
        Self {
            TraceId: String::new(),
            Depth: 0,
            Context: TraceContext::Root,
            TypeEnv: TypeStorage::default(),
        }
    }
}

/// What the trace is actually inside of.
#[derive(Debug, Clone, PartialEq)]
pub enum TraceContext {
    Root,
    Function { Name: String },
    Loop { Iteration: usize },
    Block,
}
