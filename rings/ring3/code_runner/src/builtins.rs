//! Built-in functions and the registry that maps their names to implementations.

use std::collections::HashMap;

use parser::error::{CompilerError, CompilerResult};
use crate::runner::CodeRunner;
use crate::value::{FormatValue, Value};

/// Signature every built-in shares: the runner (for state/output) plus the
/// already-evaluated arguments, returning a [`Value`].
pub type BuiltinFn = fn(&mut CodeRunner, &[Value]) -> CompilerResult<Value>;

/// Registers every built-in into `Map`. Add a line here to expose a new one.
pub fn AddBuiltins(Map: &mut HashMap<String, BuiltinFn>) {
    Map.insert("print".to_string(), Builtin_Print as BuiltinFn);
    Map.insert("println".to_string(), Builtin_Println as BuiltinFn);
    Map.insert("type".to_string(), Builtin_Type as BuiltinFn);
}

/// Prints its arguments separated by spaces, with no trailing newline.
fn Builtin_Print(_Runner: &mut CodeRunner, Args: &[Value]) -> CompilerResult<Value> {
    let Line = Args.iter().map(FormatValue).collect::<Vec<_>>().join(" ");
    print!("{}", Line);
    Ok(Value::Unit)
}

/// Like `print`, but adds a trailing newline.
fn Builtin_Println(_Runner: &mut CodeRunner, Args: &[Value]) -> CompilerResult<Value> {
    let Line = Args.iter().map(FormatValue).collect::<Vec<_>>().join(" ");
    println!("{}", Line);
    Ok(Value::Unit)
}

/// Returns a string representation of the argument's type.
fn Builtin_Type(_Runner: &mut CodeRunner, Args: &[Value]) -> CompilerResult<Value> {
    if Args.len() != 1 {
        return Err(CompilerError::RuntimeError(format!(
            "Built-in function 'type' expected 1 argument, got {}",
            Args.len()
        )));
    }
    let TypeStr = match &Args[0] {
        Value::Unit => "unit",
        Value::Int(_) => "int",
        Value::Float(_) => "float",
        Value::Bool(_) => "bool",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Struct(_) => "struct",
    };
    Ok(Value::String(TypeStr.to_string()))
}
