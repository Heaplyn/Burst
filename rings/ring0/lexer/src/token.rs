// src/token.rs

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // ===== Keywords =====
    /// `function` keyword
    Function,
    /// `var` keyword
    Var,
    /// `return` keyword
    Return,
    /// `struct` keyword
    Struct,
    /// `enum` keyword
    Enum,
    /// `havoc` keyword
    Havoc,
    /// `interrupt` keyword
    Interrupt,
    /// `unreachable` keyword
    Unreachable,
    /// `panic` keyword
    Panic,
    /// `where` keyword
    Where,
    /// `as` keyword
    As,
    /// `match` keyword
    Match,

    // ===== Identifiers & Literals =====
    /// Identifier (variable/function/type name)
    Ident(String),
    /// Integer literal (e.g., 42, 0xFF)
    IntLiteral(i64),
    /// Float literal (e.g., 3.14, 1.5e-10)
    FloatLiteral(f64),
    /// String literal (e.g., 'hello')
    StringLiteral(String),
    /// Bit-precise type (e.g., i32, u16, b8, f64)
    BitPreciseType { Kind: char, Bits: u32 },
    
    // ===== Operators & Symbols =====
    /// `+` addition operator
    Plus,
    /// `-` subtraction or negation
    Minus,
    /// `*` multiplication or dereference
    Star,
    /// `/` division
    Slash,
    /// `%` modulo/remainder
    Percent,
    /// `=` assignment
    Equal,
    /// `->` function return arrow
    Arrow,
    /// `:=` type set operator
    TypeSet,
    /// `<` less than
    Less,
    /// `>` greater than
    Greater,
    /// `<=` less than or equal
    LessEqual,
    /// `>=` greater than or equal
    GreaterEqual,

    // ===== Delimiters =====
    /// `(` open parenthesis
    OpenParen,
    /// `)` close parenthesis
    CloseParen,
    /// `{` open brace / curly brace
    OpenBrace,
    /// `}` close brace / curly brace
    CloseBrace,
    /// `[` open square bracket
    OpenBracket,
    /// `]` close square bracket
    CloseBracket,
    /// `:` colon
    Colon,
    /// `;` semicolon
    Semicolon,
    /// `,` comma
    Comma,
    /// `.` dot
    Dot,
}