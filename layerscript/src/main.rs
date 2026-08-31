#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ast::Layer;
use lexer::Lexer as LexerStruct;
use parser::Parser as ParserStruct;
use elaboration::ElaborationContext;
use command_parser::{Cli, Commands};
use code_runner::*;
use lexer::token::*;

/// runs the lexer, parser, and elaborator in order
fn RunPipeline(SourceCode: &str, Verbose: bool) {
    // 1. Run the Lexer
    let Tokens: Vec<_> = LexerStruct::New(SourceCode).collect();
    if Verbose {
        println!("Tokens: {:?}", Tokens);
    }

    // 2. Run the Parser
    let mut P = ParserStruct::New(Tokens,Layer::New());
    match P.Parse() {
        Ok(Ast) => {
            println!("AST constructed with {} top-level items.", Ast.Children.len());
            if Verbose {
                println!("AST Detail: {:#?}", Ast);
            }

            // 3. Run Elaboration / Constraint extraction
            let mut ElabCtx = ElaborationContext::New();
            if let Err(E) = ElabCtx.ElaborateLayer(&Ast) {
                eprintln!("Elaboration Error: {}", E);
                return;
            }
            
            println!("Verification & Compilation Successful!");
            let mut NewRunner = CodeRunner::New(CompilerConfig::New());
            println!("Running code...\n\n");
            // Use debug print if we want to print CodeRunner
            // (Note: CodeRunner derives Debug)
            let RunnerCode = NewRunner.RunCode(&[Ast]);
            match RunnerCode {
                Ok(Value) => println!("Execution Result: {:?}", Value),
                Err(E) => {
                    let Line = P.Tokens.last().map(|t| t.Line).unwrap_or(0);
                    eprintln!("Execution Error: {:?},\n Line: {:?}", E, Line);
                }
            }
            
            
            
            
            //println!("Tokens: {:?}",Ast.Children);
        }
        Err(E) => {
            eprintln!("Parsing Error: {} Line: {}", E,P.Peek().map(|t| t.Line).unwrap_or(0));
        }
    }
}

/// entry point for the executable
fn main() {
    // Parse arguments using clap
    let Args = Cli::ParseArgs();

    global_config::Verbose.store(Args.Verbose, std::sync::atomic::Ordering::Relaxed);
    global_config::DebugMode.store(Args.Debug, std::sync::atomic::Ordering::Relaxed);

    match &Args.Command {
        Commands::Compile { Input, OptLevel } => {
            println!("Reading source file: {:?}", Input);
            match std::fs::read_to_string(Input) {
                Ok(SourceCode) => {
                    println!("Compiling with optimization level -O{}", OptLevel);
                    RunPipeline(&SourceCode, Args.Verbose);
                }
                Err(E) => {
                    eprintln!("Error reading file {:?}: {}", Input, E);
                }
            }
        }
        Commands::Eval { Code } => {
            println!("Evaluating inline code: '{}'", Code);
            RunPipeline(Code, Args.Verbose);
        }
        Commands::Test { Filter } => {
            println!("Running tests. Filter: {:?}", Filter);
        }
    }
}
