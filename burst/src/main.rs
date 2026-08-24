use lexer::Lexer;
use parser::Parser;
use elaboration::ElaborationContext;

fn main() {
    // A sample Burst code snippet
    let source_code = "panic;";
    println!("Compiling Burst code: '{}'", source_code);

    // 1. Run the Lexer
    let tokens: Vec<_> = Lexer::new(source_code).collect();
    println!("Tokens: {:?}", tokens);

    // 2. Run the Parser
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            println!("AST: {:?}", ast);

            // 3. Run Elaboration / Constraint extraction
            let mut elab_ctx = ElaborationContext::new();
            for stmt in &ast {
                if let Err(e) = elab_ctx.elaborate_statement(stmt) {
                    eprintln!("Elaboration Error: {}", e);
                    return;
                }
            }
            println!("Elaboration constraints: {:?}", elab_ctx.constraints);
            println!("Verification & Compilation Successful!");
        }
        Err(err) => {
            eprintln!("Parsing Error: {}", err);
        }
    }
}
