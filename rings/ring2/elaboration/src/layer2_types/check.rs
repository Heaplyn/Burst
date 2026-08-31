//! Ring 2 · Elaboration · **Layer 2 (Types) — statement-level checks**
//!
//! The rules that decide whether a specific *use* of a value is well-typed:
//! operator applicability, assignment compatibility, argument matching,
//! return matching.
//!
//! Requires: [`infer::InferExpression`](super::infer::InferExpression),
//! [`ast::Type`].

use ast::Type;

use super::errors::TypeError;

/// Check that `Op` (an arithmetic / comparison / logical operator) is defined
/// on `Lhs` and `Rhs`, and return the result type.
///
/// Coercion rules (kept small on purpose):
///   - two ints of possibly different widths → widest of the two
///   - two floats of possibly different widths → widest of the two
///   - anything else with a comparison operator → `bool` (b1)
///   - mixed families or unsupported combos → error
pub fn CheckBinaryOp(Op: &str, Lhs: &Type, Rhs: &Type) -> Result<Type, TypeError> {
    // Comparison / equality: any two same-family operands → bool.
    let is_compare = matches!(Op, "==" | "!=" | "<" | ">" | "<=" | ">=");
    let is_logical = matches!(Op, "&&" | "||");
    let is_arith = matches!(Op, "+" | "-" | "*" | "/" | "%");
    let is_bitwise = matches!(Op, "&" | "|" | "^" | "<<" | ">>");

    let bool_type = Type::BitPrecise('b', 1);

    if is_logical {
        // Boolean operators demand booleans on both sides.
        if !IsBool(Lhs) || !IsBool(Rhs) {
            return Err(TypeError::BadOperator {
                Op: Op.to_string(),
                Lhs: Lhs.clone(),
                Rhs: Rhs.clone(),
                At: ast::SourceLocation::Builtin(),
            });
        }
        return Ok(bool_type);
    }

    if is_compare {
        // Comparisons: require same family, produce a bool.
        if SameFamily(Lhs, Rhs) {
            return Ok(bool_type);
        }
        return Err(TypeError::BadOperator {
            Op: Op.to_string(),
            Lhs: Lhs.clone(),
            Rhs: Rhs.clone(),
            At: ast::SourceLocation::Builtin(),
        });
    }

    if is_arith || is_bitwise {
        // Arithmetic / bitwise: return the wider of the two, which must
        // share a family.
        return WidenNumeric(Lhs, Rhs).ok_or_else(|| TypeError::BadOperator {
            Op: Op.to_string(),
            Lhs: Lhs.clone(),
            Rhs: Rhs.clone(),
            At: ast::SourceLocation::Builtin(),
        });
    }

    // Unknown operator string — treat as not-yet-implemented.
    Err(TypeError::NotImplemented(format!("operator {}", Op)))
}

/// Assignment: RHS type must coerce into LHS type. We only accept exact
/// matches or numeric widening for now.
pub fn CheckAssignment(Target: &Type, Value: &Type) -> Result<(), TypeError> {
    if TypesMatch(Target, Value) || WidenNumeric(Target, Value).as_ref() == Some(Target) {
        Ok(())
    } else {
        Err(TypeError::Mismatch {
            Expected: Target.clone(),
            Found: Value.clone(),
            At: ast::SourceLocation::Builtin(),
        })
    }
}

/// Function call: each argument's type must match its declared parameter type.
pub fn CheckFunctionCall(Params: &[Type], Args: &[Type]) -> Result<(), TypeError> {
    if Params.len() != Args.len() {
        return Err(TypeError::ArityMismatch {
            Name: String::from("<call>"),
            Expected: Params.len(),
            Found: Args.len(),
            At: ast::SourceLocation::Builtin(),
        });
    }
    for (p, a) in Params.iter().zip(Args.iter()) {
        CheckAssignment(p, a)?;
    }
    Ok(())
}

/// Return statement: the produced value's type must match the function's
/// declared return type.
pub fn CheckReturn(Declared: &Type, Actual: &Type) -> Result<(), TypeError> {
    CheckAssignment(Declared, Actual)
}

// ------------------------------------------------------------------
// Small helpers — kept private, used across the checks above.
// ------------------------------------------------------------------

fn IsBool(t: &Type) -> bool {
    matches!(Peel(t), Type::BitPrecise('b', 1))
}

/// Peel a `Type::Where(base, _)` down to its underlying base type. Layer 2
/// compares *structural shape* — the refinement predicate is Layer 3's job,
/// not Layer 2's. `where` clauses on parameters would otherwise cause a
/// spurious `Mismatch { Expected: i32, Found: Where(i32, …) }` on every call.
///
/// Peels transitively (handles `Where(Where(i32, P), Q)`), and is a no-op on
/// non-`Where` types.
fn Peel(t: &Type) -> &Type {
    let mut cur = t;
    while let Type::Where(inner, _) = cur {
        cur = inner;
    }
    cur
}

/// Two types are the *same numeric family* (both `i`, both `u`, both `f`, or
/// both `b`) or both `Unit`. Ignores refinements.
fn SameFamily(a: &Type, b: &Type) -> bool {
    match (Peel(a), Peel(b)) {
        (Type::BitPrecise(ka, _), Type::BitPrecise(kb, _)) => ka == kb,
        (Type::Unit, Type::Unit) => true,
        (Type::Named(x), Type::Named(y)) => x == y,
        _ => false,
    }
}

fn TypesMatch(a: &Type, b: &Type) -> bool {
    let (pa, pb) = (Peel(a), Peel(b));
    pa == pb || matches!((pa, pb), (Type::Inferred, _) | (_, Type::Inferred))
}

/// Return the "widest" (largest bit width) of two numeric types with the same
/// family. `None` if they aren't compatible. Refinements are ignored — the
/// widened type is a bare `BitPrecise` (the caller can re-wrap in `Where` if
/// it wants to carry a predicate forward).
fn WidenNumeric(a: &Type, b: &Type) -> Option<Type> {
    match (Peel(a), Peel(b)) {
        (Type::BitPrecise(ka, wa), Type::BitPrecise(kb, wb)) if ka == kb => {
            Some(Type::BitPrecise(*ka, (*wa).max(*wb)))
        }
        _ => None,
    }
}
