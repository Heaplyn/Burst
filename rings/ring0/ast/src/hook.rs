//! Ring 0 · AST · **Variable Hooks**
//!
//! Reactive logic attached to a variable binding (`on_change`, `on_read`, …).
//! Requires: [`LayerKind`](crate::LayerKind).

use crate::LayerKind;

/// Logic that runs when variables change.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableHook {
    pub Kind: HookKind,
    pub Body: Vec<LayerKind>,
}

impl VariableHook {
    pub fn New(Kind: HookKind, Body: Vec<LayerKind>) -> Self {
        Self { Kind, Body }
    }
}

/// The different kinds of variable behaviors.
#[derive(Debug, Clone, PartialEq)]
pub enum HookKind {
    OnChange,
    OnRead,
    OnAssign,
    OnDrop,
    OnError,
}
