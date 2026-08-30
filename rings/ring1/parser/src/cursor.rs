//! Low-level token cursor: peeking, advancing, and matching.

use lexer::token::{Token, TokenKind};

use crate::Parser;

impl Parser {
    /// Look at the current token.
    pub fn Peek(&self) -> Option<&Token> {
        self.Tokens.get(self.Position)
    }

    /// Look ahead or behind by a specific amount. Positive = ahead, 0 = current,
    /// negative = into History (-1 = most recently consumed).
    pub fn PeekAt(&self, Offset: isize) -> Option<&Token> {
        if Offset >= 0 {
            self.Tokens.get(self.Position + Offset as usize)
        } else {
            let back = (-Offset) as usize;
            if back == 0 || back > self.History.len() {
                return None;
            }
            self.History.get(self.History.len() - back)
        }
    }

    /// Pull the next token from the list, recording it in History.
    pub fn Advance(&mut self) -> Option<&Token> {
        if !self.IsAtEnd() {
            if let Some(tok) = self.Tokens.get(self.Position) {
                self.History.push(tok.clone());
            }
            self.Position += 1;
        }
        self.Tokens.get(self.Position - 1)
    }

    /// Check the current token's kind without consuming.
    pub fn Check(&self, Target: TokenKind) -> bool {
        self.Peek().map(|t| &t.Kind) == Some(&Target)
    }

    /// Consume the current token if it matches `Target`.
    pub fn Match(&mut self, Target: TokenKind) -> bool {
        if self.Check(Target) {
            self.Advance();
            true
        } else {
            false
        }
    }

    /// True once the cursor has run past the last token.
    pub fn IsAtEnd(&self) -> bool {
        self.Position >= self.Tokens.len()
    }

    /// Enforce a specific token or error out with `Message`.
    pub fn Consume(&mut self, Target: TokenKind, Message: &str) -> Result<(), String> {
        if self.Check(Target) {
            self.Advance();
            Ok(())
        } else {
            Err(format!("{}. Found {:?}", Message, self.Peek()))
        }
    }
}
