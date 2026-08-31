/// every single token the lexer can find
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    /// what the token actually is (var ident etc)
    pub Kind: TokenKind,
    /// what line it started on
    pub Line: usize,
    /// what column it started on
    pub Column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable<Type> {
    /// what the token actually is (var ident etc)
    pub MemoryAddress:usize,
    Value:Type,
    Name: String,
}

/// the different kinds of words and symbols
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // ===== Keywords =====
    Function,
    /// immutable by default, set true to be mutable on the arg
    Let(bool),
    Mut,
    Return,
    Struct,
    Enum,
    Havoc,
    Interrupt,
    Unreachable,
    Panic,
    Where,
    As,
    Match,
    If,
    Else,
    While,
    For,
    Loop,
    Goto,

    // ===== Identifiers & Literals =====
    Ident(String),
    IntLiteral(i64),
    FloatLiteral(f64),
    StringLiteral(String),
    BitPreciseType { Kind: char, Bits: u32 },
    
    // ===== Operators & Symbols =====
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Equal,
    EqualEqual,
    NotEqual,
    Not,
    Arrow,
    TypeSet,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    And,
    Or,

    // ===== Delimiters =====
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    OpenBracket,
    CloseBracket,
    Colon,
    Semicolon,
    Comma,
    Dot,

    /// end of file token
    End,
    ///Invalid
    Invalid,
}
