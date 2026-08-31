//! Ring 2 · Elaboration · **Layer 1 (Semantics) — Symbols**
//!
//! A `Symbol` is the *resolved identity* of a declaration. Every `var x`,
//! `function foo`, and parameter name in the source becomes exactly one
//! Symbol — distinct declarations that happen to share a name get distinct
//! `SymbolId`s.
//!
//! The `SymbolTable` is an arena — symbols are pushed in and referred to by a
//! small integer index (`SymbolId`). This gives us:
//!   - `Copy` handles (a `SymbolId` is just a `usize`) so we can stash them
//!     freely in maps, sets, and later layers' side tables
//!   - stable references — arena entries never move
//!   - fast lookup by id
//!
//! Requires: [`ast::Type`], [`ast::SourceLocation`].

use ast::{SourceLocation, Type};

/// Uniquely identifies a declaration. Assigned by [`SymbolTable::Intern`] and
/// used everywhere a later layer wants to refer back to "that specific `x`,
/// not any other `x` in the program".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SymbolId(pub usize);

/// One declaration's full record. The `Kind` distinguishes what *kind* of
/// declaration it is (a `var`, a `function`, a parameter, etc.), and the
/// optional `DeclaredType` remembers the annotation the user wrote (or
/// `None` if they left it out and elaboration needs to infer it).
#[derive(Debug, Clone)]
pub struct Symbol {
    pub Id: SymbolId,
    pub Name: String,
    pub Kind: SymbolKind,
    pub DeclaredType: Option<Type>,
    pub DeclaredAt: SourceLocation,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SymbolKind {
    /// A `var` / `let` binding. `IsMutable` is `true` for `var`.
    Variable { IsMutable: bool },
    /// A `function` declaration.
    Function,
    /// A formal parameter of a function.
    Parameter,
    /// A user-defined type (`struct`, `enum`, or `type` alias).
    Type,
    /// A `namespace` block.
    Namespace,
}

/// Arena of every symbol in the program. Owned by
/// [`ResolvedProgram`](super::resolve::ResolvedProgram); layer 2+ query it by
/// `SymbolId`.
#[derive(Debug, Default, Clone)]
pub struct SymbolTable {
    entries: Vec<Symbol>,
}

impl SymbolTable {
    pub fn New() -> Self {
        Self { entries: Vec::new() }
    }

    /// Reserve a fresh `SymbolId` and store `sym` under it. Returns the id.
    ///
    /// The caller is expected to have already set `sym.Id` to a placeholder
    /// (any value — we overwrite it here) so construction sites don't need to
    /// know the arena size beforehand.
    pub fn Intern(&mut self, mut sym: Symbol) -> SymbolId {
        let id = SymbolId(self.entries.len());
        sym.Id = id;
        self.entries.push(sym);
        id
    }

    /// Look a symbol up by id. Returns `None` only if the caller cooked up an
    /// id from a different table — otherwise this always succeeds because
    /// `SymbolId`s are only issued by `Intern` above.
    pub fn Get(&self, id: SymbolId) -> Option<&Symbol> {
        self.entries.get(id.0)
    }

    pub fn Len(&self) -> usize {
        self.entries.len()
    }

    pub fn Iter(&self) -> impl Iterator<Item = &Symbol> {
        self.entries.iter()
    }
}
