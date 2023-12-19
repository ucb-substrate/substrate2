---
sidebar_position: 2
---

import CodeSnippet from '@site/src/components/CodeSnippet';
import InverterMod from '@substrate/examples/sky130_inverter/src/lib.rs?snippet';
import InverterTb from '@substrate/examples/sky130_inverter/src/tb.rs?snippet';
import CargoToml from '@substrate/examples/sky130_inverter/Cargo.toml?snippet';

# Designing an inverter

In this tutorial, we'll design and simulate a schematic-level inverter in
the Skywater 130nm process to showcase some of the capabilities of Substrate's
analog simulation interface. We'll also go into some more detail about what
the code you're writing is actually doing.

## Setup

### Rust

Ensure that you have a recent version of Rust installed.
Add the Substrate registry to your Cargo config:

```toml title="~/.cargo/config.toml"
[registries]
substrate = { index = "https://github.com/substrate-labs/crates-index" }
```

You only need to do this the first time you set up Substrate.

Next, create a new Rust project:
```bash
cargo new --lib sky130_inverter && cd sky130_inverter
```

In your project's `Cargo.toml`, add the following dependencies:

<CodeSnippet language="toml" title="Cargo.toml" snippet="dependencies">{CargoToml}</CodeSnippet>

Let's now add some imports that we'll use later on.
Replace the content of `src/lib.rs` with the following:

<CodeSnippet language="rust" title="src/lib.rs" snippet="imports">{InverterMod}</CodeSnippet>

### Simulators

This tutorial will demonstrate how to invoke both [ngspice](https://ngspice.sourceforge.io/index.html) and [Spectre](https://www.cadence.com/en_US/home/tools/custom-ic-analog-rf-design/circuit-simulation/spectre-ams-designer.html) from Substrate to run transient simulations.
You can choose to use whichever simulator you would like, but make sure to install the appropriate simulator before
running your Rust code. We recommend an ngspice version of at least 41.

### SKY130 PDK

If you would like to simulate using ngspice, you will need to install the open source PDK by cloning the [`skywater-pdk` repo](https://github.com/ucb-substrate/skywater-pdk) and pulling the relevant libraries:

```
git clone https://github.com/ucb-substrate/skywater-pdk.git && cd skywater-pdk
git submodule update --init libraries/sky130_fd_pr/latest
```

Also, ensure that the `SKY130_OPEN_PDK_ROOT` environment variable points to the location of the repo you just cloned.

If you would like to use Spectre, you will also need to ensure that the `SKY130_COMMERCIAL_PDK_ROOT` environment variable points to an installation of the commercial SKY130 PDK.

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

## Inverter parameters

Now that we've defined an IO, we can define a **block**.
Substrate blocks are analogous to modules or cells in other generator frameworks.

While Substrate does not require you to structure your blocks in any particular way,
it is common to define a struct for your block that contains all of its parameters.

We'll make our inverter generator have three parameters:
* An NMOS width.
* A PMOS width.
* A channel length.

We're assuming here that the NMOS and PMOS will have the same length.

In this tutorial, we store all dimensions as integers in layout database units.
In the SKY130 process, the database unit is a nanometer, so supplying an NMOS width
of 1,200 will produce a transistor with a width of 1.2 microns.

We'll now define the struct representing our inverter:
<CodeSnippet language="rust" title="src/lib.rs" snippet="inverter-struct">{InverterMod}</CodeSnippet>

There are a handful of `#[derive]` attributes that give our struct properties that Substrate requires.
For example, blocks must implement `Eq` so that Substrate can tell if two blocks are equivalent. It is important
that `Eq` is implemented in a way that makes sense as Substrate uses it to determine if a block can be reused
or needs to be regenerated.

## Schematic Generator

We can now generate a schematic for our inverter.

Describing a Schematic in Substrate requires implementing two traits:
* `ExportsNestedData` declares what nested data a block exposes, such as internal nodes or instances. For now, we don't want to expose anything, so we'll set the associated `NestedData` type to Rust's unit type, `()`.
* `Schematic` specifies the actual schematic in a particular **schema**. A schema is essentially just a format for representing a schematic. In this case, we want to use the `Sky130Pdk` schema as our inverter should be usable in any block generated in SKY130.

Here's how our schematic generator looks:
<CodeSnippet language="rust" title="src/lib.rs" snippet="inverter-schematic">{InverterMod}</CodeSnippet>

The calls to `cell.instantiate(...)` create two sub-blocks: an NMOS and a PMOS.
Note how we pass transistor dimensions to the SKY130-specific `Nfet01v8` and `Pfet01v8` blocks.

The calls to `cell.connect(...)` connect the instantiated transistors to the ports of our inverter.
For example, we connect the drain of the NMOS (`nmos.io().d`) to the inverter output (`io.dout`).

## Testbench

Let's now simulate our inverter and measure the rise and fall times.
For now, we'll use ngspice as our simulator. Later, we'll add support for Spectre.

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

As a result, creating a testbench is the same as creating a regular block except that we don't have to define an IO.
All testbenches must declare their IO to be `TestbenchIo`, which has one port, `vss`, that allows 
simulators to identify a global ground (which they often assign to node 0).

Just like regular blocks, testbenches are usually structs containing their parameters.
We'll make our testbench take two parameters:
* A PVT corner.
* An `Inverter` instance to simulate.

Here's how that looks in Rust code:

<CodeSnippet language="rust" title="src/tb.rs" snippet="struct-and-impl">{InverterTb}</CodeSnippet>

The `Pvt<Sky130Corner>` in our testbench is essentially a 3-tuple of a process corner,
voltage, and temperature. The process corner here is an instance of `Sky130Corner`,
which is defined in the `sky130pdk` plugin for Substrate.

Let's now create the schematic for our testbench. We will do this in the `Ngspice` schema so that the ngspice simulator plugin knows how to netlist and simulate our testbench. This should have three components:
* A pulse input source driving the inverter input.
* A dc voltage source supplying power to the inverter.
* The instance of the inverter itself.

Recall that schematic generators can return data for later use. Here, we'd like to probe
the output node of our inverter, so we'll set `Data` in `HasSchematicData` to be of type `Node`.

Here's our testbench setup:

<CodeSnippet language="rust" title="src/tb.rs" snippet="schematic">{InverterTb}</CodeSnippet>

We create two Spectre-specific `Vsource`s (one for VDD, the other as an input stimulus).
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

We define `Vout` as a receiver for data saved during simulation. We then tell Substrate what data we want to save from
our testbench by implementing the `SaveTb` trait.

## Design

Let's use the code we've written to write a script that
automatically sizes our inverter for equal rise and fall times.

We'll assume that we have a fixed NMOS width and channel length and a set
of possible PMOS widths to sweep over.

Here's our implementation:
<CodeSnippet language="rust" title="src/tb.rs" snippet="design">{InverterTb}</CodeSnippet>

We sweep over possible PMOS widths. For each width,
we create a new testbench instance and tell Substrate to simulate it.
We use the `WaveformRef` API to look for 20-80% transitions, and capture their duration.
Finally, we keep track of (and eventually return) the inverter instance that minimizes
the absolute difference between the rise and fall times.

You may also notice that the `run` function is generic over the simulator `S`, requiring only that
the `InverterTb` implements `Testbench` and yields `Vout` as an output. This allows to support additional
simulators simply by implementing `Testbench` for each simulator we would like to support.

## Running the script

Let's now run the script we wrote. We must first create a Substrate **context** that stores all information 
relevant to Substrate. This includes
the tools you've set up, the current PDK, all blocks that have been generated,
cached computations, and more.

<CodeSnippet language="rust" title="src/tb.rs" snippet="sky130-open-ctx">{InverterTb}</CodeSnippet>

We can then write a Rust unit test to run our design script:

<CodeSnippet language="rust" title="src/tb.rs" snippet="tests">{InverterTb}</CodeSnippet>

To run the test, run

```
cargo test design_inverter_ngspice -- --show-output
```

If all goes well, the test above should print
the inverter dimensions with the minimum rise/fall time difference.

## Adding Spectre support
Because we designed in multi-simulator support from the beginning, adding Spectre support is simply a matter
of defining a Spectre-specific testbench schematic, running the appropriate Spectre simulation, and 
returning the data in the appropriate format.

To add Spectre support, we can simply add the following code:

<CodeSnippet language="rust" title="src/tb.rs" snippet="spectre-support">{InverterTb}</CodeSnippet>

Before running the new Spectre test, ensure that the `SKY130_COMMERCIAL_PDK_ROOT` environment variable points to your installation of
the SKY130 commercial PDK.
Also ensure that you have correctly set any environment variables needed by Spectre.

To run the test, run

```
cargo test design_inverter_spectre --features spectre -- --show-output
```

## Conclusion

You should now be well equipped to start writing your own schematic generators in Substrate.
A full, runnable example for this tutorial is available [here](https://github.com/substrate-labs/substrate2/tree/main/examples/sky130_inverter).

