---
sidebar_position: 1
---

# Overview

At a high level, Substrate generators are called blocks and are defined as any type that implement the `Block` trait.

```rust
pub trait Block: Serialize + Deserialize {
  type Io: AnalogIo;

  fn id() -> ArcStr;

  fn name(&self) -> ArcStr {
    arcstr::literal!("unnamed")
  }
}

pub trait HasSchematic: Block {
  type Data;
  fn schematic(&self, ctx: &Context) -> Result<SchematicCell<Self>>;
}

pub trait HasLayout: Block {
  type Data;
  fn layout(&self, ctx: &Context) -> Result<LayoutCell<Self>>;
}
```

By implementing a variety of traits such as `HasSchematic` and `HasLayout`, a `Block` can define a set of "views" that can be used to describe the block. The `Context` object stores important runtime configuration such as the process in which to generate the block and the tool plugins to be used. It also contains a cache of generated blocks and design script results to avoid duplicate computation.

## Example 

Here is a brief example illustrating conceptually how a voltage divider should be created using this system:

```rust
pub struct VDivider {
    pub r1: usize,
    pub r2: usize,
}

#[derive(AnalogIo)]
pub struct VDividerIO {
    vdd: Signal<InOut>,
    vout: Signal<Out>,
    vss: Signal<InOut>,
}

impl Block for VDivider {
    type Io = VDividerIO;

    fn id() -> ArcStr {
        arcstr::literal!("vdivider")
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("vdivider_{}_{}", self.r1, self.r2)
    }

    fn io(&self) -> Self::Io
}


impl HasSchematic for VDivider {
  type Data = ();
  fn schematic(&self, ctx: &Context) -> SubstrateResult<SchematicCell> {
    let mut cell = ctx.schematic_cell_builder::<VDivider>();

    let r1 = cell.instantiate("r1", Resistor { r: 10 })?;

    cell.connect(cell.io.vdd, r1.io.p);
    cell.connect(cell.io.vout, r1.io.n);

    let r2: InstancePtr<Resistor> = cell.instantiate("r2", Resistor { r: 20 })?;

    Ok(cell.finish(()))
  }
}
```
