//! Ring 0 · Lexer · **Lexer struct and cursor helpers**
//!
//! Owns the character stream and line/column bookkeeping.
//! Requires: [`token`](crate::token).

use crate::token::{Token, TokenKind};

/// The thing that turns text into a stream of tokens.
pub struct Lexer<'a> {
    /// The actual text we are reading.
    pub Source: &'a str,
    /// Character iterator with lookahead.
    pub Chars: std::iter::Peekable<std::str::Chars<'a>>,
    /// Token we saved for later.
    pub PeekedToken: Option<Token>,
    /// Line count for error messages.
    pub CurrentLine: usize,
    /// Column count for error messages.
    pub CurrentColumn: usize,
}

impl Token {
    pub fn New(Kind: TokenKind, Line: usize, Column: usize) -> Self {
        Self { Kind, Line, Column }
    }
}

impl<'a> Lexer<'a> {
    /// Starts a new lexer at line 1 column 1.
    pub fn New(Source: &'a str) -> Self {
        Self {
            Source,
            Chars: Source.chars().peekable(),
            PeekedToken: None,
            CurrentLine: 1,
            CurrentColumn: 1,
        }
    }

    /// Pulls a char and keeps tracking in sync.
    pub(crate) fn AdvanceChar(&mut self) -> Option<char> {
        let ch = self.Chars.next()?;
        if ch == '\n' {
            self.CurrentLine += 1;
            self.CurrentColumn = 1;
        } else {
            self.CurrentColumn += 1;
        }
        Some(ch)
    }

    /// Looks at the next char without pulling it.
    pub(crate) fn PeekChar(&mut self) -> Option<&char> {
        self.Chars.peek()
    }

    /// Checks the next token without pulling it from the stream.
    pub fn PeekToken(&mut self) -> Option<Token> {
        if self.PeekedToken.is_none() {
            self.PeekedToken = self.NextToken();
        }
        self.PeekedToken.clone()
    }

    /// Clears out the spaces and newlines.
    pub(crate) fn SkipWhitespace(&mut self) {
        while let Some(&ch) = self.PeekChar() {
            if ch.is_whitespace() {
                self.AdvanceChar();
            } else {
                break;
            }
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;
    fn next(&mut self) -> Option<Self::Item> {
        self.NextToken()
    }
}
