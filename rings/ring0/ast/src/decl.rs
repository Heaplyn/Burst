//! Ring 0 · AST · **Declarations**
//!
//! Small named-and-typed pieces that show up inside functions/structs/enums.
//! Requires: [`Type`](crate::ty::Type).

use crate::ty::Type;

/// Function params with names and types.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub Name: String,
    pub Type_: Type,
    pub Value: Type,
}

/// Struct fields with names and types.
#[derive(Debug, Clone, PartialEq)]
pub struct StructField {
    pub Name: String,
    pub Type_: Type,
}

/// Enum variants with optional payloads.
#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub Name: String,
    pub Payload: Option<Type>,
}

/// Generic params for type functions.
#[derive(Debug, Clone, PartialEq)]
pub struct GenericParam {
    pub Name: String,
    pub Bound: Option<Type>,
}
