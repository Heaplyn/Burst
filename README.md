# Burst: The Principle of Most Speed

**Burst** is an experimental, bare-metal systems programming language designed for hyper-aggressive optimization. It models programs as a **Partially Ordered Multiset (POMSET) of traces** subject to logical constraints.

## 🏗️ Project Architecture (The Ring System)

The compiler is structured into a multi-crate workspace, following a unidirectional "Ring" dependency model:

- **Ring 0: Foundation**
  - [`ast`](./rings/ring0/ast): The core "Everything is a Layer" AST. Divided into `types.rs` (atoms) and `lib.rs` (layers).
  - [`lexer`](./rings/ring0/lexer): The tokenizer, supporting bit-precise types (`i32`, `b8`, etc.) and the new keyword set.
- **Ring 1: Parsing**
  - `parser`: (In Development) Transforms tokens into recursive `Layer` trees.
- **Ring 2: Elaboration**
  - [`elaboration`](./rings/ring2/elaboration): Analyzes layers to extract SMT constraints and semantic metadata.
- **Ring 3: Interface**
  - [`command_parser`](./rings/ring3/command_parser): Handles CLI arguments and workspace configuration.
- **Driver**
  - [`burst`](./burst): The main entry point that orchestrates the compilation pipeline.

## 💎 Code Style: PascalCase

Burst's Rust implementation uses **PascalCase** for all identifiers (Functions, Variables, Fields, Structs, Enums) to distinguish the compiler logic from standard Rust library code. Standard Rust warnings are suppressed via `#![allow(non_snake_case)]`.

## 🚀 Current Status

- [x] **PascalCase Refactor**: Complete across all crates.
- [x] **Ring 0 (AST)**: Reorganized into a clean `Layer`-based architecture.
- [x] **Lexer**: Fully operational with support for `var`, `function`, `match`, and `where`.
- [x] **Elaboration**: Basic `Layer`-aware constraint extraction is functional.
- [ ] **Parser**: Under reconstruction to support recursive `Layer` building.

## 🛠️ Building and Testing

Ensure you have the latest Rust toolchain installed.

```powershell
# Check the entire workspace
cargo check

# Run all tests
cargo test
```

## 📖 Documentation

Comprehensive language specifications and design notes are maintained in the [Obsidian Vault](file:///C:/Users/Kyle/Documents/Burst%20Language).
