---
sidebar_position: 1
---

# Overview
At a high level, Substrate generators are called _blocks_ and are defined as any type that implements the `Block` trait.

```rust
pub trait Block: Serialize + Deserialize {
  type Io: AnalogIo;

  fn id() -> ArcStr;

  fn name(&self) -> ArcStr {
    arcstr::literal!("unnamed")
  }

  fn io(&self) -> Self::Io;
}

pub trait HasSchematic: Block {
  type Data;
  fn schematic(&self, cell: &mut SchematicBuilder<Self>) -> Result<Self::Data>;
}

pub trait HasLayout: Block {
  type Data;
  fn layout(&self, cell: &mut LayoutBuilder<Self>) -> Result<Self::Data>;
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

    fn io(&self) -> Self::Io {
        Self::Io {
            p: Default::default(),
            n: Default::default(),
        }
    }
}


impl HasSchematic for VDivider {
  type Data = ();
  fn schematic(&self, cell: &mut SchematicBuilder<Self>) -> SubstrateResult<Self::Data> {
    let r1 = cell.instantiate(Resistor { r: 10 })?;
    let r2 = cell.instantiate(Resistor { r: 20 })?;

    cell.connect(cell.io.vdd, r1.io.p);
    cell.connect_all([cell.io.vout, r1.io.n, r2.io.p);
    cell.connect(cell.io.vss, r2.io.n);

    Ok(())
  }
}
```
