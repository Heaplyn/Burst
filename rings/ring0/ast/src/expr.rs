//! Ring 0 · AST · **Expression**
//!
//! Every syntactic thing that can be computed or matched on.
//! Requires: nothing at the crate level (uses `Type` transitively through `Where`
//! only inside the `Type` enum itself; `Expression` does not embed `Type`).

/// Everything we can compute or do math with.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    /// Just a variable name.
    Variable(String),
    /// A whole number.
    LiteralInt(i64),
    /// A number with a decimal.
    LiteralFloat(f64),
    /// True or false bits.
    LiteralBool(bool),
    /// Text inside quotes.
    LiteralString(String),
    /// Bit-precise type used as a value.
    TypeLiteral { Kind: char, Bits: u32 },
    /// Two things joined by an operator like `+` or `<`.
    BinaryOp {
        Op: String,
        Lhs: Box<Expression>,
        Rhs: Box<Expression>,
    },
    /// One thing with an operator like `*ptr`.
    UnaryOp {
        Op: String,
        Target: Box<Expression>,
    },
    /// Calling a function with args.
    FunctionCall {
        Name: String,
        Args: Vec<Expression>,
    },
    /// Reaching into a struct like `cpu.rax`.
    MemberAccess {
        Target: Box<Expression>,
        Member: String,
    },
    /// Bit-precise type specifically for the lexer.
    BitPreciseType { Kind: char, Bits: u32 },
    /// Indexing into an array or pointer like `ptr[index]`.
    IndexAccess {
        Target: Box<Expression>,
        Index: Box<Expression>,
    },
    /// Invalid (used as a sentinel by error recovery paths).
    Invalid,
}
