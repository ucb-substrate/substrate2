---
sidebar_position: 2
---

import CodeSnippet from '@site/src/components/CodeSnippet';
import SubstrateRegistryConfig from '@site/src/components/SubstrateRegistryConfig.mdx';
import DependenciesSnippet from '@site/src/components/DependenciesSnippet';
import {isRelease} from '@site/src/utils/versions';
export const inverterMod = require(`{{EXAMPLES}}/sky130_inverter/src/lib.rs?snippet`);
export const inverterTb = require(`{{EXAMPLES}}/sky130_inverter/src/tb.rs?snippet`);
export const cargoToml = require(`{{EXAMPLES}}/sky130_inverter/Cargo.toml?snippet`);

# Designing an inverter

In this tutorial, we'll design and simulate a schematic-level inverter in
the Skywater 130nm process to showcase some of the capabilities of Substrate's
analog simulation interface. We'll also go into some more detail about what
the code you're writing is actually doing.

## Setup

### Rust

Ensure that you have a recent version of Rust installed.
{ isRelease("{{VERSION}}") ? <div>
Add the Substrate registry to your Cargo config: 

<SubstrateRegistryConfig/>

You only need to do this the first time you set up Substrate.
</div> : <div/> }


Next, create a new Rust project:
```bash
cargo new --lib sky130_inverter && cd sky130_inverter
```

In your project's `Cargo.toml`, add the following dependencies:

<DependenciesSnippet version="{{VERSION}}" language="toml" title="Cargo.toml" snippet="dependencies">{cargoToml}</DependenciesSnippet>

Let's now add some imports that we'll use later on.
Replace the content of `src/lib.rs` with the following:

<CodeSnippet language="rust" title="src/lib.rs" snippet="imports">{inverterMod}</CodeSnippet>

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

<CodeSnippet language="rust" title="src/lib.rs" snippet="inverter-io">{inverterMod}</CodeSnippet>

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
<CodeSnippet language="rust" title="src/lib.rs" snippet="inverter-struct">{inverterMod}</CodeSnippet>

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
<CodeSnippet language="rust" title="src/lib.rs" snippet="inverter-schematic">{inverterMod}</CodeSnippet>

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
<CodeSnippet language="rust" title="src/tb.rs" snippet="imports">{inverterTb}</CodeSnippet>

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

<CodeSnippet language="rust" title="src/tb.rs" snippet="struct-and-impl">{inverterTb}</CodeSnippet>

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

<CodeSnippet language="rust" title="src/tb.rs" snippet="schematic">{inverterTb}</CodeSnippet>

We create two Spectre-specific `Vsource`s (one for VDD, the other as an input stimulus).
We also instantiate our inverter and connect everything up.
The `cell.signal(...)` calls create intermediate nodes.
Creating them isn't strictly necessary (we could connect `inv.io().vdd` directly to `vddsrc.io().p`,
for example), but they can sometimes improve readability of your code and of generated schematics.
Finally, we return the node that we want to probe.

## Design

Let's use the code we've written to write a script that
automatically sizes our inverter for equal rise and fall times.

We'll assume that we have a fixed NMOS width and channel length and a set
of possible PMOS widths to sweep over.

Here's our implementation:
<CodeSnippet language="rust" title="src/tb.rs" snippet="ngspice-design">{inverterTb}</CodeSnippet>

We sweep over possible PMOS widths. For each width,
we create a new testbench instance and tell Substrate to simulate it.
We use the `WaveformRef` API to look for 20-80% transitions, and capture their duration.
Finally, we keep track of (and eventually return) the inverter instance that minimizes
the absolute difference between the rise and fall times.

## Running the script

Let's now run the script we wrote. We must first create a Substrate **context** that stores all information 
relevant to Substrate. This includes
the tools you've set up, the current PDK, all blocks that have been generated,
cached computations, and more.

<CodeSnippet language="rust" title="src/tb.rs" snippet="sky130-open-ctx">{inverterTb}</CodeSnippet>

We can then write a Rust unit test to run our design script:

<CodeSnippet language="rust" title="src/tb.rs" snippet="ngspice-tests">{inverterTb}</CodeSnippet>

To run the test, run

```
cargo test design_inverter_ngspice -- --show-output
```

If all goes well, the test above should print
the inverter dimensions with the minimum rise/fall time difference.

## Adding Spectre support
Adding Spectre support is simply a matter
of defining a Spectre-specific testbench schematic and running the appropriate Spectre simulation
in the inverter design script.

We first create the Spectre-specific testbench:

<CodeSnippet language="rust" title="src/tb.rs" snippet="spectre-schematic">{inverterTb}</CodeSnippet>

We then modify the original design loop to take in a desired backend and run the appropriate simulation:

<CodeSnippet language="rust" title="src/tb.rs" snippet="final-design" diffSnippet="ngspice-design">{inverterTb}</CodeSnippet>

We now have to create a context with Spectre and the commercial PDK installed:

<CodeSnippet language="rust" title="src/tb.rs" snippet="sky130-commercial-ctx">{inverterTb}</CodeSnippet>

We can then create a cargo test to run our design script:

<CodeSnippet language="rust" title="src/tb.rs" snippet="spectre-tests">{inverterTb}</CodeSnippet>

Finally, we will need to modify the ngspice test to specify the desired backend:

<CodeSnippet language="rust" title="src/tb.rs" snippet="final-tests" diffSnippet="ngspice-tests">{inverterTb}</CodeSnippet>

Before running the new Spectre test, ensure that the `SKY130_COMMERCIAL_PDK_ROOT` environment variable points to your installation of
the SKY130 commercial PDK.
Also ensure that you have correctly set any environment variables needed by Spectre.

To run the test, run

```
cargo test design_inverter_spectre --features spectre -- --show-output
```

## Conclusion

You should now be well-equipped to start writing your own schematic generators in Substrate.
A full, runnable example for this tutorial is available [here]({{GITHUB_URL}}/examples/sky130_inverter).

