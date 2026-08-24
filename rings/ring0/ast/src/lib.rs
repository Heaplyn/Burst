#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Bitfield(u32),   // b<N>
    Unsigned(u32),   // u<N>
    Signed(u32),     // i<N>
    Float(u32),      // f<N>
    Pointer(Box<Type>),
    Refined {
        base: Box<Type>,
        constraint: Box<Expr>, // Holds the "where <expr>" AST representation
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    LiteralInt(i64),
    LiteralFloat(f64),
    Variable(String),
    BinaryOp {
        op: String,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Cast {
        expr: Box<Expr>,
        target_type: Box<Type>,
    },
}


#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    LetBinding { name: String, val: Expr },
    Assignment { target: Expr, val: Expr },
    Havoc(Expr),
    Panic,
    Unreachable,
    Interrupt { asm: String, target: Expr, body: Vec<Stmt> },
}
