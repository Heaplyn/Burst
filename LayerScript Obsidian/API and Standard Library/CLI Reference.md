# Command Line Interface (CLI) Reference

The LayerScript compiler is controlled via a command-line interface driver named `layerscript` (formerly `burst`). The CLI provides commands to compile files, evaluate inline snippets, and execute project tests.

---

## 1. Global Flags

These flags can be applied globally to all subcommands:

- `-v, --verbose`  
  Enables verbose log outputs, displaying lexical token lists, detailed AST tree outputs, and active SMT constraints sent to the solver.
- `-w, --workspace <DIR>`  
  Specifies the root directory of the LayerScript workspace. Defaults to the current working directory.
- `-h, --help`  
  Prints help information and available command options.
- `-V, --version`  
  Prints the active version of the LayerScript compiler.

---

## 2. Subcommands

### `compile` (or `build`)
Compiles a LayerScript source file, extracts SMT constraints, verifies them against Z3, and generates output targets.

```powershell
layerscript compile <FILE> [OPTIONS]
```

**Options:**
- `-O <LEVEL>`  
  Specify the optimization level.
  * `-O0`: No optimizations. Emits all checks as runtime panics.
  * `-O1`: Basic optimization. Folds simple constants.
  * `-O2` (Default): Standard optimization. Runs path optimizations, SMT proof erasure, and registers trace observability folding.
  * `-O3`: Aggressive optimization. Aggressive loop folding, register pressure optimizations, and automatic parallelization via POMSET.

**Example:**
```powershell
layerscript compile examples/refinement.layerscript -O2
```

---

### `eval`
Evaluates a raw inline LayerScript code string directly in the shell using the interpreter engine. Very useful for testing logic and verifying how a snippet tokenizes or parses.

```powershell
layerscript eval "<CODE_STRING>" [OPTIONS]
```

**Example:**
```powershell
layerscript eval "fn main() { var x = 10; print(x); }" --verbose
```

---

### `test`
Discovers and runs test procedures inside the workspace.

```powershell
layerscript test [OPTIONS]
```

**Options:**
- `-f, --filter <PATTERN>`  
  Only executes tests matching the specified string pattern.

**Example:**
```powershell
layerscript test --filter "parser"
```
