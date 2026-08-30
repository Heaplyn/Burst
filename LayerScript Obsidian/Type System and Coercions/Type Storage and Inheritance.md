# Type Storage and Inheritance

## Type Storage

Each layer contains a `TypeStorage` that holds defined types and aliases. This allows for lexical scoping of types within the layer hierarchy.

```rust
pub struct TypeStorage {
    pub DefinedTypes: HashMap<String, TypeDefinition>,
    pub TypeAliases: HashMap<String, Type>,
}
```

### Type Definitions
A `TypeDefinition` contains the physical layout of the type (Struct, Enum, etc.) along with its metadata and attributes.

```rust
pub struct TypeDefinition {
    pub Name: String,
    pub Kind: TypeKind,
    pub SourceLocation: SourceLocation,
    pub Docs: Option<String>,
}
```

## Variable Storage

Each layer contains a `VariableStorage` that holds variable definitions declared in that scope.

```rust
pub struct VariableStorage {
    pub Variables: HashMap<String, VariableDefinition>,
}
```

### Variable Definitions
A `VariableDefinition` contains the type annotations, mutability, and initial value of variables declared in the scope:

```rust
pub struct VariableDefinition {
    pub Name: String,
    pub TypeAnnotation: Option<Type>,
    pub IsMutable: bool,
    pub Value: Expression,
}
```

## Inheritance Rules

1. **Child inherits parent types**: Types defined in an ancestor layer are visible to all descendants.
2. **Shadowing**: A child layer can define a type with the same name as a parent type, effectively shadowing it within that scope.
3. **Root Environment**: The `Program` layer contains the built-in primitives.

## Type Lookup Order

1. **Current layer** `TypeStorage`.
2. **Parent layers** (recursively up to the root).
3. **Built-in primitives** (i8..i128, u8..u128, f32..f128, b1..b128).

## Built-in Types (Primitive Enum)

In the implementation, primitives are represented by the `BitPrecise` variant of the `Type` enum:

| Variant | Description |
|---------|-------------|
| `BitPrecise('i', N)` | Signed integer of N bits. |
| `BitPrecise('u', N)` | Unsigned integer of N bits. |
| `BitPrecise('f', N)` | Floating point of N bits. |
| `BitPrecise('b', N)` | Raw bit vector of N bits. |
| `Pointer(Box<Type>)` | Raw memory address. |
| `Array(Box<Type>, N)`| Contiguous sequence. |
| `Unit` | Empty type `()`. |

## Custom Type Definitions

```layerscript
// Struct: mapped to TypeKind::Struct
struct player {
    health: f64,
    position: vector3,
}

// Enum: mapped to TypeKind::Enum
enum color {
    red,
    green,
    custom(r: u8, g: u8, b: u8),
}

// Type alias: added to TypeAliases
type score = i32;
```
