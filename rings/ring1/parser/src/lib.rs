use ast::Stmt;
use lexer::Token;


pub struct Parser {
    tokens: Vec<Token>,
    position: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt, String> {
        let token = self.peek();
        match token {
            Some(Token::Panic) => {
                self.advance();
                self.consume(Token::Semicolon, "Expected ';' after panic")?;
                Ok(Stmt::Panic)
            }
            Some(Token::Unreachable) => {
                self.advance();
                self.consume(Token::Semicolon, "Expected ';' after unreachable")?;
                Ok(Stmt::Unreachable)
            }
            _ => Err(format!("Unexpected token: {:?}", token)),
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.tokens.get(self.position - 1)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn consume(&mut self, target: Token, message: &str) -> Result<(), String> {
        if self.peek() == Some(&target) {
            self.advance();
            Ok(())
        } else {
            Err(message.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_basic() {
        let tokens = vec![Token::Panic, Token::Semicolon];
        let mut parser = Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast, vec![Stmt::Panic]);
    }
}
