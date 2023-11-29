---
sidebar_position: 4
---

import CodeSnippet from '@site/src/components/CodeSnippet';
import Core from '@substrate/docs/examples/examples/core.rs?snippet';

# Schematic cell intermediate representation (SCIR)

SCIR is an intermediate representation used by Substrate to allow schematic portability between netlisters and simulators.
This section will cover where SCIR is used in Substrate's API and how it interfaces with plugins for tools like ngspice and Spectre.

## Overview

### Basic objects

The table below provides high-level definitions of basic SCIR objects.

| Term | Definition |
| --- | --- |
| Instance | An instantiation of another SCIR object. |
| Cell | A collection of interconnected SCIR instances, corresponding to a SUBCKT in SPICE. |
| Signal | A node or bus within a SCIR cell. |
| Slice | A subset of a SCIR signal. |
| Port | A SCIR signal that has been exposed to the instantiators of its parent SCIR cell. |
| Connection | A SCIR signal from a parent cell connected to a port of child SCIR instance. |
| Library | A set of cells, of which one may be designated as the "top" cell. |

Take the following SPICE circuit as an example:

```spice
* CMOS buffer

.subckt buffer din dout vdd vss
X0 din dinb vdd vss inverter
X1 dinb dout vdd vss inverter
.ends

.subckt inverter din dout vdd vss
X0 dout din vss vss sky130_fd_pr__nfet_01v8 w=2 l=0.15
X1 dout din vdd vdd sky130_fd_pr__pfet_01v8 w=4 l=0.15
.ends
```

This circuit could conceptually be parsed to a SCIR library containing two cells named `buffer` and `inverter`. The buffer cell would contain 5 signals (`din`, `dinb`, `dout`, `vdd`, and `vss`), 4 of which are exposed as ports, as well as two instances of the `inverter` cell. The `dinb` signal is connected to the `dout` port of the first inverter instance and the `din` port of the second inverter instance.

### Primitives

Since SCIR cells are simply collections of SCIR instances, SCIR instances must be able to instantiate more than just cells since we would otherwise only be able to represent an empty hierarchy. As such, SCIR allows users to define a set of arbitrary primitives that can be instantiated within SCIR cells. These primitives are opaque to SCIR and contain any data that the user sees fit.

In the above buffer example, the `sky130_fd_pr__nfet_01v8` and `sky130_fd_pr__pfet_01v8` are a type of primitive called a "raw instance" that allow a SCIR instance to reference an external SPICE model and provide its parameters. In Rust, the primitive definition looks like this:

```rust
enum Primitive {
    // ...

    /// A raw instance with an associated cell.
    RawInstance {
        /// The ordered ports of the instance.
        ports: Vec<ArcStr>,
        /// The associated cell.
        cell: ArcStr,
        /// Parameters associated with the raw instance.
        params: HashMap<ArcStr, ParamValue>,
    }

    // ...
}
```

While SCIR cells and instances do not have parameters, parameters can be injected using SCIR primitives as shown above.

### Schemas

SCIR schemas are simply sets of primitives that can be used to describe circuits. For example, the 
SPICE schema consists of MOSFET, resistor, capacitor, raw instance, and other primitives that can 
describe any circuit that can be netlisted to SPICE. Similarly, SKY130 is also a schema since it 
has its own set of primitive MOSFETs and resistors that can be fabricated in the SKY130 process.

When you write a schema, you can also specify which schemas it can be converted to. This allows you to elegantly 
encode portability in the type system. Since the SKY130 PDK supports simulations in ngspice and Spectre, we 
can declare that the SKY130 schema can be converted to both the ngspice and Spectre schemas. 
The specifics of this procedure will be detailed later on in this section.

### Relationship to Substrate

Generators in Substrate produce cells that can be exported to SCIR. Substrate's APIs allow defining 
schematics in different schemas, which encodes generator compatibility in the Rust type system. For example, 
a Substrate block with a schematic in the `Sky130Pdk` schema can be included in a Spectre testbench's schematic in the `Spectre` schema, but cannot be included in an HSPICE testbench since the `Sky130Pdk` schema is not compatible with the HSPICE schema. Similarly, you cannot instantiate a SKY130 schematic in a schematic in a different process node (e.g. GF180).

While the user generally interfaces with Substrate's block API, simulator and netlister plugins interface with SCIR.
This allows backend tools to abstract away Substrate's internal representation of cells.
For simulation and netlisting, Substrate will export the appropriate cell to SCIR and pass the generated SCIR 
library to the necessary plugin for processing.

## Technical Details

### Schemas

Every SCIR library requires an underlying schema that implements the [`Schema`](https://api.substratelabs.io/scir/schema/trait.Schema.html) trait.

```rust
pub trait Schema {
    type Primitive: Primitive;
}
```

A SCIR schema has an associated primitive type that describes available primitives for representing objects that cannot be represented directly in SCIR. As an example, the most basic schema, [`NoSchema`](https://api.substratelabs.io/scir/schema/struct.NoSchema.html), has a primitive type of [`NoPrimitive`](https://api.substratelabs.io/scir/schema/struct.NoPrimitive.html) that cannot be instantiated — as such, any SCIR library with this schema will have no primitives.

The relationship between schemas is encoded via the [`FromSchema`](https://api.substratelabs.io/scir/schema/trait.FromSchema.html) trait, which describes how one schema is converted to another.

```rs
pub trait FromSchema<S: Schema>: Schema {
    type Error;

    // Required methods
    fn convert_primitive(
        primitive: <S as Schema>::Primitive
    ) -> Result<<Self as Schema>::Primitive, Self::Error>;
    fn convert_instance(
        instance: &mut Instance,
        primitive: &<S as Schema>::Primitive
    ) -> Result<(), Self::Error>;
}
```

Schemas that are inter-convertible must have a 1-to-1 correspondence between their primitives, as shown by the 
signature of `fn convert_primitive(...)`. The instance conversion function, `fn convert_instance(...)`, 
allows you to modify the connections of a SCIR instance that is associated with a primitive to correctly
connect to the ports of the primitive in the new schema.

The `FromSchema` trait is particularly important since it allows for schematics to be made simulator and netlist portable, and potentially even process portable, as we will see later.

### Libraries

Once we have a schema, we can start creating a SCIR library by instantiating a [`LibraryBuilder`](https://api.substratelabs.io/scir/struct.LibraryBuilder.html). To create a library with the [`StringSchema`](https://api.substratelabs.io/scir/schema/struct.StringSchema.html) schema, whose primitives are arbitrary `ArcStr`s, we write the following:

<CodeSnippet language="rust" snippet="scir-library-builder">{Core}</CodeSnippet>

SCIR libraries are collections of SCIR cells and primitives. We can create a new cell and add it to our library:

<CodeSnippet language="rust" snippet="scir-library-cell">{Core}</CodeSnippet>

We can also add primitives to the library as follows (since we are using `StringSchema`, the value of the primitive must be an `ArcStr`):

<CodeSnippet language="rust" snippet="scir-library-primitive">{Core}</CodeSnippet>

SCIR cells may contain signals that connect instances and/or serve as ports that interface with parent cells.

<CodeSnippet language="rust" snippet="scir-library-signals">{Core}</CodeSnippet>

SCIR cells may also contain instances of SCIR primitives and other cells. We can connect ports of each instance to signals in the parent cell. While connections to instances of SCIR cells must connect to ports declared in the underlying cell, connections to primitives are not checked by SCIR as primitives are opaque to SCIR.

We can first instantiate the resistor primitives we defined earlier and add our voltage divider cell to our SCIR library.

<CodeSnippet language="rust" snippet="scir-library-primitive-instances">{Core}</CodeSnippet>

We can then create a cell that instantiates two of our newly-defined voltage divider cell.

<CodeSnippet language="rust" snippet="scir-library-instances">{Core}</CodeSnippet>

### Bindings

SCIR primitives and cells can be instantiated in Substrate generators using *bindings*. Suppose we have the following schema that supports instantiating resistor and capacitor primitives:

<CodeSnippet language="rust" snippet="scir-schema">{Core}</CodeSnippet>

We can create a Substrate block whose schematic corresponds to a `MyPrimitive::Resistor` using a [`PrimitiveBinding`](https://api.substratelabs.io/substrate/schematic/struct.PrimitiveBinding.html). It can then be instantiated in other Substrate generators just like any other block.

<CodeSnippet language="rust" snippet="scir-primitive-binding">{Core}</CodeSnippet>

Similarly, we can bind to a SCIR cell using a [`ScirBinding`](https://api.substratelabs.io/substrate/schematic/struct.ScirBinding.html):

<CodeSnippet language="rust" snippet="scir-scir-binding">{Core}</CodeSnippet>
