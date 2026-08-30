# Layer System

## Everything is a Layer

LayerScript organizes code into **layers**. Every construct (program, function, block, variable, hook) is a layer. In the implementation, this is represented by the `Layer` struct in `ring0/ast`.

```layerscript
// Root layer: the entire script
struct point { x: f64, y: f64 }  // This is a layer

function distance(p: point, q: point) -> f64 {  // Function layer
    var dx = p.x - q.x;  // Variable layer
    var dy = p.y - q.y;  // Variable layer
    return (dx * dx + dy * dy).sqrt();  // Return layer
}
```

## Layer Implementation (PascalCase)

The `Layer` structure is universal. It contains metadata, constraints, and child layers.

```rust
pub struct Layer {
    pub Id: LayerId,
    pub Kind: LayerKind,
    pub Metadata: LayerMetadata,
    pub Children: Vec<Layer>,
    pub Constraints: Vec<Constraint>,
    pub Observability: ObservabilityFlags,
    pub TypeStorage: TypeStorage,
    pub VariableStorage: VariableStorage,
    pub TraceInfo: TraceInfo,
}
```

### Layer Kinds
The `LayerKind` enum defines what a layer represents:
*   `Program`: The top-level container.
*   `Function`: Parameters, return types, and safety flags.
*   `VariableBinding`: Type annotations, mutability, and lifecycle hooks.
*   `Block`: A lexical scope.
*   `Loop`: Iterative control flow.
*   `Panic` / `Unreachable`: Safety intrinsics.

## LayerBuilder

Creating layers is done via the `LayerBuilder` to ensure all metadata (like source location) is correctly initialized.

```rust
let Root = LayerBuilder::New(LayerKind::Program, SourceLoc)
    .WithDoc("Main entry".to_string())
    .WithChild(FunctionLayer)
    .Build();
```

## layertrace - Runtime Trace

The `TraceInfo` within each layer provides the data needed for the `layertrace` runtime manager to perform introspection.

```layerscript
function main() {
    // Get current layer
    var current = layertrace.current();
    
    // Look up a type anywhere in the hierarchy
    var type_info = layertrace.lookup_type("Point");
}
```
