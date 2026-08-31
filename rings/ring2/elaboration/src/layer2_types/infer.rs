//! Ring 2 · Elaboration · **Layer 2 (Types) — Expression type inference**
//!
//! Given an expression and a lexical environment mapping names → types,
//! compute the expression's type. This is a small **bidirectional inferrer**
//! — it walks the tree bottom-up, unifying operands and picking the "widest"
//! numeric type when a binary op mixes widths.
//!
//! Not real HM: we don't unify polymorphic variables, we don't infer
//! function signatures, and there are no let-generalizations. LayerScript's
//! types are almost entirely concrete (bit-precise integers, floats, bools,
//! etc.), which makes this small.
//!
//! Requires: [`ast::Expression`], [`ast::Type`], `TypeEnv` from
//! [`typed_ast`](super::typed_ast).

use ast::{Expression, Type};

use super::errors::TypeError;
use super::typed_ast::TypeEnv;

/// Infer the type of `expr` in the environment `env`.
///
/// The `env` argument is the current lexical mapping from names to types.
/// Callers build it during their own tree walk (usually mirroring the
/// resolver's `ScopeStack`).
pub fn InferExpression(expr: &Expression, env: &TypeEnv) -> Result<Type, TypeError> {
    match expr {
        // ---- Literals: pick the natural default width. ----
        // Users can constrain further at binding sites (`var x: i8 = 5;`).
        Expression::LiteralInt(_) => Ok(Type::BitPrecise('i', 32)),
        Expression::LiteralFloat(_) => Ok(Type::BitPrecise('f', 64)),
        Expression::LiteralBool(_) => Ok(Type::BitPrecise('b', 1)),
        Expression::LiteralString(_) => Ok(Type::Named("String".to_string())),

        // A bare type name used as a value (as in `IntOrFloat(true)`) has
        // "type of a type" semantics — we model that as a `Named("type")`.
        Expression::TypeLiteral { .. } | Expression::BitPreciseType { .. } => {
            Ok(Type::Named("type".to_string()))
        }

        // ---- A variable reference: look it up in the environment. ----
        Expression::Variable(name) => env
            .Lookup(name)
            .cloned()
            .ok_or_else(|| TypeError::NotImplemented(format!("no type for '{}'", name))),

        // ---- Binary op: infer both sides, then dispatch to the operator
        //      table for a compatible result type. ----
        Expression::BinaryOp { Op, Lhs, Rhs } => {
            let l = InferExpression(Lhs, env)?;
            let r = InferExpression(Rhs, env)?;
            super::check::CheckBinaryOp(Op, &l, &r)
        }

        // ---- Unary op: result type is the operand type for `-`, `!`, `~`;
        //      dereference (`*ptr`) peels off a Pointer/Reference. ----
        Expression::UnaryOp { Op, Target } => {
            let t = InferExpression(Target, env)?;
            match Op.as_str() {
                "*" => match t {
                    Type::Pointer(inner) | Type::Reference(inner) => Ok(*inner),
                    other => Err(TypeError::NotImplemented(format!(
                        "cannot dereference {:?}",
                        other
                    ))),
                },
                "!" | "-" | "~" => Ok(t),
                other => Err(TypeError::NotImplemented(format!("unary {}", other))),
            }
        }

        // ---- Function call: look the callee up as a variable (Layer 1
        //      declared functions as symbols, and Layer 2 stashes them in
        //      the env with their declared return type). We don't yet have
        //      full function-type-in-Type, so the env stores the return
        //      type directly for callable names. ----
        Expression::FunctionCall { Name, Args } => {
            // Type-check each arg for its side-effect on error propagation.
            for a in Args {
                let _ = InferExpression(a, env)?;
            }
            env.Lookup(Name).cloned().ok_or_else(|| {
                TypeError::NotImplemented(format!("no return type for '{}'", Name))
            })
        }

        // ---- Field / index access: we don't know struct layouts yet. ----
        Expression::MemberAccess { Target, .. } => {
            let _ = InferExpression(Target, env)?;
            Ok(Type::Inferred)
        }
        Expression::IndexAccess { Target, Index } => {
            let _ = InferExpression(Target, env)?;
            let _ = InferExpression(Index, env)?;
            Ok(Type::Inferred)
        }

        Expression::Invalid => Ok(Type::Inferred),
    }
}
