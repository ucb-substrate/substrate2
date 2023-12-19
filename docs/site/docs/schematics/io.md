---
sidebar_position: 1
---

import CodeSnippet from '@site/src/components/CodeSnippet';
import VdividerMod from '@substrate/examples/spice_vdivider/src/lib.rs?snippet';
import Core from '@substrate/docs/examples/examples/core.rs?snippet';

# IOs

In this section, we'll explore how to define and use the interfaces between generated schematics.

## Defining an IO

The first step in creating a Substrate schematic generator is to define an interface that other generators can use to instantiate your generator. An interface, called an IO in Substrate, defines a set of ports and their directions.

<CodeSnippet language="rust" snippet="vdivider-io">{VdividerMod}</CodeSnippet>

An IO must implement the [`Io`](https://api.substratelabs.io/substrate/io/trait.Io.html) trait. Implementing this trait is most easily done by using `#[derive(Io)]`.

## Schematic types

The IO struct itself does not store any connectivity data, but rather is a template for what an instantiation of the IO should look like. In this sense, the IO struct describes the **schematic type** of an interface (i.e. what signals it contains and how wide each bus within it is).

`VdividerIo::default()`, for example, would describe an IO template where the `vout` port should be a single-bit output signal. We can also describe an IO containing two 5-bit buses as follows:

<CodeSnippet language="rust" snippet="array-io">{Core}</CodeSnippet>

This allows you to parametrize the contents of your interface at runtime in whatever way you like. For example, we can do some calculations in the constructor for `ArrayIo`:

<CodeSnippet language="rust" snippet="array-io-constructor">{Core}</CodeSnippet>

### Port directions

Since port direction rules are often broken in analog design, Substrate does not enforce any directionality checks when connecting two ports. However, Substrate does run a basic driver analysis that throws warnings if there are multiple drivers of a net or no drivers, which may be helpful for debugging purposes.

All IOs implement the [`Directed`](https://api.substratelabs.io/substrate/io/trait.Directed.html) trait, 
which allows them to specify the direction of each of their constituent ports. By default, all 
signals are made to be [`InOut`](https://api.substratelabs.io/substrate/io/enum.Direction.html#variant.InOut), 
but this can be overwritten by wrapping the signal with one of the 
[`Input`](https://api.substratelabs.io/substrate/io/struct.Input.html), 
[`Output`](https://api.substratelabs.io/substrate/io/struct.Output.html), or 
[`InOut`](https://api.substratelabs.io/substrate/io/struct.InOut.html) wrapper types. 
While you are not required to specify directions, it is recommended to improve debuggability and 
readability of your generators.

Wrapping a composite type with a direction will overwrite the direction of all constituent signals. 
In the example below, all of the ports of `SramObserverIo` are inputs.

<CodeSnippet language="rust" snippet="sram-io">{Core}</CodeSnippet>

Similarly, if we wanted to create an `SramDriverIo` that drives the input signals of an SRAM and reads the output, we can use the [`Flipped`](https://api.substratelabs.io/substrate/io/struct.Flipped.html) wrapper type, which flips the direction of each constituent port.

<CodeSnippet language="rust" snippet="sram-driver-io">{Core}</CodeSnippet>


## Bundles

Since IO structs only define the properties of an interface, a separate struct is needed to store 
connectivity data for the signals and buses defined by the IO struct. This struct is called a 
**bundle** and is associated with an IO struct via the 
[SchematicType](https://api.substratelabs.io/substrate/io/trait.SchematicType.html) trait that 
all IOs must implement.

A bundle essentially just stores what each port in the IO is connected to and is created when the schematic type described by an IO struct is instantiated. In the case of the `VdividerIo` given before, the `#[derive(Io)]` macro automatically generates an appropriate schematic type called `VdividerIoSchematic`:

```rust
pub struct VdividerIoSchematic {
    pub vdd: Node,
    pub vss: Node,
    pub dout: Node,
}
```

While IO structs describe the type of an interface, bundles describe the data of an interface and represent the physical wires (`Node`s) in a netlist. As such, bundles can be connected to one another and probed during simulation.

## Connections

Substrate encodes whether two bundles can be connected using the [`Connect`](https://api.substratelabs.io/substrate/io/trait.Connect.html) marker trait.

Connections are made between two bundles by flattening bundles into an array of constituent wires and connecting these wires in order. As such, only bundles of the same type or derived types can be connected by default since Substrate cannot make any assumptions on the ordering of wires in different bundle types.

### Custom connections

While you are free to implement `Connect` on whichever types you like, this requires you to ensure that the behavior above achieves what you want. In general, you should prefer to implement `From` or some other conversion function in order to encode connections between similar IOs.

Suppose we have the following two IOs:

<CodeSnippet language="rust" snippet="mos-io">{Core}</CodeSnippet>

We should not directly implement `Connect` on their associated bundles since the flattened bundles have different lengths, resulting in one wire being left floating after the connection is made. Instead, we can write the following to make it easy to convert a source `ThreePortMosIoSchematic` bundle to a `FourPortMosIoSchematic` bundle that can be connected to the destination `FourPortMosIoSchematic` bundle:

<CodeSnippet language="rust" snippet="mos-io-from">{Core}</CodeSnippet>

However, sometimes, we might want to tie the body port to a separate node:

<CodeSnippet language="rust" snippet="mos-io-body">{Core}</CodeSnippet>

With these functions, we could conceptually write things like this:

```rust
cell.connect(three_port_io.into(), four_port_io);
cell.connect(three_port_io.with_body(vdd), four_port_io);
```
