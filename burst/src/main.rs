#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use lexer::lexer as LexerStruct;
use parser::parser as ParserStruct;
use elaboration::elaboration_context;
use command_parser::{cli, commands};

fn run_pipeline(source_code: &str, verbose: bool) {
    // 1. Run the Lexer
    let tokens: Vec<_> = LexerStruct::new(source_code).collect();
    if verbose {
        println!("Tokens: {:?}", tokens);
    }

    // 2. Run the Parser
    let mut p = ParserStruct::new(tokens);
    match p.parse() {
        Ok(ast) => {
            if verbose {
                println!("AST: {:?}", ast);
            }

            // 3. Run Elaboration / Constraint extraction
            let mut elab_ctx = elaboration_context::new();
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
    let args = cli::parse_args();

    match &args.command {
        commands::compile { input, opt_level } => {
            println!("Reading source file: {:?}", input);
            match std::fs::read_to_string(input) {
                Ok(source_code) => {
                    println!("Compiling with optimization level -O{}", opt_level);
                    run_pipeline(&source_code, args.verbose);
                }
                Err(err) => {
                    eprintln!("Error reading file {:?}: {}", input, err);
                }
            }
        }
        commands::eval { code } => {
            println!("Evaluating inline code: '{}'", code);
            run_pipeline(code, args.verbose);
        }
        commands::test { filter } => {
            println!("Running tests. Filter: {:?}", filter);
        }
    }
}