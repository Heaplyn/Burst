//! Ring 0 · Lexer · **Keyword table**
//!
//! Maps a bare identifier string to its keyword `TokenKind`, or `None` if it's
//! a user identifier / bit-precise type / literal.
//! Requires: [`token`](crate::token).

use crate::token::TokenKind;

/// Returns `Some(TokenKind)` if `word` is a keyword or a bit-precise type;
/// `None` if it should be lexed as an `Ident`.
pub fn LookupKeyword(word: &str) -> Option<TokenKind> {
    Some(match word {
        "function" | "fn" => TokenKind::Function,
        "var" => TokenKind::Let(true),
        "let" => TokenKind::Let(false),
        "mut" => TokenKind::Mut,
        "where" => TokenKind::Where,
        "or" => TokenKind::WhereOr,
        "havoc" => TokenKind::Havoc,
        "interrupt" => TokenKind::Interrupt,
        "unreachable" => TokenKind::Unreachable,
        "panic" => TokenKind::Panic,
        "as" => TokenKind::As,
        "match" => TokenKind::Match,
        "struct" => TokenKind::Struct,
        "enum" => TokenKind::Enum,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "while" => TokenKind::While,
        "for" => TokenKind::For,
        "loop" => TokenKind::Loop,
        "return" => TokenKind::Return,
        "goto" => TokenKind::Goto,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        _ => return AsBitPreciseType(word),
    })
}

/// Recognises `i32`, `u8`, `b16`, `f64`, etc. — a family letter followed by digits.
fn AsBitPreciseType(word: &str) -> Option<TokenKind> {
    let head = word.chars().next()?;
    if !matches!(head, 'i' | 'u' | 'b' | 'f') || word.len() < 2 {
        return None;
    }
    if !word[1..].chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let bits = word[1..].parse::<u32>().ok()?;
    Some(TokenKind::BitPreciseType { Kind: head, Bits: bits })
}
