#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ast::*;
use lexer::token;

pub struct parser {
    tokens: Vec<token>,
    position: usize,
}

impl parser {
    pub fn new(tokens: Vec<token>) -> Self {
        Self { tokens, position: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<statement>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<statement, String> {
        let tok = self.peek();
        match tok {
            Some(token::panic) => {
                self.advance();
                self.consume(token::semicolon, "Expected ';' after panic")?;
                Ok(statement::panic)
            }
            Some(token::unreachable) => {
                self.advance();
                self.consume(token::semicolon, "Expected ';' after unreachable")?;
                Ok(statement::unreachable)
            }
            Some(token::fn_) => {
                println!("Parsing function declaration");
                
                self.advance(); // Consume the 'fn' token
                self.consume(token::open_brace, "Expected '{' after function declaration")?;
               Ok(statement::function { name : format!("FuncComplete"), params: vec![], return_type: None, body : vec![] })
            }
            
            _ => Err(format!("Unexpected token: {:?}", tok)),
         };
    
    // Parse function body: { ... }
    //self.expect(token::LBRACE)?;
    //self.expect(token::RBRACE)?;
    Ok(statement::function { name : format!("hi"), params: vec![], return_type: None, body : vec![] })
        }
        
    
            

   
        
    

    fn peek(&self) -> Option<&token> {
        self.tokens.get(self.position)
    }

    fn advance(&mut self) -> Option<&token> {
        if !self.is_at_end() {
            self.position += 1;
        }
        self.tokens.get(self.position - 1)
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len()
    }

    fn consume(&mut self, target: token, message: &str) -> Result<(), String> {
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
        let tokens = vec![token::panic, token::semicolon];
        let mut p = parser::new(tokens);
        let ast = p.parse().unwrap();
        assert_eq!(ast, vec![statement::panic]);
    }
}
