# Modules and Namespaces

In LayerScript, code organization and encapsulation follow the universal rule: **everything is a layer**. Consequently, modules, packages, and namespaces are not merely compile-time namespaces—they are represented as structural parent layers that encapsulate child layers.

---

## 1. Defining Modules

Modules can be declared explicitly using the `module` keyword or implicitly via the file-system structure.

### Explicit Modules
You can group related definitions (structs, functions, variables) inside an explicit module block:

```layerscript
module Network {
    pub struct Header {
        length: u16,
        protocol: b8,
    }

    pub function parse(packet: b8*) {
        // parser implementation...
    }
}
```

### Implicit (File-based) Modules
Every `.layerscript` file automatically acts as a module named after the file. For example, a file named `math.layerscript` implicitly defines the `math` module.

---

## 2. Visibility and Exporting (`pub`)

By default, all children inside a layer are **private** to that layer and cannot be referenced or accessed from outer scopes. To make a child layer visible to parent or sibling layers, use the `pub` keyword.

```layerscript
module Controller {
    var state: u32 = 0; // Private: Outer scopes cannot read or modify this directly

    pub function get_state() -> u32 {
        return state; // Public: Accessible outside the Controller module
    }
}
```

---

## 3. Importing Namespaces (`use` and `import`)

To reference declarations from other modules, use the `import` or `use` statements.

### The `import` Statement
Brings a module or submodule into scope:

```layerscript
import Network;

function main() {
    var hdr: Network.Header; // Referenced via namespace
}
```

### The `use` Statement
Brings specific definitions directly into the local layer scope, removing the need for namespace prefixing:

```layerscript
use Network.Header;
use Network.parse;

function main() {
    var hdr: Header;
    parse(&hdr as b8*);
}
```

To import all public elements from a module, use the wildcard operator `*`:
```layerscript
use Network.*;
```

---

## 4. Encapsulation & Observability Boundaries

Because modules are layers, they help define **observability boundaries**:
- Internal mutable variables (like `var state`) are contained within the module layer.
- If no `pub` interfaces expose a private variable, and the variable does not write to a hardware register or memory-mapped I/O, the SMT solver is free to fold, optimize, or entirely compile away that variable's operations.
