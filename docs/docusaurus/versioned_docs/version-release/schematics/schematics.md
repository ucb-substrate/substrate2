---
sidebar_position: 2
---

import CodeSnippet from '@site/src/components/CodeSnippet';
export const vdividerMod = require(`{{EXAMPLES}}/spice_vdivider/src/lib.rs?snippet`);
export const core = require(`{{EXAMPLES}}/substrate_api_examples/src/lib.rs?snippet`);

# Schematic generators

This section will cover the basics of writing schematic generators.

## Nested data

Before we can write a schematic generator, we first need to specify what data a block exposes in its
schematic. For example, in a digital buffer circuit, we may want to expose the two internal inverters
so that they can be probed during simulation. We can also expose the internal node that connects the
two inverters for easy access.

<CodeSnippet language="rust" snippet="buffer-nested-data">{core}</CodeSnippet>

### Nested views

Essentially, the only requirement for a struct to be used as nested data is that is has a **nested view**.
A nested view describes how the data changes as it is nested in new cells. For example, a [`Node`]
in one cell becomes a [`NestedNode`]
when that cell is instantiated within another cell, storing the path to itself from the top cell.

Generally, you should not need to directly create your own nested views as `#[derive(NestedData)]` will do
this for you. However, it sometimes may be useful to include your own data that you want to be propagated
up from instance to instance. You can do this by implementing the
[`HasNestedView`] trait.

For example, say you want to propagate up some integer value that was calculated while generating your schematic alongside some nested instances. Then you might define your own nested view and manually implement `HasNestedView` as follows:

<CodeSnippet language="rust" snippet="custom-nested-view">{core}</CodeSnippet>

If you don't want to deal with the extra layer of indirection while accessing the struct, you can also do something like this:

<CodeSnippet language="rust" snippet="custom-nested-view-2">{core}</CodeSnippet>

However, we don't recommend you do this unless you know what you're doing since it is more prone to error and a bit difficult to understand.

:::warning

Be careful when implementing `HasNestedView` yourself, since propagating a node without nesting it correct may cause issues when trying to probe it or allow you to do incorrect things like try to connect to a node in a nested instance. Generally speaking, you should always nest fields that have a nested view.

:::

## Defining a schematic

Once a block has an associated IO and nested data, you can define its schematic using the [`Schematic`] trait:

<CodeSnippet language="rust" snippet="vdivider-schematic">{vdividerMod}</CodeSnippet>

Let's look at what each part of the implementation is doing.

- `type Schema = Spice` declares that `Vdivider` defines its schematic
  in a specific format called `Spice`.
  This allows us to netlist the voltage divider to SPICE and run simulations with it
  in SPICE simulators. For more details on schemas, see the [SCIR chapter](./scir.md).
- `type NestedData = ()` declares that `Vdivider` has no associated nested data.
- `fn schematic(...)`, which defines our schematic, takes in three arguments:
  - `&self` - the block itself, which should contain parameters to the generator.
  - `io` - the bundle corresponding to the cell's IO.
  - `cell` - a Substrate [`CellBuilder`] that provides several helper methods for instantiating sub-blocks, connecting bundles, and running simulations, among other things.
- The two calls to `cell.instantiate(...)` create two resistor instances, one with resistance `self.r1` and the other with resistance `self.r2`, and add them to the schematic.
- The four calls to `cell.connect(...)` connect the terminals of the resistor to the outward-facing IO wires of the cell.
- The final line of the implementation, `Ok(())`, indicates that there was no error and returns `()`. We return `()` because we declared the nested data type to be `()`.

:::info Instances and cells

You may have noticed that `cell.instantiate(...)` returns an
[`Instance`]. We define **instances** as specific instantiations of an underlying **cell**,
or a template for the contents of the instance. The `fn schematic(...)` that we are implementing is generating a cell, and our calls to `cell.instantiate(...)` are running other cell generators then instantiating them as an instance that we can connect to other instances.

This distinction is important since one we generate the underlying cell,
we can create as many instances as we want without needing to regenerate the underlying cell.
The instances will simply point to the cell that has already been generated, and we can access
contents of the underlying cell using functions like [`Instance::try_data`] and [`Instance::block`].

:::

### Error handling

The above example does not have any error handling. That is, the generator would panic if there were any errors
while generating the nested resistor cells.

#### Parallel error propagation

The above code with additional logic for propagating errors is included below:

<CodeSnippet language="rust" snippet="vdivider-try-data-error-handling">{core}</CodeSnippet>

This looks a bit more complex than typical Rust error handling because, by default, calls to `cell.instantiate(...)` generate the instantiated
cell in the background. This allows you to effortlessly generate cells in parallel, but it does
require a bit more thoughtful error handling.

In the code above, we first start generating the two
resistors in parallel by calling `cell.instantiate(...)` twice. While they are both in progress, we
block on the first resistor and return any errors that may have been encountered using Rust's `?` syntax.
We then block on the second resistor and do the same thing. Now that the any potential errors have been propagated, we proceed as normal.

#### Sequential error propagation

If we don't need parallelism and want to be able to handle errors immediately, we can write the following:

<CodeSnippet language="rust" snippet="vdivider-instantiate-blocking-error-handling">{core}</CodeSnippet>

The calls to `cell.instantiate_blocking(...)` wait until the underlying cell has finished generating before returning, allowing us to propagate errors immediately.

:::danger
The errors returned by `cell.instantiate_blocking(...)` and `cell.instantiate(...)` followed by
`r1.try_data()` are irrecoverable because instantiating a block both generates a cell and adds
it to the schematic. Even though we are checking whether the generator succeeded, we cannot
retroactively take the failed cell out of the schematic. That is, we cannot do something like this:

<CodeSnippet language="rust" snippet="vdivider-instantiate-blocking-bad">{core}</CodeSnippet>

Even though it looks like we succesfully recovered from an error, the error was
already been pushed into the schematic via `cell.instantiate_blocking(...)`.
The above methods only work if we want to propagate errors.
If you want to recover from errors, you should use the generate/add workflow outlined next.
:::

#### Error recovery

The correct way to recover from errors is to first generate the underlying cell, check if
it is generated successfully, and only then add it to the schematic:

<CodeSnippet language="rust" snippet="vdivider-generate-add-error-handling">{core}</CodeSnippet>

In this case, we start generating the two resistors in parallel using `cell.generate(...)` followed by `cell.instantiate_blocking(...)`.
We then block on the first resistor and if generation succeeded, we add the resistor to the schematic and connect it up. Otherwise, we just connect the output port to VDD directly.

[`Node`]: https://api.substratelabs.io/substrate/io/struct.Node.html
[`NestedNode`]: https://api.substratelabs.io/substrate/io/struct.NestedNode.html
[`HasNestedView`]: https://api.substratelabs.io/substrate/schematic/trait.HasNestedView.html
[`Schematic`]: https://api.substratelabs.io/substrate/schematic/trait.Schematic.html
[`Instance`]: https://api.substratelabs.io/substrate/schematic/struct.Instance.html
[`CellBuilder`]: https://api.substratelabs.io/substrate/schematic/struct.CellBuilder.html
[`Instance::try_data`]: https://api.substratelabs.io/substrate/schematic/struct.Instance.html#method.try_data
[`Instance::block`]: https://api.substratelabs.io/substrate/schematic/struct.Instance.html#method.block
