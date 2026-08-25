#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Bitfield(u32),   // b<N>
    Unsigned(u32),   // u<N>
    Signed(u32),     // i<N>
    Float(u32),      // f<N>
    Pointer(Box<Type>),
    Refined {
        base: Box<Type>,
        constraint: Box<Expression>, // Holds the "where <expr>" AST representation
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    LiteralInt(i64),
    LiteralFloat(f64),
    Variable(String),
    BinaryOp {
        op: String,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    Cast {
        expr: Box<Expression>,
        target_type: Box<Type>,
    },
}


#[derive(Debug, Clone, PartialEq)]
pub enum Statement {
    LetBinding { name: String, val: Expression },
    Assignment { target: Expression, val: Expression },
    Havoc(Expression),
    Panic,
    Unreachable,
    Interrupt { asm: String, target: Expression, body: Vec<Statement> },
}
