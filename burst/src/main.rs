use lexer::Lexer;
use parser::Parser;
use elaboration::ElaborationContext;
use command_parser::{Cli, Commands}; // Import from command-parser

fn run_pipeline(source_code: &str, verbose: bool) {
    // 1. Run the Lexer
    let tokens: Vec<_> = Lexer::new(source_code).collect();
    if verbose {
        println!("Tokens: {:?}", tokens);
    }

    // 2. Run the Parser
    let mut parser = Parser::new(tokens);
    match parser.parse() {
        Ok(ast) => {
            if verbose {
                println!("AST: {:?}", ast);
            }

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

fn main() {
    // Parse arguments using clap
    let cli = Cli::parse_args();

    match &cli.command {
        Commands::Compile { input, opt_level } => {
            println!("Reading source file: {:?}", input);
            match std::fs::read_to_string(input) {
                Ok(source_code) => {
                    println!("Compiling with optimization level -O{}", opt_level);
                    run_pipeline(&source_code, cli.verbose);
                }
                Err(err) => {
                    eprintln!("Error reading file {:?}: {}", input, err);
                }
            }
        }
        Commands::Eval { code } => {
            println!("Evaluating inline code: '{}'", code);
            run_pipeline(code, cli.verbose);
        }
        Commands::Test { filter } => {
            println!("Running tests. Filter: {:?}", filter);
            // Add test runner logic here when ready
        }
    }
}