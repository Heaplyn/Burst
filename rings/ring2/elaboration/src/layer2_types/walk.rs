//! Ring 2 · Elaboration · **Layer 2 (Types) — the walking pass**
//!
//! Traverses the layer tree, threading a [`TypeEnv`] through the scopes and
//! populating a [`TypeTable`]. Reports mismatch/arity/bad-op errors as it goes.
//!
//! Requires: everything in `layer2_types` and Layer 1's `SymbolTable` /
//! `ResolvedProgram`.

use ast::{Expression, Layer, LayerKind, Type};

use crate::layer1_semantics::ResolvedProgram;

use super::check::{CheckAssignment, CheckReturn};
use super::errors::TypeError;
use super::infer::InferExpression;
use super::typed_ast::{TypeEnv, TypeTable};

/// The full output of Layer 2 — a per-layer type map plus any errors.
#[derive(Debug, Default)]
pub struct TypedProgram {
    pub Types: TypeTable,
    pub Errors: Vec<TypeError>,
}

/// Public entry point. Walks the whole program and returns the Typed view.
///
/// `Resolved` is used to seed the environment with function-return types /
/// parameter types so `InferExpression` can resolve names.
pub fn CheckProgram(Root: &Layer, Resolved: &ResolvedProgram) -> TypedProgram {
    let mut w = Walker {
        env: TypeEnv::FromSymbolTable(&Resolved.Symbols),
        types: TypeTable::New(),
        errors: Vec::new(),
        CurrentReturn: None,
    };
    w.Walk(Root);
    TypedProgram { Types: w.types, Errors: w.errors }
}

struct Walker {
    env: TypeEnv,
    types: TypeTable,
    errors: Vec<TypeError>,
    /// While inside a function, the Declared return type — used by `Return`.
    CurrentReturn: Option<Type>,
}

impl Walker {
    fn Walk(&mut self, L: &Layer) {
        match &L.Kind {
            LayerKind::Program => {
                for c in &L.Children {
                    self.Walk(c);
                }
            }

            LayerKind::Function { Params, ReturnType, .. } => {
                // Push a body scope with the params bound to their types.
                self.env.Enter();
                for p in Params {
                    self.env.Insert(&p.Name, p.Type_.clone());
                }
                let PrevReturn = self.CurrentReturn.take();
                self.CurrentReturn = ReturnType.clone();

                for c in &L.Children {
                    self.Walk(c);
                }

                self.CurrentReturn = PrevReturn;
                self.env.Leave();
            }

            LayerKind::VariableBinding { Name, TypeAnnotation, InitialValue, .. } => {
                // Infer the initializer, resolve the Declared type, and cache.
                let Declared = TypeAnnotation.clone().unwrap_or(Type::Inferred);
                let FinalType = match InitialValue {
                    Some(init) => match InferExpression(init, &self.env) {
                        Ok(Inferred_) => {
                            // If the user wrote a type, RHS must fit.
                            if !matches!(Declared, Type::Inferred) {
                                if let Err(e) = CheckAssignment(&Declared, &Inferred_) {
                                    self.errors.push(e);
                                }
                                Declared.clone()
                            } else {
                                Inferred_
                            }
                        }
                        Err(e) => {
                            self.errors.push(e);
                            Declared.clone()
                        }
                    },
                    None => Declared.clone(),
                };

                self.env.Insert(Name, FinalType.clone());
                self.types.Set(L.Id.clone(), FinalType);
            }

            LayerKind::Assignment { Target, Value } => {
                let lt = InferExpression(Target, &self.env);
                let rt = InferExpression(Value, &self.env);
                match (lt, rt) {
                    (Ok(l), Ok(r)) => {
                        if let Err(e) = CheckAssignment(&l, &r) {
                            self.errors.push(e);
                        }
                    }
                    (Err(e), _) | (_, Err(e)) => self.errors.push(e),
                }
            }

            LayerKind::Expression(e) => match InferExpression(e, &self.env) {
                Ok(t) => self.types.Set(L.Id.clone(), t),
                Err(e) => self.errors.push(e),
            },

            LayerKind::Return { Value } => {
                let Produced = match Value {
                    Some(v) => match InferExpression(v, &self.env) {
                        Ok(t) => t,
                        Err(e) => {
                            self.errors.push(e);
                            Type::Inferred
                        }
                    },
                    None => Type::Unit,
                };
                if let Some(Decl) = &self.CurrentReturn {
                    if let Err(e) = CheckReturn(Decl, &Produced) {
                        self.errors.push(e);
                    }
                }
                self.types.Set(L.Id.clone(), Produced);
            }

            LayerKind::Conditional { Condition, .. } => {
                // Condition should be a bool; recurse into arms.
                match InferExpression(Condition, &self.env) {
                    Ok(t) if matches!(t, Type::BitPrecise('b', _)) => {}
                    Ok(t) => self.errors.push(TypeError::Mismatch {
                        Expected: Type::BitPrecise('b', 1),
                        Found: t,
                        At: L.Metadata.SourceLocation.clone(),
                    }),
                    Err(e) => self.errors.push(e),
                }
                for c in &L.Children {
                    self.Walk(c);
                }
            }

            LayerKind::Block => {
                self.env.Enter();
                for c in &L.Children {
                    self.Walk(c);
                }
                self.env.Leave();
            }

            _ => {
                for c in &L.Children {
                    self.Walk(c);
                }
            }
        }
    }
}

// Silence "unused import" if the walker ever loses these — kept live for
// downstream users of the module.
#[allow(dead_code)]
fn _keep_imports_live(_e: &Expression) {}
