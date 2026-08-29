# FFI and `extern`

LayerScript is a bare-metal language, so talking to C, the OS, and other object files is a first-class concern. The Foreign Function Interface (FFI) is deliberately thin: an `extern` declaration tells the compiler the *shape* of a symbol that lives outside the current compilation, and the linker resolves it later.

---

## 1. Calling C Functions

To call a function defined in C (or any object that exposes a C ABI symbol), declare its signature with `extern` and no body. The compiler emits a relocation instead of code.

```layerscript
// Declares symbols resolved at link time.
extern function printf(format: u8*, ...) -> i32;
extern function malloc(size: usize) -> u8*;
extern function free(ptr: u8*);

function main() {
    var msg: u8* = "hello from layerscript\n";
    printf(msg);
}
```

Variadic parameters use `...` and follow the target's C calling convention exactly — LayerScript performs **no** implicit promotion or boxing across the boundary.

---

## 2. The `extern` Keyword

`extern` may prefix a **function** (import a symbol) or a **block** (group symbols under one convention/library).

```layerscript
extern "C" {
    function open(path: u8*, flags: i32) -> i32;
    function read(fd: i32, buf: u8*, count: usize) -> isize;
    function close(fd: i32) -> i32;
}
```

To *export* a LayerScript function so C can call it, mark it `pub extern` and give it a stable symbol name:

```layerscript
pub extern "C" function ls_add(a: i32, b: i32) -> i32 {
    return a + b;   // callable from C as `ls_add`
}
```

Exported functions have their name **un-mangled** and their layout frozen — the compiler will not reorder parameters or dissolve them into registers beyond what the ABI dictates.

---

## 3. Calling Conventions

The convention string after `extern` selects the ABI. Unspecified means `"C"`.

| String | ABI | Typical use |
| :--- | :--- | :--- |
| `"C"` | Platform C (cdecl / System V / AAPCS) | Default; C libraries. |
| `"stdcall"` | Callee cleans the stack | Legacy Win32 APIs. |
| `"sysv64"` | System V AMD64 | Explicit Unix x86-64. |
| `"naked"` | No prologue/epilogue | Interrupt handlers, trampolines. |

```layerscript
extern "stdcall" function MessageBoxA(hwnd: usize, text: u8*, caption: u8*, kind: u32) -> i32;
```

---

## 4. Safety Considerations

The FFI boundary is where LayerScript's proof machinery ends and raw trust begins. The compiler cannot see the body of an `extern` function, so it cannot verify anything about it.

- **Pointers crossing the boundary lose their refinements.** A `T*` proven in-bounds inside LayerScript is just an address to C. Re-establish invariants on return with a `where` clause or an explicit `panic` guard.
- **`havoc` after foreign writes.** If a C function mutates memory the compiler has cached, mark it: `havoc buffer;` (see [Hardware and Havoc](Hardware%20and%20Havoc.md)). Otherwise trace folding may keep a stale value.
- **Ownership is manual.** Memory from `malloc` is not tracked by any layer; pair every allocation with `free`.
- **Nulls are real.** Foreign code can hand back a null `T*`. Refine it before use: `var p: T* where p != null = get_ptr();`.

> [!WARNING]
> An `extern` call is an **observability boundary**. The compiler assumes it may read or write any memory and touch hardware, so it will not fold, reorder, or delete traces across the call unless you narrow that assumption.

## See also
- [Hardware and Havoc](Hardware%20and%20Havoc.md)
- [Memory Layout and Packing](Memory%20Layout%20and%20Packing.md)
- [Functions and Procedures](../Language%20Specification/Functions%20and%20Procedures.md)
