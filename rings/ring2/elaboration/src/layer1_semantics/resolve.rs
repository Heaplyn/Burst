//! Ring 2 · Elaboration · **Layer 1 (Semantics) — Name resolution walker**
//!
//! This is the actual pass. It walks the layer tree, opens a fresh scope at
//! every function / block, interns each declaration as a `Symbol`, and checks
//! every `Expression::Variable("x")` against the current scope stack.
//!
//! Because `Expression::Variable` carries no unique node id, we don't produce
//! an annotated tree. Instead we produce:
//!   - a `SymbolTable` of every declaration
//!   - a `Vec<SemanticError>` of every use of an undefined name (and every
//!     duplicate declaration)
//!
//! Later layers get the symbol table plus a fresh `ScopeStack` and re-walk
//! the tree — the scope structure is deterministic, so re-walking produces
//! the same bindings.
//!
//! Requires: [`ast::Layer`], [`ast::LayerKind`], [`ast::Expression`],
//! [`scope::ScopeStack`], [`symbol::SymbolTable`], [`errors::SemanticError`].

use ast::{Expression, Layer, LayerKind, SourceLocation};

use super::errors::SemanticError;
use super::scope::ScopeStack;
use super::symbol::{Symbol, SymbolId, SymbolKind, SymbolTable};

/// The output of Layer 1.
///
/// - `Symbols` — the arena of every declaration in the program.
/// - `Errors`  — everything the resolver found that should be reported (may
///   be non-empty even when we return `Ok`; the caller decides whether any
///   error is fatal, e.g. `@strict` mode).
#[derive(Debug, Default)]
pub struct ResolvedProgram {
    pub Symbols: SymbolTable,
    pub Errors: Vec<SemanticError>,
}

/// Public entry point. Walks the whole program, resolving names.
///
/// Seeds the root scope with the names of every built-in function known to
/// the runtime (`print`, `println`, …) so calls to them aren't flagged as
/// "undefined". This list must stay in sync with `builtins::AddBuiltins` in
/// `code_runner`; a future refactor could move the list into `ast` so both
/// crates share one source of truth.
pub fn Resolve(Root: &Layer) -> ResolvedProgram {
    let mut r = Resolver::New();
    r.SeedBuiltins(&["print", "println"]);
    r.WalkLayer(Root);
    ResolvedProgram { Symbols: r.symbols, Errors: r.errors }
}

// ------------------------------------------------------------------
// Internal walker
// ------------------------------------------------------------------

struct Resolver {
    symbols: SymbolTable,
    scopes: ScopeStack,
    errors: Vec<SemanticError>,
}

impl Resolver {
    fn New() -> Self {
        Self {
            symbols: SymbolTable::New(),
            scopes: ScopeStack::New(),
            errors: Vec::new(),
        }
    }

    /// Declare a list of built-in function names in the root scope so that
    /// references to them from user code resolve rather than error.
    fn SeedBuiltins(&mut self, Names: &[&str]) {
        for n in Names {
            self.DeclareSymbol(
                n,
                SymbolKind::Function,
                Some(ast::Type::Unit), // return type is (); overriden if we type them properly later
                SourceLocation::Builtin(),
            );
        }
    }

    /// Walks one layer. Dispatches on `LayerKind` — declarations get interned
    /// into the current scope, blocks open a fresh scope, expressions have
    /// their variable references checked.
    fn WalkLayer(&mut self, L: &Layer) {
        match &L.Kind {
            // ---- The whole program: walk children in the root scope. ----
            LayerKind::Program => {
                for child in &L.Children {
                    self.WalkLayer(child);
                }
            }

            // ---- A function: declare it, then walk its body in a fresh scope
            //      that has its parameters pre-declared. ----
            LayerKind::Function { Name, Params, ReturnType, .. } => {
                // 1. Declare the function itself in the *current* (outer) scope.
                self.DeclareSymbol(
                    Name,
                    SymbolKind::Function,
                    ReturnType.clone(),
                    L.Metadata.SourceLocation.clone(),
                );

                // 2. Enter a fresh scope for the body.
                self.scopes.Enter();

                // 3. Declare each parameter into the body scope.
                for p in Params {
                    self.DeclareSymbol(
                        &p.Name,
                        SymbolKind::Parameter,
                        Some(p.Type_.clone()),
                        L.Metadata.SourceLocation.clone(),
                    );
                }

                // 4. Walk the body (function's children == body statements).
                for child in &L.Children {
                    self.WalkLayer(child);
                }

                self.scopes.Leave();
            }

            // ---- A variable binding: declare the name after resolving the
            //      initializer (so `var x = x;` correctly reports the RHS `x`
            //      as undefined instead of finding the still-being-declared
            //      one). ----
            LayerKind::VariableBinding { Name, TypeAnnotation, IsMutable, InitialValue, Hooks } => {
                if let Some(init) = InitialValue {
                    self.CheckExpr(init, &L.Metadata.SourceLocation);
                }

                self.DeclareSymbol(
                    Name,
                    SymbolKind::Variable { IsMutable: *IsMutable },
                    TypeAnnotation.clone(),
                    L.Metadata.SourceLocation.clone(),
                );

                // Hook bodies aren't syntactic children of the binding right
                // now (they're stored as `LayerKind` blobs in the hook), so
                // we don't recurse into them here. A future pass would open a
                // scope with `new`, `old` params and walk each hook body.
                let _ = Hooks;
            }

            // ---- Assignment: both sides are expressions. ----
            LayerKind::Assignment { Target, Value } => {
                self.CheckExpr(Target, &L.Metadata.SourceLocation);
                self.CheckExpr(Value, &L.Metadata.SourceLocation);
            }

            // ---- A bare expression statement. ----
            LayerKind::Expression(e) => {
                self.CheckExpr(e, &L.Metadata.SourceLocation);
            }

            // ---- Blocks and everything with children: open scope, recurse. ----
            LayerKind::Block => {
                self.scopes.Enter();
                for child in &L.Children {
                    self.WalkLayer(child);
                }
                self.scopes.Leave();
            }

            // ---- Control flow: condition first, then the branch bodies. ----
            LayerKind::Conditional { Condition, .. } => {
                self.CheckExpr(Condition, &L.Metadata.SourceLocation);
                for child in &L.Children {
                    self.WalkLayer(child);
                }
            }

            LayerKind::Loop { .. } => {
                // Loop condition is stored inside `Kind`; for now we just walk
                // the children (the loop body) — a future pass can dig into
                // `LoopKind::While(expr)` / `For{Init,Condition,Update}`.
                self.scopes.Enter();
                for child in &L.Children {
                    self.WalkLayer(child);
                }
                self.scopes.Leave();
            }

            LayerKind::Return { Value } => {
                if let Some(v) = Value {
                    self.CheckExpr(v, &L.Metadata.SourceLocation);
                }
            }

            LayerKind::Havoc { Target } => {
                self.CheckExpr(Target, &L.Metadata.SourceLocation);
            }

            LayerKind::Interrupt { .. } => {
                for child in &L.Children {
                    self.WalkLayer(child);
                }
            }

            // ---- Type/struct/enum declarations: declare the name; fields
            //      don't participate in ordinary name resolution. ----
            LayerKind::Struct { Name, .. } => {
                self.DeclareSymbol(
                    Name,
                    SymbolKind::Type,
                    None,
                    L.Metadata.SourceLocation.clone(),
                );
            }
            LayerKind::Enum { Name, .. } => {
                self.DeclareSymbol(
                    Name,
                    SymbolKind::Type,
                    None,
                    L.Metadata.SourceLocation.clone(),
                );
            }

            // ---- Terminals: no names to resolve. ----
            LayerKind::Panic | LayerKind::Unreachable => {}

            // ---- Anything else: descend into children so we don't silently
            //      skip a whole subtree. ----
            _ => {
                for child in &L.Children {
                    self.WalkLayer(child);
                }
            }
        }
    }

    /// Intern a declaration and insert it into the current scope, reporting a
    /// duplicate/shadow error if the scope rejects it.
    fn DeclareSymbol(
        &mut self,
        name: &str,
        kind: SymbolKind,
        declared_type: Option<ast::Type>,
        at: SourceLocation,
    ) -> SymbolId {
        let id = self.symbols.Intern(Symbol {
            Id: SymbolId(0), // placeholder — Intern overwrites
            Name: name.to_string(),
            Kind: kind,
            DeclaredType: declared_type,
            DeclaredAt: at.clone(),
        });
        if let Err(e) = self.scopes.Insert(name, id, at) {
            self.errors.push(e);
        }
        id
    }

    /// Walk an expression tree, reporting any variable reference that doesn't
    /// resolve in the current scope stack.
    fn CheckExpr(&mut self, e: &Expression, at: &SourceLocation) {
        match e {
            Expression::Variable(name) => {
                if self.scopes.Lookup(name).is_none() {
                    self.errors.push(SemanticError::UndefinedName {
                        Name: name.clone(),
                        At: at.clone(),
                    });
                }
            }
            Expression::BinaryOp { Lhs, Rhs, .. } => {
                self.CheckExpr(Lhs, at);
                self.CheckExpr(Rhs, at);
            }
            Expression::UnaryOp { Target, .. } => self.CheckExpr(Target, at),
            Expression::FunctionCall { Name, Args } => {
                // A call site names a function; check that name too.
                if self.scopes.Lookup(Name).is_none() {
                    self.errors.push(SemanticError::UndefinedName {
                        Name: Name.clone(),
                        At: at.clone(),
                    });
                }
                for a in Args {
                    self.CheckExpr(a, at);
                }
            }
            Expression::MemberAccess { Target, .. } => self.CheckExpr(Target, at),
            Expression::IndexAccess { Target, Index } => {
                self.CheckExpr(Target, at);
                self.CheckExpr(Index, at);
            }
            // Literals & sentinels have nothing to resolve.
            Expression::LiteralInt(_)
            | Expression::LiteralFloat(_)
            | Expression::LiteralBool(_)
            | Expression::LiteralString(_)
            | Expression::TypeLiteral { .. }
            | Expression::BitPreciseType { .. }
            | Expression::Invalid => {}
        }
    }
}
