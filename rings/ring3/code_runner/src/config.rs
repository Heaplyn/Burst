//! Runner configuration.

use elaboration::*;
/// Tunables for a run. Empty for now — a home for future flags
/// (optimization level, trace verbosity, etc.).
#[derive(PartialEq, Default,Clone,Debug)]
pub struct CompilerConfig {
    Context: ElaborationContext,
}

impl CompilerConfig {
    pub fn New() -> Self {
        Self { Context: ElaborationContext::New() }
    }
}
