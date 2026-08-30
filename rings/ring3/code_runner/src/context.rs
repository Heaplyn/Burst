//! Execution context: variable storage and the call-frame stack.

use std::collections::HashMap;

use ast::Type;

use parser::error::{CompilerError, CompilerResult};
use crate::value::Value;

/// Holds all live variables: globals plus a stack of per-call frames.
#[derive(Debug, Clone)]
pub struct ExecutionContext {
    Variables: HashMap<String, VariableEntry>,
    Stack: Vec<Frame>,
}



/// A stored variable and whether it may be reassigned.
#[derive(Debug, Clone)]
pub struct VariableEntry {
    Value: Value,
    IsMutable: bool,
}

impl ExecutionContext {
    pub fn New() -> Self {
        Self {
            Variables: HashMap::new(),
            Stack: Vec::new(),
        }
    }

    /// Opens a new call frame named after the function being entered.
    pub fn PushFrame(&mut self, Name: &str) {
        self.Stack.push(Frame {
            Name: Name.to_string(),
            Variables: HashMap::new(),
        });
    }

    /// Closes the innermost call frame.
    pub fn PopFrame(&mut self) {
        self.Stack.pop();
    }

    /// Writes into the innermost frame, or the globals if no frame is open.
    pub fn SetVariable(&mut self, Name: &str, Value: Value, IsMutable: bool) {
        if let Some(Frame) = self.Stack.last_mut() {
            Frame
                .Variables
                .insert(Name.to_string(), VariableEntry { Value, IsMutable });
        } else {
            self.Variables
                .insert(Name.to_string(), VariableEntry { Value, IsMutable });
        }
    }

    /// Declares a variable in the current scope, initialized to the default
    /// value for `Type_`. Used to bind function parameters into a fresh frame.
    pub fn DeclareVariable(&mut self, Name: &str, Type_: &Type) -> CompilerResult<()> {
        let Initial = match Type_ {
            Type::BitPrecise('i', _) => Value::Int(0),
            Type::BitPrecise('f', _) => Value::Float(0.0),
            Type::BitPrecise('b', _) => Value::Bool(false),
            Type::Unit => Value::Unit,
            _ => {
                return Err(CompilerError::TypeError(format!(
                    "Cannot declare variable '{}': unsupported type {:?}",
                    Name, Type_
                )))
            }
        };
        self.SetVariable(Name, Initial, false);
        Ok(())
    }

    /// Looks a name up: innermost frame first (LIFO), then globals.
    pub fn GetVariable(&self, Name: &str) -> Option<Value> {
        for Frame in self.Stack.iter().rev() {
            if let Some(Entry) = Frame.Variables.get(Name) {
                return Some(Entry.Value.clone());
            }
        }
        self.Variables.get(Name).map(|Entry| Entry.Value.clone())
    }

    /// Invalidate cached knowledge about `Target` (placeholder for `havoc`).
    pub fn Invalidate(&mut self, Target: &str) {
        // (Implementation depends on cache system)
    }

    /// Keep a value in cache across a `havoc` (placeholder).
    pub fn KeepCache(&mut self, Target: &str) {
        // (Implementation depends on cache system)
    }
}

/// One call frame: a name and its local variables.
#[derive(Debug, Clone)]
pub struct Frame {
    pub Name: String,
    pub Variables: HashMap<String, VariableEntry>,
}
