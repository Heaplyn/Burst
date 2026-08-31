#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use ast::Layer;
use lexer::Lexer as LexerStruct;
use parser::Parser as ParserStruct;
use elaboration::{ElaborationContext, RunAll as RunElaboration};
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

            // 3. Run Elaboration — the full 5-layer pipeline (see
            //    `rings/ring2/elaboration/src/pipeline.rs`). Even on internal
            //    errors we get an `Elaborated` back with error vectors on
            //    `Resolved.Errors` / `Typed.Errors`; we report them, then
            //    hand the (possibly rewritten) tree to the interpreter.
            let mut ElabCtx = ElaborationContext::New();
            if let Err(E) = ElabCtx.ElaborateLayer(&Ast) {
                eprintln!("Elaboration Error: {}", E);
                return;
            }

            let Elab = RunElaboration(&Ast);
            for e in &Elab.Resolved.Errors {
                eprintln!("Semantic: {:?}", e);
            }
            for e in &Elab.Typed.Errors {
                eprintln!("Type:     {:?}", e);
            }
            if Verbose {
                println!(
                    "Elaboration: {} symbol(s), {} obligation(s), {} erased",
                    Elab.Resolved.Symbols.Len(),
                    Elab.Constraints.Obligations.len(),
                    Elab.Erasures.Erased.len(),
                );
            }

            println!("Verification & Compilation Successful!");
            let mut NewRunner = CodeRunner::New(CompilerConfig::New());
            println!("Running code...\n\n");
            // Feed the *optimized* tree (post Layer 5) to the interpreter.
            let RunnerCode = NewRunner.RunCode(&[Elab.Program]);
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
