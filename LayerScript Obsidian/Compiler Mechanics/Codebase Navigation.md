# Codebase Navigation

this is how you find your way around the layerscript compiler source code. everything is built using a **Ring System** where lower rings are basic building blocks and higher rings use them to do complex stuff.

---

## 🏗️ the ring system

we follow a strict rule: higher rings can talk to lower rings, but never the other way around. 

### [Ring 0: Foundation](file:///C:/Users/Kyle/Downloads/Projects/LayerScript/rings/ring0)
- **`ast/`**: the heart of the compiler. 
    - `types.rs`: the "atoms" like `Type` and `Expression`.
    - `lib.rs`: the "molecules" like `Layer` and `LayerKind`.
- **`lexer/`**: the part that reads raw text.
    - `token.rs`: defines what words the compiler understands.
    - `lib.rs`: splits the script into a stream of tokens with line/column tracking.

### [Ring 1: Parsing](file:///C:/Users/Kyle/Downloads/Projects/LayerScript/rings/ring1)
- **`parser/`**: turns the flat list of tokens into a recursive tree of `Layer` objects. this is where the grammar lives.

### [Ring 2: Elaboration](file:///C:/Users/Kyle/Downloads/Projects/LayerScript/rings/ring2)
- **`elaboration/`**: the logic center. it walks the layer tree and extracts SMT-LIB constraints for Z3 to prove things are safe.

### [Ring 3: Interface](file:///C:/Users/Kyle/Downloads/Projects/LayerScript/rings/ring3)
- **`command_parser/`**: handles the command line arguments (`compile`, `eval`, `test`).

### [Driver: LayerScript](file:///C:/Users/Kyle/Downloads/Projects/LayerScript/layerscript)
- the main entry point that ties all the rings together into a single pipeline.

---

## 🔄 how data flows

1. **Text**: you write `.layerscript` code.
2. **Tokens**: the `Lexer` turns text into `Token` objects (with line/column info).
3. **Layers**: the `Parser` builds a `Layer` tree (recursive AST).
4. **Constraints**: the `Elaborator` turns layers into logical propositions (SMT strings).

---

## 🛠️ development tips

- **PascalCase**: all functions, variables, and fields in the rust code use `PascalCase`.
- **Everything is a Layer**: if you add a new feature (like a new loop type), add it to `LayerKind` in Ring 0 first.
- **Fast Checks**: run `cargo check` to make sure your types line up before you try to run the whole thing.
- **Examples**: test your changes against the `.layerscript` files in the `examples/` folder.
