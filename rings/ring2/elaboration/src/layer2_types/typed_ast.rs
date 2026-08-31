//! Ring 2 · Elaboration · **Layer 2 (Types) — Typed AST side table**
//!
//! Rather than mint a whole parallel tree, Layer 2 keeps a **side map**:
//! `LayerId → Type` for statement layers whose "result type" matters (function
//! return, variable binding, expression statement), and a lexical `TypeEnv`
//! stack that mirrors the resolver's scope stack.
//!
//! Downstream layers (3, 4, 5) don't need a whole Typed tree — they need to
//! ask "what's the type of this expression at this scope?" and re-walking with
//! a `TypeEnv` answers that just as well.
//!
//! Requires: [`ast::LayerId`], [`ast::Type`], and (indirectly) the
//! [`SymbolTable`](crate::layer1_semantics::SymbolTable) Produced by Layer 1.

use std::collections::HashMap;

use ast::{LayerId, Type};

use crate::layer1_semantics::{SymbolKind, SymbolTable};

/// Association of a runtime name to a type at the current scope.
#[derive(Debug, Default, Clone)]
pub struct TypeEnv {
    /// Stack of name-map scopes. Newest (innermost) is last.
    stack: Vec<HashMap<String, Type>>,
}

impl TypeEnv {
    pub fn New() -> Self {
        Self { stack: vec![HashMap::new()] }
    }

    /// Build a `TypeEnv` populated with every Declared symbol from Layer 1's
    /// symbol table — useful for spot-checking expressions without doing a
    /// full walk. Real callers usually push/pop as they walk.
    pub fn FromSymbolTable(tbl: &SymbolTable) -> Self {
        let mut env = Self::New();
        for sym in tbl.Iter() {
            if let Some(t) = &sym.DeclaredType {
                // Skip type declarations (they aren't values you can look up
                // as names in expressions).
                if !matches!(sym.Kind, SymbolKind::Type | SymbolKind::Namespace) {
                    env.Insert(&sym.Name, t.clone());
                }
            }
        }
        env
    }

    pub fn Enter(&mut self) {
        self.stack.push(HashMap::new());
    }

    pub fn Leave(&mut self) {
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    pub fn Insert(&mut self, name: &str, ty: Type) {
        if let Some(top) = self.stack.last_mut() {
            top.insert(name.to_string(), ty);
        }
    }

    pub fn Lookup(&self, name: &str) -> Option<&Type> {
        for scope in self.stack.iter().rev() {
            if let Some(t) = scope.get(name) {
                return Some(t);
            }
        }
        None
    }
}

/// Cache of "the type of the value Produced by layer L" — populated during
/// type-checking, queried by refinement/optimization layers when they need to
/// know the shape of an intermediate result.
#[derive(Debug, Default, Clone)]
pub struct TypeTable {
    by_layer: HashMap<usize, Type>,
}

impl TypeTable {
    pub fn New() -> Self {
        Self::default()
    }

    pub fn Set(&mut self, id: LayerId, t: Type) {
        self.by_layer.insert(id.Id, t);
    }

    pub fn Get(&self, id: LayerId) -> Option<&Type> {
        self.by_layer.get(&id.Id)
    }
}

/// Convenience pair used by APIs that want to hand around a whole expression
/// with its Inferred_ type in one struct.
#[derive(Debug, Clone)]
pub struct TypedExpr {
    pub Expr: ast::Expression,
    pub Ty: Type,
}
