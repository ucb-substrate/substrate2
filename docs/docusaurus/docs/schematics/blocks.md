---
sidebar_position: 2
---

import CodeSnippet from '@site/src/components/CodeSnippet';
export const vdividerMod = require("{{EXAMPLES}}/spice_vdivider/src/lib.rs?snippet");
export const core = require("{{EXAMPLES}}/substrate_api_examples/src/lib.rs?snippet");

# Blocks

Blocks are composable generator units that are analogous to modules or cells in other generator frameworks. In this section, we'll detail how blocks are intended to be used in Substrate.

## Defining a block

A block is simply a Rust type that stores generator parameters:

<CodeSnippet language="rust" snippet="vdivider-struct">{vdividerMod}</CodeSnippet>

A block must implement the [`Block`] trait. In the
above example, this is done using `#[derive(Block)]` and `#[substrate(io = "VdividerIo")]`. However, this only works when the IO (in this
case, `VdividerIo`) implements `Default`.

Though it is more verbose, it is generally preferred to implement the trait manually as this also allows you to provide a more descriptive name for generated cells and parameterize the block's IO:

<CodeSnippet language="rust" snippet="sram-block">{core}</CodeSnippet>

There are a few things you need to specify when defining a block:

| <div style={{width: "100px"}}>Member</div> | Description                                                                                                                                                                                      |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `type Io`                                  | The IO type of the block. See the [IOs section](./io.md) for more details.                                                                                                                       |
| `fn name`                                  | Returns a name describing a specific instantiation of a block. This is used to create descriptive cell names when netlisting or writing a layout to GDS.                                         |
| `fn io`                                    | Returns an instantiation of the block's IO type, describing the properties of the IO for a specific set of parameters. This allows you to vary bus lengths at runtime based on block parameters. |

## Block contents

The `Block` trait requires you to implement several other traits, the most notable of which is the `Eq` trait. Substrate uses your `Eq` implementation to determine if a block needs to be regenerated, or if it has already been generated. As such, your block type should contain all of the parameters that impact your generator's behavior.

If you use `#[derive(Eq)]`, you will generally be safe as long as you keep all of your parameters in your block struct. Let's revisit the voltage divider example.

<CodeSnippet language="rust" snippet="vdivider-struct">{vdividerMod}</CodeSnippet>

This derived `Eq` implementation is fine, since it checks that both resistors are equal. If the two resistances of the voltage divider haven't changed, we can reuse a previously generated instance of the voltage divider. This `Eq` implementation, however, is a bit problematic:

<CodeSnippet language="rust" snippet="vdivider-bad-eq">{core}</CodeSnippet>

Now, let's say you generate a voltage divider with two 100 ohm resistors. Then, you try to generate a voltage divider with one 100 ohm resistor and one 200 ohm resistor. Since Substrate thinks these are equivalent due to your `Eq` implementation, it will reuse the previously generated voltage divider with two 100 ohm resistors!

:::warning
Make sure that your block struct contains any relevant parameters and has a correct `Eq` implementation. Otherwise, Substrate may incorrectly cache generated versions of your block, leading to errors that are extremely difficult to catch.
:::

[`Block`]: {{API}}/substrate/block/trait.Block
