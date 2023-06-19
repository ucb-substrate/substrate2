---
sidebar_position: 5
---

# PDKs

A _process_ is a method of fabrication that determines the layers and devices available to a 
circuit designer. A _process design kit (PDK)_ is a set of utilities for working with
a specific process, such as transistors, layers, standard cells, timing information, simulation
models, DRC rules, etc.

Previous generator frameworks aim to be largely process-portable, but it is close to impossible
to make a (layout) generator fully process-portable except for simple digital circuits like inverters.
Substrate therefore doesn't aim to make complex generators entirely process-portable in the sense that they can
be run with any PDK, but rather makes it easy to specify and extend the PDKs supported by the generator.

## The `Pdk` trait

To do this, the `Context` object is made generic in the PDK:

```rust
pub struct Context<PDK: Pdk> {
    // ... Other context fields.

    pdk: PDK,
}
```

In Substrate, PDKs implement the `Pdk` trait, which allows the PDK to specify common building blocks, process parameters (i.e. layers), standard cells, etc.

:::note
The `Pdk` trait is subject to change, but the below definition is provided to give an idea of the 
components that are expected to be process portable.
:::

```rust
pub trait Pdk {
    type LayerMap;

    fn id() -> ArcStr;

    fn mos_db() -> MosDb;

    fn stdcell_db() -> StdcellDb;

    fn resistor_db() -> ResistorDb;

    fn capacitor_db() -> CapacitorDb;

    fn inductor_db() -> InductorDb;

    fn layers() -> Self::LayerMap;
}
```

The `HasSchematic` and `HasLayout` traits are also made generic, allowing users to specify which processes they support. This allows process-specific generators to use specific layers within a PDK. Process-portable generators can use a blanket implementation of these traits.

## Examples

### Process-portable generators

Here is an example of an entirely process-portable schematic generator for a voltage divider:

```rust
impl<T: Pdk> HasSchematic<T> for VDivider {
  type Data = ();
  fn schematic(&self, cell: &mut SchematicBuilder<T, VDivider>) -> SubstrateResult<Self::Data> {
    // NOTE: `resistor_db` semantics are not yet fleshed out
    let r1 = cell.instantiate("r1", ctx.resistor_db().resistance(self.r1).get());
    let r2 = cell.instantiate("r2", ctx.resistor_db().resistance(self.r2).get());

    cell.connect(cell.io.vdd, r1.io.p);
    cell.connect_all([cell.io.vout, r1.io.n, r2.io.p);
    cell.connect(cell.io.vss, r2.io.n);

    Ok(())
  }
}
```

### Process-specific generators

More complex generators, especially layout generators, will need to be process specific. For example, an SRAM bitcell array for Sky130 would need to look something like this:


```rust
impl HasSchematic<Sky130Pdk> for SramSpCellArray {
  type Data = ();
  fn schematic(&self, ctx: &mut SchematicBuilder<Sky130Pdk, SramSpCellArray>) 
        -> SubstrateResult<Self::Data> {

    for i in 0..self.rows {
        for j in 0..self.cols {
            cell.instantiate_and_connect(
                format!("sp_cell_{i}_{j}"),
                Sky130SpCell,
                Sky130SpCellIO {
                    vdd: cell.io.vdd,
                    vss: cell.io.vss,
                    bl: cell.io.bl[j],
                    br: cell.io.br[j],
                    wl: cell.io.wl[i],
                }
            );
        }
    }

    Ok(())
  }
}
```

### Multi-process generators

To extend this to another PDK, we could copy the implementation and replace Sky130 specifics with the desired PDK:

```rust
impl HasSchematic<Intel16Pdk> for SramSpCellArray {
  type Data = ();
  fn schematic(&self, ctx: &mut SchematicBuilder<Intel16Pdk, SramSpCellArray>)
        -> SubstrateResult<Self::Data> {

    for i in 0..self.rows {
        for j in 0..self.cols {
            cell.instantiate_and_connect(
                format!("sp_cell_{i}_{j}"),
                Intel16SpCell,
                Intel16SpCellIO {
                    vdd: cell.io.vdd,
                    vss: cell.io.vss,
                    bl: cell.io.bl[j],
                    br: cell.io.br[j],
                    wl: cell.io.wl[i],
                }
            );
        }
    }

    Ok(())
  }
}
```

Since we now have schematic implementations for both PDKs, `SramSpCellArray` can be instantiated in either PDK.

However, this results in a lot of code duplication. The intended method would be to create a `SpCell` block that dispatches the schematic instantiation to the appropriate process-specific cell:

```rust
impl HasSchematic<Sky130Pdk> for SramSpCell {
  type Data = ();

  fn schematic(&self, cell: &mut SchematicBuilder<Sky130Pdk, SramSpCell>)
        -> SubstrateResult<Self::Data> {

    cell.instantiate_and_connect(
        format!("sp_cell_{i}_{j}"),
        Sky130SpCell,
        cell.io.flatten().unflatten()
    );


    Ok(cell.finish(()))
  }

  // Does not show this cell in the hierarchy during SPICE export.
  fn flatten_in_hierarchy(&self) -> bool {
    true
  }
}

impl HasSchematic<Intel16Pdk> for SramSpCell {
  type Data = ();

  fn schematic(&self, cell: &mut SchematicBuilder<Intel16Pdk, SramSpCell>) 
        -> SubstrateResult<Self::Data> {

    cell.instantiate_and_connect(
        format!("sp_cell_{i}_{j}"),
        Intel16SpCell,
        cell.io.flatten().unflatten()
    );


    Ok(())
  }

  // Does not show this cell in the hierarchy during SPICE export.
  fn flatten_in_hierarchy(&self) -> bool {
    true
  }
}

#[impl_pdks(Sky130Pdk, Intel16Pdk)]
impl<T> HasSchematic<T> for SramSpCellArray {
  type Data = ();
  fn schematic(&self, ctx: &mut SchematicBuilder<T, SramSpCellArray>) 
        -> SubstrateResult<Self::Data> {

    let mut cell = ctx.schematic_cell_builder::<SramSpCellArray>();

    for i in 0..self.rows {
        for j in 0..self.cols {
            cell.instantiate_and_connect(
                format!("sp_cell_{i}_{j}"),
                SramSpCell, 
                SramSpCellIO {
                    vdd: cell.io.vdd,
                    vss: cell.io.vss,
                    bl: cell.io.bl[j],
                    br: cell.io.br[j],
                    wl: cell.io.wl[i],
                }
            );
        }
    }


    Ok(())
  }
}
```

While the code is now a bit more verbose, this clearly encodes the portions of the design 
that are common between PDKs and the portions that are not. It should also be doable to 
use macros to simplify the implementation of `SramSpCell`. Another benefit is that the 
hierarchy of design is easy to follow, especially if the `SpCell` block is used elsewhere.
