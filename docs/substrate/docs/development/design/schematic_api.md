---
sidebar_position: 2
---

# Schematic API

Making a schematic involves creating a cell, instantiating instances within it, and 
connecting those instances to one another and to the top-level IO.

Specifically, creating a voltage divider schematic looks something like this:

```rust
impl HasSchematic for VDivider {
  type Data = VDividerData;
  fn schematic(&self, ctx: &Context) -> SubstrateResult<SchematicCell> {
    let mut cell = ctx.schematic_cell_builder::<VDivider>();

    let r1: InstancePtr<Resistor> = cell.instantiate("r1", Resistor { r: 10 });

    cell.connect(cell.io.vdd, r1.io.p);
    cell.connect(cell.io.vout, r1.io.n);

    let r2: InstancePtr<Resistor> = cell.instantiate_and_connect(
        "r2",
        Resistor { r: 20 },
        Wire<ResistorIo> {
          p: r1.io.n,
          n: cell.io.vss,
        }
    );

    Ok(cell.finish(VDividerData { r1, r2 }))
  }
}
```

## Cells

Schematic cells consist of a set of instances, their connections, and extra 
data defined by the user that is used to access instances and other computation results from other views or cells.

```rust
// Basically just `SchematicCell` without the `data` field
pub struct SchematicCellBuilder<T> {
  // ... Fields relevant to schematic storage (i.e. instances and connections)
  block: T,
  io: Wire<Flipped<T::Io>>,
}

pub struct SchematicCell<T> {
  // ... Fields relevant to schematic storage (i.e. instances and connections)
  block: T,
  data: T::Data,
  io: Wire<Flipped<T::Io>>,
}
```

The `SchematicCellBuilder` is necessary since the user-defined `T::Data` may not be 
constructable until after the schematic is constructed and all the instances have been 
created. Hence, the data is specified with `SchematicCellBuilder::finish`. 

This also helps since the cell should not be modifiable once it is returned from 
the `schematic` function of a block.

The `block` field is used to store the parameters used to create a cell, in case this needs to be accessed for layout/schematic matching.

### `SchematicCellBuilder`

Calling `cell.instantiate::<T>` creates an instance, initializes its interface by assigning its ports unique IDs, adds it to the cell, and returns an `SchematicInstancePtr<T>` (explained in more detail in the [Instances](#instances) section).

Note that because an instance needs to have a newly initialized IO for it to be meaningful, we do not want the cell to allow adding instances outside of calling `cell.instantiate::<T>`.

:::info
Parallelizing several `cell.instantiate` calls within a single `schematic` function is a bit difficult,
so instantiation is serialized by default. `SchematicCellBuilder` should support `cell.instantiate_all`,
which generates several instances in parallel.

A macro can be used to make `cell.instantiate_all` generic across various tuple sizes.
:::


### Nested node access

During simulation, it is often useful to access nested nodes to tell the simulator which nodes to save. 
To make this simulator portable, it is best to be able to recover the path to a desired node, 
rather than access the node via a string.

Specifically, we want to be able to do something like this by utilizing our saved `T::Data`:

```rust
let cell = ctx.get_schematic_cell::<VDivider>();
ctx.save_node(cell.data.r1.io.p);
```

When saving to spice, we would need to write out the full path `"r1.p"`, which means we need to be able to recover the full path from only the struct that `cell.data.r1.io.p` represents (let's call this `Port` for the sake of explanation, though the name is subject to change).

Assuming that instances are assigned context-unique IDs, recovering the path to a node with only the node itself is doable by storing parent pointers in ports and instances:

```rust
pub struct Port {
   id: PortId, // For looking up connectivity information in the containing cell.
   parent: InstanceId, // Context-unique instance ID
}

// Generic instance
pub struct Instance {
    // ... Other fields for storing instance information

    // Context-unique instance ID, `None` if the parent is an uninstantiated cell
    parent: Option<InstanceId>, 
}
```

By traversing the parent pointers, we can recover the path to a node from only the 
`Port` struct contained at `cell.data.r1.io.p`.


## Instances

Instances are instantiated cells, storing a unique name (checked at runtime) and a pointer to its underlying
cell. It also needs to store an IO instantiated by its parent cell since different instances with the same underlying cell should have unique ports.

```rust
pub struct SchematicInstance<T> {
    name: ArcStr,
    io: T::Io,
    cell: Arc<SchematicCell<T>>,
}

pub struct SchematicInstancePtr<T>(SchematicInstance<T>);

impl<T> Deref for SchematicInstancePtr<T>{
    type Target = SchematicInstance<T>;

    // ...
}
```

`SchematicInstancePtr` is required to make it clear that cloning something returned by 
`cell.instantiate::<T>` does not clone the underlying instance (i.e. connecting the cloned 
ports will connect the ports of the original instance). Using this strategy works because 
schematic instances do not need to be mutated by the user.
