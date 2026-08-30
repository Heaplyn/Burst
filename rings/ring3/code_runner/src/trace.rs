//! Execution trace: recorded events plus a snapshot of the program's layers.

use ast::Layer;

use crate::value::Value;

/// A single observable thing that happened during execution.
#[derive(Debug, Clone, PartialEq)]
pub enum TraceEvent {
    Havoc { Target: String },
    Read { Variable: String, Value: Value },
    Write { Variable: String, Old: Value, New: Value },
    FunctionCall { Name: String, Args: Vec<Value>, Result: Value },
    Interrupt { Syscall: String },
}

/// The running trace: recorded events, and the top-level layers so calls can be resolved by name.
#[derive(Debug, Clone, PartialEq)]
pub struct LayerTrace {
    Events: Vec<TraceEvent>,
    /// Snapshot of the top-level layers; read by the runner to resolve function names.
    pub(crate) Layers: Vec<Layer>,
}

impl LayerTrace {
    /// Builds a trace seeded with the given top-level layers.
    pub fn NewFrom(Layers: &[Layer]) -> Self {
        Self {
            Events: Vec::new(),
            Layers: Layers.to_vec(),
        }
    }

    /// An empty trace with no layers.
    pub fn New() -> Self {
        Self {
            Events: Vec::new(),
            Layers: Vec::new(),
        }
    }

    pub fn RecordEvent(&mut self, Event: TraceEvent) {
        self.Events.push(Event);
    }
}
