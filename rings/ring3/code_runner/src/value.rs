//! Runtime values and their display formatting.

/// A value produced while executing a program.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Unit,
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Array(Vec<Value>),
    Struct(Vec<(String, Value)>),
}

/// Renders a runtime [`Value`] as a human-readable string (used by `print`/`println`).
pub fn FormatValue(v: &Value) -> String {
    match v {
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        _ => "...".to_string(),
    }
}
