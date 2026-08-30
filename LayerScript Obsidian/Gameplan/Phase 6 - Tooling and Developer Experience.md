# Phase 6 — Tooling & Developer Experience

> **Goal:** Make LayerScript pleasant to use — a full CLI, editor support, project management, and documentation.
> **Owns crates:** [`rings/ring3/command_parser`](../../rings/ring3/command_parser/src/lib.rs) + the [`layerscript`](../../layerscript/src/main.rs) driver.
> **Milestone:** `layerscript --help` lists all commands and they work.

---

## Current state

The CLI is a working scaffold built on `clap` (see [CLI Reference](../API%20and%20Standard%20Library/CLI%20Reference.md)):

- Global flags `--verbose/-v`, `--workspace/-w`.
- `compile <FILE> [-O]`, `eval <CODE>`, `test [--filter]`.
- [`main.rs`](../../layerscript/src/main.rs) `RunPipeline` wires lex → parse → elaborate → run; `compile` reads a file, `eval` runs inline, `test` is a stub.

**Gaps:** no `build`/`run`/`fmt`/`init`/`check`/`doc` subcommands; no colored output or progress; `test` doesn't discover or run tests; `config` crate is a lone `pub static Verbose: bool = true` and should hold real settings.

## 6.1 CLI improvements

- [ ] `layerscript build` (→ binary), `run` (execute), `check` (type-check only), `fmt`, `init`, `doc`.
- [ ] Promote `code_runner`'s pipeline into named subcommands; unify with `-O` levels.
- [ ] Colored diagnostics + progress indicators.

## 6.2 Editor support

- [ ] LSP server (reuse the parser/elaborator as a library).
- [ ] VS Code extension: syntax highlighting, autocomplete, hover, go-to-def, find-refs, rename, inline type hints, error underlines. (A `.vscode/rust.code-snippets` already exists in the repo for the compiler's own dev.)

## 6.3 Project management

- [ ] `LayerScript.toml` manifest, dependency + module resolution ([Modules and Namespaces](../Language%20Specification/Modules%20and%20Namespaces.md)), build profiles (dev/release), cross-compilation.

## 6.4 Documentation

- [ ] Keep this vault as the language reference; generate stdlib/API docs from source.
- [ ] Contributing guide + architecture diagram (the [ring architecture](../Compiler%20Mechanics/Codebase%20Navigation.md) in the README is the seed).

---

## Acceptance criteria

- `layerscript build examples/hello_world.layerscript` produces a runnable artifact.
- `layerscript check` reports type/refinement errors without running.

## See also
- [[Phase 5 - Standard Library and Runtime]] · [[Phase 7 - Advanced Features]] · [CLI Reference](../API%20and%20Standard%20Library/CLI%20Reference.md)
