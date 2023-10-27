---
sidebar_position: 1
---

# Creating your first block

A **block** is the essential element of hierarchy in Substrate.

Before we can begin creating Substrate blocks, however, we need to set up a Substrate context.

## Contexts

A Substrate **context** stores all information relevant to Substrate, including
the tools you've set up, the current PDK, all blocks that have been generated,
cached computations, and more.

Let's set up a default context:

```rust
use substrate::context::Context;

fn main() {
    let mut ctx = Context::new();
}
```

## Blocks

Now that we have a context, let's create a block representing an inverter.

The most important part of a block definition is its `Io`.
The `Io` specifies the ports of your block, along with their directions.

In addition to specifying the `Io`, we also need to give our block
a unique ID and a name. The ID must be unique and constant; it is used by Substrate
to look up blocks in a given crate. The block name is what shows up
in generated artifacts. Substrate will automatically resolve duplicate
block names for you.

Here's how we create our inverter block:

```rust
use serde::{Serialize, Deserialize};
use substrate::block::{self, Block};
use sky130pdk::mos::*;
use substrate::io::*;
use substrate::schematic::*;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Inverter {
    pub nmos: MosParams,
    pub pmos: MosParams,
}

#[derive(Debug, Default, Clone, Io)]
pub struct InverterIo {
    pub input: Input<Signal>,
    pub output: Output<Signal>,
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
}

impl Block for Inverter {
    type Io = InverterIo;
    type Kind = block::Cell;

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}
```

Our inverter block is parameterized by an nmos and pmos `MosParams` object.
The `MosParams` struct simply stores a transistor size (length, width, and number of fingers).

Now that we've defined the `Io` of our block, we can define a schematic.

## Schematics

We first implement the `ExportsSchematicData` trait on `Inverter`.
This trait declares that `Inverter` can generate schematic data.
This allows us to do things like save nodes we might want to probe in simulation, but for
now, we don't have any particular data we'd like to save, so we'll set `Data` to Rust's
empty unit type.

```rust
// hidden-rust-doc-start
use serde::{Serialize, Deserialize};
use substrate::block::{self, Block};
use sky130pdk::mos::MosParams;
use substrate::io::*;
use substrate::schematic::*;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Inverter {
    pub nmos: MosParams,
    pub pmos: MosParams,
}

#[derive(Debug, Default, Clone, Io)]
pub struct InverterIo {
    pub input: Input<Signal>,
    pub output: Output<Signal>,
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
}

impl Block for Inverter {
    type Io = InverterIo;
    type Kind = block::Cell;

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}
// hidden-rust-doc-end
impl ExportsNestedData for Inverter {
    type NestedData = ();
}
```

We now specify the actual content of the schematic
by implementing `Schematic`. This trait requires us to specify
the PDK for which the schematic is valid. In this case, the schematic
is for the `Sky130Pdk`.

```rust
// hidden-rust-doc-start
use serde::{Serialize, Deserialize};
use substrate::block::{self, Block};
use sky130pdk::mos::{Nfet01v8, Pfet01v8, MosParams};
use sky130pdk::Sky130Pdk;
use substrate::io::*;
use substrate::schematic::*;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Inverter {
    pub nmos: MosParams,
    pub pmos: MosParams,
}

#[derive(Debug, Default, Clone, Io)]
pub struct InverterIo {
    pub input: Input<Signal>,
    pub output: Output<Signal>,
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
}

impl Block for Inverter {
    type Io = InverterIo;
    type Kind = block::Cell;

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for Inverter {
    type NestedData = ();
}
// hidden-rust-doc-end
impl Schematic<Sky130Pdk> for Inverter {
    fn schematic(
        &self,
        io: &InverterIoSchematic,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        let nmos = cell.instantiate(Nfet01v8::new(self.nmos));
        let nmos = nmos.io();
        cell.connect(nmos.d, io.output);
        cell.connect(nmos.g, io.input);
        cell.connect(nmos.s, io.vss);
        cell.connect(nmos.b, io.vss);

        let pmos = cell.instantiate(Pfet01v8::new(self.pmos));
        let pmos = pmos.io();
        cell.connect(pmos.d, io.output);
        cell.connect(pmos.g, io.input);
        cell.connect(pmos.s, io.vdd);
        cell.connect(pmos.b, io.vdd);
        Ok(())
    }
}
```

## Export

We can now export the schematic of our inverter to Substrate's schematic IR, SCIR:

```rust
// hidden-rust-doc-start
use serde::{Serialize, Deserialize};
use substrate::block::{self, Block};
use sky130pdk::mos::{Nfet01v8, Pfet01v8, MosParams};
use sky130pdk::Sky130Pdk;
use substrate::io::*;
use substrate::schematic::*;
use substrate::context::Context;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Inverter {
    pub nmos: MosParams,
    pub pmos: MosParams,
}

#[derive(Debug, Default, Clone, Io)]
pub struct InverterIo {
    pub input: Input<Signal>,
    pub output: Output<Signal>,
    pub vdd: InOut<Signal>,
    pub vss: InOut<Signal>,
}

impl Block for Inverter {
    type Io = InverterIo;
    type Kind = block::Cell;

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for Inverter {
    type NestedData = ();
}

impl Schematic<Sky130Pdk> for Inverter {
    fn schematic(
        &self,
        io: &InverterIoSchematic,
        cell: &mut CellBuilder<Sky130Pdk>,
    ) -> substrate::error::Result<Self::NestedData> {
        let nmos = cell.instantiate(Nfet01v8::new(self.nmos));
        let nmos = nmos.io();
        cell.connect(nmos.d, io.output);
        cell.connect(nmos.g, io.input);
        cell.connect(nmos.s, io.vss);
        cell.connect(nmos.b, io.vss);

        let pmos = cell.instantiate(Pfet01v8::new(self.pmos));
        let pmos = pmos.io();
        cell.connect(pmos.d, io.output);
        cell.connect(pmos.g, io.input);
        cell.connect(pmos.s, io.vdd);
        cell.connect(pmos.b, io.vdd);
        Ok(())
    }
}
// hidden-rust-doc-end
fn main() {
    let mut ctx = Context::new();
    let inv = Inverter {
        nmos: MosParams { w: 1_000, l: 150, nf: 1 },
        pmos: MosParams { w: 2_000, l: 150, nf: 1 },
    };
    let lib = ctx.export_scir::<Sky130Pdk, _>(inv);
}
```

The resulting SCIR library can then be exported as a Spice/Spectre/Verilog netlist,
depending on which netlisters you have available.
