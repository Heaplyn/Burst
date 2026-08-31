//! Ring 0 · AST · **Type & Variable Storage**
//!
//! Per-layer maps of defined types, type aliases, and known variables.
//! Requires: [`Type`](crate::ty::Type), [`Expression`](crate::expr::Expression),
//! and [`SourceLocation`](crate::source::SourceLocation).

use std::collections::HashMap;

use crate::decl::{EnumVariant, GenericParam, StructField};
use crate::expr::Expression;
use crate::source::SourceLocation;
use crate::ty::Type;

/// Where we store types for each layer.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TypeStorage {
    pub DefinedTypes: HashMap<String, TypeDefinition>,
    pub TypeAliases: HashMap<String, Type>,
}

/// Where we store variables for each layer.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct VariableStorage {
    pub Variables: HashMap<String, VariableDefinition>,
}

/// Full definition of a variable in the environment.
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDefinition {
    pub Name: String,
    pub TypeAnnotation: Option<Type>,
    pub IsMutable: bool,
    pub Value: Expression,
}

/// Full definition of a user type.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDefinition {
    pub Name: String,
    pub Kind: TypeKind,
    pub SourceLocation: SourceLocation,
    pub Docs: Option<String>,
}

/// The physical layout of a type.
#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Struct(Vec<StructField>),
    Enum(Vec<EnumVariant>),
    Alias(Box<Type>),
    Generic(Vec<GenericParam>),
}
