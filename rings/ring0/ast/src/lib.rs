#![allow(non_camel_case_types)]
#![allow(non_snake_case)]


#[derive(Debug, Clone, PartialEq)]
pub enum statement {
    panic,
    unreachable,
    empty,
    let_binding {
        name: String,
        type_annotation: Option<Type>,
        value: expression,
    },
    assignment {
        target: expression,
        value: expression,
    },
    havoc {
        target: expression,
    },
    function {
        name: String,
        params: Vec<Param>,
        return_type: Option<Type>,
        body: Vec<statement>,
    },
    struct_declaration {
        name: String,
        fields: Vec<StructField>,
    },
    block(Vec<statement>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum expression {
    variable(String),
    literal_int(i64),
    literal_float(f64),
    type_literal { kind: char, bits: u32 },
    // Add more: binary_op, unary_op, function_call, etc.
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub name: String,
    pub type_: Type,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    bit_precise(char, u32),  // e.g., i32, u16, b8, f64
    named(String),           // User-defined types
    // Add more: pointer, array, etc.
}

impl Type {
    pub fn bit_precise(kind: char, bits: u32) -> Self {
        Type::bit_precise(kind, bits)
    }
    
    pub fn named(name: String) -> Self {
        Type::named(name)
    }
}