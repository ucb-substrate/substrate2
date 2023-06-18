---
sidebar_position: 4
---

# Context

Substrate uses a global context that is passed around as an immutable reference.
The `Context` object stores important runtime configuration such as the process 
in which to generate the block and the tool plugins to be used. It also contains 
a cache of generated blocks and design script results to avoid duplicate computation.

```rust
pub struct Context {
  inner: Arc<RwLock<ContextData>>,
}

pub struct ContextData {
  schematic: SchematicData,
  layout: LayoutData,

  // ... Additional configuration like the PDK, netlister, etc.
}

pub struct SchematicData {
  // Stores `T -> GenerationMap<Param, SchematicCell<T>> where T: HasSchematic`
  // This is fine since if schematic is not found, it will be generated
  // (i.e. we are guaranteed to be able to retrieve it)
  //
  // `GenerationMap` allows for threads to block on another thread generating
  // the same cell instead of doing duplicate computation.
  cells: TypeMap![Send + Sync],

  // Also stores untyped cells for internal processing.
  cells_untyped: GenerationMap<Param, CellId, SchematicCellUntyped>,
}

pub struct LayoutData {
  // Stores `T -> GenerationMap<Param, LayoutCell<T>> where T: HasLayout`
  // This is fine since if layout is not found, it will be generated
  // (i.e. we are guaranteed to be able to retrieve it)
  //
  // `GenerationMap` allows for threads to block on another thread generating
  // the same cell instead of doing duplicate computation.
  cells: TypeMap![Send + Sync],
  
  // Also stores untyped cells for internal processing.
  cells_untyped: GenerationMap<Param, CellId, LayoutCellUntyped>,
}
```

