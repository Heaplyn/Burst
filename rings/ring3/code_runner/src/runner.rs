//! The `CodeRunner` itself: construction, top-level entry point, function
//! resolution, and small type/default helpers. Expression evaluation lives in
//! [`crate::eval`] and layer execution in [`crate::exec`].

use std::collections::HashMap;

use ast::{Layer, LayerKind, Type};

use crate::builtins::{AddBuiltins, BuiltinFn};
use crate::config::CompilerConfig;
use crate::context::ExecutionContext;
use parser::error::{CompilerError, CompilerResult};
use crate::trace::LayerTrace;
use crate::value::Value;

/// The interpreter: holds variable state, the execution trace, config, and the
/// built-in function registry.
#[derive(Debug, Clone)]
pub struct CodeRunner {
    pub(crate) Context: ExecutionContext,
    pub(crate) Trace: LayerTrace,
    pub(crate) Config: CompilerConfig,
    /// str function name -> built-in implementation.
    pub(crate) Builtins: HashMap<String, BuiltinFn>,
    pub(crate) ReturnValue: Option<Value>,
}

impl CodeRunner {
    pub fn New(Config: CompilerConfig) -> Self {
        let mut Builtins: HashMap<String, BuiltinFn> = HashMap::new();
        AddBuiltins(&mut Builtins);

        Self {
            Context: ExecutionContext::New(),
            Trace: LayerTrace::New(),
            Config,
            Builtins,
            ReturnValue: None,
        }
    }

    // ============================================
    // Code Entry Point
    // ============================================

    pub fn RunCode(&mut self, Layers: &[Layer]) -> CompilerResult<Value> {
        // The driver hands us the single `Program` layer wrapped in a slice, so
        // unwrap it to reach the real top-level items (functions, globals, calls).
        let TopLevel: &[Layer] = match Layers {
            [Program] if matches!(Program.Kind, LayerKind::Program) => &Program.Children,
            other => other,
        };

        // Snapshot the top-level items so FunctionCall can resolve names later.
        self.Trace = LayerTrace::NewFrom(TopLevel);

        // Run the top level like a script: execute every item in order, but skip
        // function *definitions* — those are only entered when they're called.
        // No `main` is required.
        let mut Last = Value::Unit;
        for Item in TopLevel {
            if matches!(Item.Kind, LayerKind::Function { .. }) {
                continue;
            }
            Last = self.RunLayer(Item)?;
        }
        Ok(Last)
    }

    // ============================================
    // Function Resolution
    // ============================================

    fn FindMainFunction<'lifetime>(
        &self,
        Layers: &'lifetime [Layer],
    ) -> CompilerResult<&'lifetime Layer> {
        for Layer in Layers {
            if let LayerKind::Function { Name, .. } = &Layer.Kind {
                if Name == "main" {
                    return Ok(Layer);
                }
            }
        }
        Err(CompilerError::RuntimeError("No 'main' function found".to_string()))
    }

    /// Finds a user-defined function layer by name.
    pub(crate) fn FindFunctionByName<'lifetime>(
        &self,
        Layers: &'lifetime [Layer],
        FuncName: &str,
    ) -> CompilerResult<&'lifetime Layer> {
        for Layer in Layers {
            if let LayerKind::Function { Name, .. } = &Layer.Kind {
                if Name == FuncName {
                    return Ok(Layer);
                }
            }
        }
        Err(CompilerError::RuntimeError(format!(
            "No function named '{}' found",
            FuncName
        )))
    }

    // ============================================
    // Utility Functions
    // ============================================

    pub(crate) fn CheckType(&self, Value: &Value, Type_: &Type) -> bool {
        match (Value, Type_) {
            (Value::Int(_), Type::BitPrecise('i', _)) => true,
            (Value::Int(_), Type::BitPrecise('u', _)) => true,
            (Value::Float(_), Type::BitPrecise('f', _)) => true,
            (Value::Bool(_), Type::BitPrecise('b', 1)) => true,
            (Value::Unit, Type::Unit) => true,
            _ => false,
        }
    }

    /// The zero value for a declared type, used when a binding has no initializer.
    pub(crate) fn DefaultValue(&self, Type_: &Option<Type>) -> CompilerResult<Value> {
        match Type_ {
            Some(Type::BitPrecise('i', _)) => Ok(Value::Int(0)),
            Some(Type::BitPrecise('f', _)) => Ok(Value::Float(0.0)),
            Some(Type::BitPrecise('b', _)) => Ok(Value::Bool(false)),
            Some(Type::Unit) => Ok(Value::Unit),
            _ => Err(CompilerError::TypeError("Cannot infer default type".to_string())),
        }
    }

    /// Looks the name up in the Builtins map. `Some(result)` means it was a
    /// built-in (handled here); `None` means fall through to user functions.
    pub(crate) fn CallBuiltin(&mut self, Name: &str, Args: &[Value]) -> Option<CompilerResult<Value>> {
        // Copy the fn pointer out of the map so the borrow on self.Builtins ends
        // before we hand &mut self to the built-in.
        let Func = self.Builtins.get(Name).copied()?;
        Some(Func(self, Args))
    }
}
