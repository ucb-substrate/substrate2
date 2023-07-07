---
sidebar_position: 2
---

import CodeSnippet from '@site/src/components/CodeSnippet';
import InverterMod from '@substrate/tests/src/shared/inverter/mod.rs?snippet';
import InverterTb from '@substrate/tests/src/shared/inverter/tb.rs?snippet';
import Context from '@substrate/tests/src/shared/pdk/mod.rs?snippet';
import Tests from '@substrate/tests/src/sim/spectre.rs?snippet';

# Designing an inverter

In this tutorial, we'll design and simulate a schematic-level inverter in
the Skywater 130nm process.

## Setup

Start by creating a new Rust project:
```bash
cargo new --lib sky130_inverter && cd sky130_inverter
```

In your project's `Cargo.toml`, add dependencies on `substrate`, `spectre`, `sky130pdk`, and `serde`:

```toml title="Cargo.toml"
[dependencies]
substrate = { version = "0.0.0", registry = "substrate", path = "../substrate" }
spectre = { version = "0.0.0", registry = "substrate", path = "../tools/spectre" }
sky130pdk = { version = "0.0.0", registry = "substrate", path = "../pdks/sky130pdk" }
serde = { version = "1", features = ["derive"] }

rust_decimal = "1.30"
rust_decimal_macros = "1.30"
```

Let's now add some imports to `src/lib.rs` that we'll use later on:
<CodeSnippet language="rust" title="src/lib.rs" snippet="imports">{InverterMod}</CodeSnippet>

## Interface

We'll first define the interface (also referred to as IO) exposed by our inverter.

The inverter should have four ports:
* `vdd` and `vss` are inout ports.
* `din` is an input.
* `dout` is the inverted output.

This is how that description translates to Substrate:

<CodeSnippet language="rust" title="src/lib.rs" snippet="inverter-io">{InverterMod}</CodeSnippet>

Each `Signal` is a single wire.
The `Input`, `Output`, and `InOut` wrappers provide directions for the `Signal`s they enclose.

The `#[derive(Io)]` attribute tells Substrate that our `InverterIo` struct should be made into a Substrate IO.

## Inverter Parameters

Now that we've defined an Io, we can define a **block**.
Substrate blocks are analogous to modules or cells in other generator frameworks.

While Substrate does not require you to structure your blocks in any particular way,
it is common to define a struct for your block that contains all of its parameters.

We'll make our inverter generator have three parameters:
* An NMOS width.
* A PMOS width.
* A channel length.

We're assuming here that the NMOS and PMOS will have the same length.

In this tutorial, we store all dimensions as integers in layout database units.
In the Sky 130 process, the database unit is a nanometer, so supplying an NMOS width
of 1,200 will produce a transistor with a width of 1.2 microns.

We'll now define the struct representing our inverter:
<CodeSnippet language="rust" title="src/lib.rs" snippet="inverter-struct">{InverterMod}</CodeSnippet>

There are a handful of `#[derive]` attributes that give our struct properties that Substrate requires.
For example, blocks must implement `Eq` so that Substrate can tell if two blocks are equivalent.

## Schematic Generator

We can finally generate a schematic for our inverter.

Describing a Schematic in Substrate requires implementing two traits:
* `HasSchematic` declares that a block has a schematic, but without describing the schematic.
* `HasSchematicImpl` specifies the actual schematic in a particular PDK.

The `HasSchematic` trait allows you to declare a `Data` type for data returned by your block's
schematic generator.
This can be useful if you, for example, want to save a node in your circuit to probe in a simulation later on.
For now, we don't want to save anything, so we'll set `Data` to Rust's unit type, `()`.

Here's how our schematic generator looks:
<CodeSnippet language="rust" title="src/lib.rs" snippet="inverter-schematic">{InverterMod}</CodeSnippet>

The calls to `cell.instantiate(...)` create two sub-blocks: an NMOS and a PMOS.
Note how we pass transistor dimensions to the SKY130-specific `Nfet01v8` and `Pfet01v8` blocks.

The calls to `cell.connect(...)` connect the instantiated transistors to the ports of our inverter.
For example, we connect the drain of the NMOS (`nmos.io().d`) to the inverter output (`io.dout`).

## Testbench

Let's now simulate our inverter and measure the rise and fall times.
We'll use the commercial Spectre simulator.

Start by creating a new file, `src/tb.rs`. Add a reference to this module
in `src/lib.rs`:

```rust title="src/lib.rs"
pub mod tb;
```

Add the following imports to `src/tb.rs`:
<CodeSnippet language="rust" title="src/tb.rs" snippet="imports">{InverterTb}</CodeSnippet>

All Substrate testbenches are blocks that have schematics.
The schematic specifies the simulation structure (i.e. input sources,
the device being tested, etc.).

As a result, creating a testbench is similar to creating a regular block, but with a few differences:
* We don't have to define an IO, since all testbenches must declare their IO to be `TestbenchIo`.
  `TestbenchIo` has one port, `vss`, that allows simulators to identify a global ground (which
  they often assign to node 0).
* Instead of implementing `HasSchematicImpl`, testbenches implement `HasTestbenchSchematicImpl`,
  which declares that a testbench has a particular setup in a given PDK **and** a given simulator.
  This allows testbenches to use simulator-specific blocks, such as PRBS sources in Spectre.
  For example, trying to instantiate a Spectre PRBS source in an Ngspice testbench
  will result in a compile-time error.

Just like regular blocks, testbenches are usually structs containing their parameters.
We'll make our testbench take two parameters:
* A PVT corner.
* An `Inverter` instance to simulate.

Here's how that looks in Rust code:

<CodeSnippet language="rust" title="src/tb.rs" snippet="struct-and-impl">{InverterTb}</CodeSnippet>

The `Pvt<Sky130Corner>` in our testbench is essentially a 3-tuple of a process corner,
voltage, and temperature. The process corner here is an instance of `Sky130Corner`,
which is defined in the `sky130pdk` plugin for Substrate.

Let's now create the schematic for our testbench. This should have three components:
* A pulse input source driving the inverter input.
* A dc voltage source supplying power to the inverter.
* The instance of the inverter itself.

Recall that schematic generators can return data for later use. Here, we'd like to probe
the output node of our inverter, so we'll set `Data` in `HasSchematic` to be of type `Node`.

Here's our testbench setup:

<CodeSnippet language="rust" title="src/tb.rs" snippet="schematic">{InverterTb}</CodeSnippet>

We create two Spectre-specific `Vsource`s (one for VDD; the other as an input stimulus).
We also instantiate our inverter and connect everything up.
The `cell.signal(...)` calls create intermediate nodes.
Creating them isn't strictly necessary (we could connect `inv.io().vdd` directly to `vddsrc.io().p`,
for example), but they can sometimes improve readability of your code and of generated schematics.
Finally, we return the node that we want to probe.

The final thing we must do is describe the data produced by our testbench.
Here, we want to measure 20-80% rise and fall times.

To make our testbench actually a testbench, we must implement the `Testbench` trait.
The `run` method of this trait allows us to configure simulator options (eg. error tolerances)
and set up analyses (AC, DC, transient, etc.).

This is how our testbench looks:

<CodeSnippet language="rust" title="src/tb.rs" snippet="testbench">{InverterTb}</CodeSnippet>

Notice in particular how we obtain the output waveform `vout`
by querying the output using the data we returned from our schematic generator.

We use the `WaveformRef` API to look for 20-80% transitions, and capture their duration.

## Design

Let's use the code we've written to write a script that
automatically sizes our inverter for equal rise and fall times.

We'll assume that we have a fixed NMOS width and channel length and a set
of possible PMOS widths to sweep over.

Here's our implementation:
<CodeSnippet language="rust" title="src/tb.rs" snippet="design">{InverterTb}</CodeSnippet>

We sweep over possible PMOS widths. For each width,
we create a new testbench instance and tell Substrate to simulate it.
We keep track of (and eventually return) the inverter instance that minimizes
the absolute difference between the rise and fall times.

## Running the script

Let's now run the script we wrote. We must first create a Substrate context:

<CodeSnippet language="rust" title="src/tb.rs" snippet="sky130-commercial-ctx">{Context}</CodeSnippet>

We can then run our design script:

```rust title="src/tb.rs"
#[test]
pub fn design_inverter() {
    let test_name = "design_inverter";
    let work_dir = "sims/";
    let mut ctx = sky130_commercial_ctx();
    let script = InverterDesign {
        nw: 1_200,
        pw: (1_200..=5_000).step_by(200).collect(),
        lch: 150,
    };
    let inv = script.run(&mut ctx, work_dir);
    println!("Designed inverter:\n{:#?}", inv);
}
```

To run the test, run

```
cargo test design_inverter -- --show-output
```

Ensure that the `SKY130_COMMERCIAL_PDK_ROOT` environment variable points to your installation of
the Sky 130 commercial PDK.

## Conclusion

If all goes well, the test above should print
the inverter dimensions with the minimum rise/fall time difference.

A full, runnable example for this tutorial is in our [test suite](https://github.com/substrate-labs/substrate2/tree/main/tests/src/shared/inverter).
