# layertrace Runtime

## Overview

`layertrace` is the global runtime manager that provides layer introspection and type resolution.

## API Reference

### current()
Returns the currently executing layer.

```layerscript
var current_layer = layertrace.current();
```

### root()
Returns the root layer (entire script).

```layerscript
var root_layer = layertrace.root();
```

### push(id)
Pushes a new layer onto the stack.

```layerscript
layertrace.push("compute_loop");
// ... code ...
layertrace.pop();
```

### pop()
Pops the current layer from the stack.

```layerscript
layertrace.pop();
```

### lookup_type(name)
Finds a type by name in the layer hierarchy.

```layerscript
var point_type = layertrace.lookup_type("Point");
```

### get_metadata(key)
Gets metadata from the current layer.

```layerscript
var doc = layertrace.get_metadata("doc");
var optimization = layertrace.get_metadata("optimization");
```

## Type Environment

`layertrace` maintains the current type environment.

```layerscript
layertrace.current().type_env  // Access type storage
```

## Runtime Type Information

```layerscript
function print_types() {
    var all_types = layertrace.root().get_all_visible_types();
    for (var ty in all_types) {
        trace!("Type: {}", ty.name);
    }
}
```