# LayerScript Core Library Reference (`core`)

this document covers the standard modules and traits that come with layerscript.

---

## 1. the prelude (`core::prelude`)

this is automatically imported. it sets up the basic aliases so you don't have to type bit-widths for everything.

| Alias | Target | Description |
| :--- | :--- | :--- |
| `int` | `i64` | signed integer |
| `uint` | `u64` | unsigned integer |
| `usize` | `u64` | pointer-sized unsigned integer |
| `bool` | `b1` | true/false bits |

---

## 2. core math (`core::math`)

operators like `+` and `*` are just shortcuts for these traits.

```layerscript
namespace core::math;

pub trait Add<T> {
    type Output;
    function add(&self, &T rhs) -> Output;
}

pub trait Sub<T> {
    type Output;
    function sub(&self, &T rhs) -> Output;
}
```

---

## 3. constraint engine (`core::constraint`)

these are the hooks into the SMT solver.

```layerscript
namespace core::constraint;

// check if we can prove something at compile time
pub function is_provable(expr: expression) -> b1;

// force a proof check
pub function verify(expr: expression);
```

---

## 4. the executor (`core::executor`)

controls how the POMSET scheduler handles threads and barriers.

```layerscript
namespace core::executor;

// pin a trace to a specific cpu core
pub function pin_trace(core_mask: usize);

// wait for all previous traces to finish
pub function trace_barrier();
```

---

## 5. hardware (`core::hardware`)

low-level hooks for registers and MMIO.

```layerscript
namespace core::hardware;

pub struct VolatileAddress<T> {
    address: usize;
}

impl<T> VolatileAddress<T> {
    pub function read(&self) -> T;
    pub function write(&self, value: T);
}

// force a reload of all registers
pub function register_flush();
```

---

## 6. memory (`core::memory`)

hooks for the heap and raw pointer math.

```layerscript
namespace core::memory;

pub function raw_alloc(size: usize) -> usize;
pub function raw_free(address: usize);
```

---

## 7. atomics (`core::atomic`)

thread-safe operations that map to CPU instructions.

```layerscript
namespace core::atomic;

pub function atomic_load_u64(target: u64*) -> u64;
pub function atomic_store_u64(target: u64*, value: u64);
pub function atomic_compare_exchange_u64(target: u64*, expected: u64, desired: u64) -> b1;
```
