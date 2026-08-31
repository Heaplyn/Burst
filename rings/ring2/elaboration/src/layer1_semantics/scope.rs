//! Ring 2 · Elaboration · **Layer 1 (Semantics) — Scope stack**
//!
//! A `Scope` is one lexical region — the body of a function, the arms of a
//! `match`, an inner `{}` block. Each scope maps names to [`SymbolId`]s.
//! Scopes nest, so lookup walks *inward → outward* until it finds a match or
//! runs off the top.
//!
//! `ScopeStack` is what the resolver actually uses at walk time. It owns a
//! `Vec<Scope>` — pushing when we enter a nested region, popping when we
//! leave — so we get RAII-ish scoping without shuffling `Box<Scope>` around.
//!
//! Requires: [`symbol::SymbolId`](super::symbol::SymbolId),
//! [`shadowing::IsShadowingAllowed`](super::shadowing::IsShadowingAllowed),
//! [`errors::SemanticError`](super::errors::SemanticError),
//! [`ast::SourceLocation`].

use std::collections::HashMap;

use ast::SourceLocation;

use super::errors::SemanticError;
use super::shadowing::{IsShadowingAllowed, ShadowDecision};
use super::symbol::SymbolId;

/// One lexical scope. Just a name→id map plus the source location of the
/// declaration that anchors it (used for error messages).
#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub Symbols: HashMap<String, ScopeEntry>,
}

#[derive(Debug, Clone)]
pub struct ScopeEntry {
    pub Id: SymbolId,
    pub DeclaredAt: SourceLocation,
}

impl Scope {
    pub fn New() -> Self {
        Self::default()
    }
}

/// Stack of scopes maintained during the resolver walk.
///
/// # Why a stack, not linked scopes
/// Linked scopes (`Box<Parent>`) force ownership gymnastics every time we
/// enter/leave a block. A stack lets the resolver just push, walk, pop — and
/// name lookup is a simple reverse iteration.
#[derive(Debug, Default)]
pub struct ScopeStack {
    stack: Vec<Scope>,
}

impl ScopeStack {
    pub fn New() -> Self {
        Self { stack: vec![Scope::New()] } // start with the program root scope
    }

    /// Push a fresh scope (entering a function body, block, arm, …).
    pub fn Enter(&mut self) {
        self.stack.push(Scope::New());
    }

    /// Pop the innermost scope (leaving that region).
    pub fn Leave(&mut self) {
        // The root scope is never popped — a bug in the walker would try to,
        // so we defensively refuse rather than panic later on empty-stack lookups.
        if self.stack.len() > 1 {
            self.stack.pop();
        }
    }

    /// Insert a symbol into the innermost scope.
    ///
    /// Returns an error if the name is already bound at this scope
    /// (duplicate), or already bound at an outer scope and shadowing is
    /// disallowed by policy.
    pub fn Insert(
        &mut self,
        Name: &str,
        Id: SymbolId,
        At: SourceLocation,
    ) -> Result<(), SemanticError> {
        // 1. Same-scope duplicate check.
        let top = self.stack.last().expect("stack invariant: root is never popped");
        if let Some(existing) = top.Symbols.get(Name) {
            return Err(SemanticError::DuplicateName {
                Name: Name.to_string(),
                First: existing.DeclaredAt.clone(),
                Second: At,
            });
        }

        // 2. Outer-scope shadowing check (only if the name is bound above us).
        let OuterHit = self.stack[..self.stack.len() - 1]
            .iter()
            .rev()
            .find_map(|s| s.Symbols.get(Name));

        if let Some(outer) = OuterHit {
            if IsShadowingAllowed(/* SameScope = */ false) == ShadowDecision::Deny {
                return Err(SemanticError::ShadowingDenied {
                    Name: Name.to_string(),
                    Outer: outer.DeclaredAt.clone(),
                    Inner: At,
                });
            }
        }

        // 3. Insert.
        let TopMut = self.stack.last_mut().unwrap();
        TopMut.Symbols.insert(Name.to_string(), ScopeEntry { Id, DeclaredAt: At });
        Ok(())
    }

    /// Look a name up through the stack, innermost → outermost. Returns
    /// `Some(SymbolId)` on the first match, `None` if the name is undefined.
    pub fn Lookup(&self, Name: &str) -> Option<SymbolId> {
        for scope in self.stack.iter().rev() {
            if let Some(entry) = scope.Symbols.get(Name) {
                return Some(entry.Id);
            }
        }
        None
    }

    pub fn Depth(&self) -> usize {
        self.stack.len()
    }
}
