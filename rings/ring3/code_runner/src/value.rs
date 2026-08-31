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
        Value::Unit => "()".to_string(),
        Value::Int(n) => n.to_string(),
        Value::Float(f) => f.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        Value::Array(elements) => {
            let formatted_elements: Vec<String> = elements
                .iter()
                .map(FormatValue)
                .collect();
            format!("[{}]", formatted_elements.join(", "))
        }
        Value::Struct(fields) => {
            let formatted_fields: Vec<String> = fields
                .iter()
                .map(|(name, val)| format!("{}: {}", name, FormatValue(val)))
                .collect();
            format!("{{ {} }}", formatted_fields.join(", "))
        }
    }
}


impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", FormatValue(self))
    }
}
