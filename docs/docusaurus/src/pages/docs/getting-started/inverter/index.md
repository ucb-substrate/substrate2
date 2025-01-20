import CodeSnippet from '@site/src/components/CodeSnippet';
import SubstrateRegistryConfig from '@site/src/components/SubstrateRegistryConfig.mdx';
import DependenciesSnippet from '@site/src/components/DependenciesSnippet';
import Sky130OpenPdk from '@site/src/pages/docs/getting-started/inverter/sky130_open_pdk.md';
import Sky130CdsPdk from '@site/src/pages/docs/getting-started/inverter/sky130_cds_pdk.md';
import {isRelease} from '@site/src/utils/versions';
export const inverterMod = require(`{{EXAMPLES}}/sky130_inverter/src/lib.rs?snippet`);
export const inverterLayout = require(`{{EXAMPLES}}/sky130_inverter/src/layout.rs?snippet`);
export const inverterOpenTb = require(`{{EXAMPLES}}/sky130_inverter/src/tb/open.rs?snippet`);
export const inverterCdsTb = require(`{{EXAMPLES}}/sky130_inverter/src/tb/cds.rs?snippet`);
export function inverterTb(open) { return open ? inverterOpenTb : inverterCdsTb; }
export const cargoToml = require(`{{EXAMPLES}}/sky130_inverter/Cargo.toml?snippet`);

In this tutorial, we'll design and simulate a schematic-level inverter in
the Skywater 130nm process to showcase some of the capabilities of Substrate's
analog simulation interface. Substrate will call into {props.open ? "open source tools (ngspice, magic, and netgen)"
: "Cadence tools (Spectre, Pegasus, and Quantus)"} to run simulations, {props.open ? "" : "DRC, "} LVS, and extraction. 

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

In your project's `Cargo.toml`, start with the following dependencies:

<DependenciesSnippet version="{{VERSION}}" language="toml" title="Cargo.toml" snippet="dependencies">{cargoToml}</DependenciesSnippet>

To pull in the plugins for the necessary Cadence tools, add these depependencies as well:

<DependenciesSnippet version="{{VERSION}}" language="toml" title="Cargo.toml" snippet={props.open ? "open-dependencies" : "cds-dependencies"}>{cargoToml}</DependenciesSnippet>

Let's now add some imports that we'll use later on.
Replace the content of `src/lib.rs` with the following:

<CodeSnippet language="rust" title="src/lib.rs" snippet="imports">{inverterMod}</CodeSnippet>

### Simulator

This tutorial will demonstrate how to invoke { props.open ? 
<a href="https://ngspice.sourceforge.io/index.html" target="_blank" rel="noopener noreferrer">ngspice</a> : 
<a href="https://www.cadence.com/en_US/home/resources/datasheets/spectre-simulation-platform-ds.html" target="_blank" rel="noopener noreferrer">Spectre</a> } 
from Substrate to run transient simulations. { props.open ? "Make sure to install an ngspice version of at least 41 before running your Rust code." : 
"Make sure that Spectre is installed and that the appropriate environment variables have been set." }

### SKY130 PDK

{ props.open ? <Sky130OpenPdk/> : <Sky130CdsPdk/> }

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

Describing a schematic in Substrate requires implementing the `Schematic` trait,
which specifies a block's schematic in a particular **schema**. A schema is essentially 
just a format for representing a schematic. In this case, we want to use the `Sky130Pdk` 
schema as our inverter should be usable in any block generated in SKY130.

Here's how our schematic generator looks:
<CodeSnippet language="rust" title="src/lib.rs" snippet="inverter-schematic">{inverterMod}</CodeSnippet>

The calls to `cell.instantiate(...)` create two sub-blocks: an NMOS and a PMOS.
Note how we pass transistor dimensions to the SKY130-specific `Nfet01v8` and `Pfet01v8` blocks.

The calls to `cell.connect(...)` connect the instantiated transistors to the ports of our inverter.
For example, we connect the drain of the NMOS (`nmos.io().d`) to the inverter output (`io.dout`).

## Testbench

Let's now simulate our inverter and measure the rise and fall times using { props.open ? "ngspice" : "Spectre" }.

Start by creating a new file, `src/tb.rs`. Add a reference to this module
in `src/lib.rs`:

```rust title="src/lib.rs"
pub mod tb;
```

Add the following imports to `src/tb.rs`:
<CodeSnippet language="rust" title="src/tb.rs" snippet="imports">{inverterTb(props.open)}</CodeSnippet>

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

<CodeSnippet language="rust" title="src/tb.rs" snippet="struct-and-impl">{inverterTb(props.open)}</CodeSnippet>

The `Pvt<Sky130Corner>` in our testbench is essentially a 3-tuple of a process corner,
voltage, and temperature. The process corner here is an instance of `Sky130Corner`,
which is defined in the `sky130pdk` plugin for Substrate.

Let's now create the schematic for our testbench. We will do this in the { props.open ? <code>Ngspice</code> : <code>Spectre</code> } schema so that the { props.open ? "ngspice" : "Spectre" } simulator plugin knows how to netlist and simulate our testbench. This should have three components:
* A pulse input source driving the inverter input.
* A DC voltage source supplying power to the inverter.
* The instance of the inverter itself.

Recall that schematic generators can return data for later use. Here, we'd like to probe
the output node of our inverter, so we'll set `Data` in `HasSchematicData` to be of type `Node`.

Here's our testbench setup:

<CodeSnippet language="rust" title="src/tb.rs" snippet="schematic">{inverterTb(props.open)}</CodeSnippet>

We create two {props.open ? "ngspice" : "Spectre"}-specific `Vsource`s (one for VDD, the other as an input stimulus).
We also instantiate our inverter and connect everything up.
The `cell.signal(...)` calls create intermediate nodes.
Creating them isn't strictly necessary (we could connect `inv.io().vdd` directly to `vddsrc.io().p`,
for example), but they can sometimes improve readability of your code and of generated schematics.
Finally, we return the node that we want to probe.

## Design

### Writing a design script

Let's use the code we've written to write a script that
automatically sizes our inverter for equal rise and fall times.

We'll assume that we have a fixed NMOS width and channel length and a set
of possible PMOS widths to sweep over.

Here's our implementation:
<CodeSnippet language="rust" title="src/tb.rs" snippet="schematic-design-script">{inverterTb(props.open)}</CodeSnippet>

We sweep over possible PMOS widths. For each width,
we create a new testbench instance and tell Substrate to simulate it.
We use the `WaveformRef` API to look for 20-80% transitions, and capture their duration.
Finally, we keep track of (and eventually return) the inverter instance that minimizes
the absolute difference between the rise and fall times.

### Running the script

Let's now run the script we wrote. We must first create a Substrate **context** that stores all information 
relevant to Substrate. This includes
the tools you've set up, the current PDK, all blocks that have been generated,
cached computations, and more. We will put this in `src/lib.rs`.

<CodeSnippet language="rust" title="src/lib.rs" snippet="sky130-open-ctx">{inverterMod}</CodeSnippet>

We can then write a Rust unit test to run our design script:

<CodeSnippet language="rust" title="src/tb.rs" snippet="schematic-tests">{inverterTb(props.open)}</CodeSnippet>

To run the test, run

```
cargo test design_inverter -- --show-output
```

If all goes well, the test above should print
the inverter dimensions with the minimum rise/fall time difference.

## Layout

### Generator

The next step is to generate an inverter layout.

Describing a layout in Substrate requires implementing the `Layout` trait,
which specifies a block's layout in a particular **schema**. In the context of layout,
a schema is essentially just a layer stack. In this case, we want to use the `Sky130Pdk` 
schema as our inverter uses the SKY130 layer stack.

Start by creating a new file, `src/layout.rs`. Add a reference to this module
in `src/lib.rs`:

```rust title="src/lib.rs"
pub mod layout;
```

In this file, add the following imports:
<CodeSnippet language="rust" title="src/layout.rs" snippet="imports">{inverterLayout}</CodeSnippet>

Then, implement the layout:
<!-- TODO: break into smaller snippets -->
<CodeSnippet language="rust" title="src/layout.rs" snippet="layout">{inverterLayout}</CodeSnippet>

### Verification

We can now run { props.open ? "" : "DRC and"} LVS using { props.open ? "magic and netgen" : "Pegasus" }.
First, add the following constants to `src/lib.rs`:

<CodeSnippet language="rust" title="src/lib.rs" snippet={ props.open ? "open-constants" : "cds-constants" }>{inverterMod}</CodeSnippet>

We can then write a cargo test in `src/layout.rs` that runs { props.open ? "" : "DRC and"} LVS automatically:

<CodeSnippet language="rust" title="src/layout.rs" snippet={ props.open ? "open-tests" : "cds-tests" }>{inverterLayout}</CodeSnippet>

## Post-extraction design

Now that we have an LVS-clean layout and schematic, we can run our design script using post-extraction simulations.

First, let's update our earlier testbench to use either the pre-extraction or post-extraction netlist.

<CodeSnippet language="rust" title="src/layout.rs" snippet="pex-tb" diffSnippet="schematic-tb">{inverterTb(props.open)}</CodeSnippet>

## Conclusion

You should now be well-equipped to start writing your own schematic generators in Substrate.
A full, runnable example for this tutorial is available [here]({{GITHUB_URL}}/examples/sky130_inverter).

