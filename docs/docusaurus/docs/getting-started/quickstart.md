---
sidebar_position: 1
---

import CodeSnippet from '@site/src/components/CodeSnippet';
import SubstrateRegistryConfig from '@site/src/components/SubstrateRegistryConfig.mdx';
import DependenciesSnippet from '@site/src/components/DependenciesSnippet';
import {isRelease} from '@site/src/utils/versions';
export const vdividerMod = require("{{EXAMPLES}}/spice_vdivider/src/lib.rs?snippet");
export const cargoToml = require("{{EXAMPLES}}/spice_vdivider/Cargo.toml?snippet");

# Quickstart

To get you up to speed with Substrate's basic features, we'll show you how make a simple voltage divider schematic.

In this tutorial, we'll breeze through the basics to give you a sense of what Substrate generators look like. If you're looking for a deeper dive into how analog design and simulation work in Substrate, check out the [Designing an inverter](inverter.md) tutorial.

## Creating a Substrate project

Substrate is fully integrated with the Rust ecosystem, so all you need to get started is a [recent installation of Rust](https://www.rust-lang.org/tools/install)! Ensure that you have version 1.70.0 or beyond.

{ isRelease("{{VERSION}}") ? <div>
First, add the Substrate registry to your Cargo config:

<SubstrateRegistryConfig/>

You only need to do this the first time you set up Substrate.
</div> : <div/> }

Next, create a new Rust project:
```bash
cargo new --lib my_generator && cd my_generator
```

In your project's `Cargo.toml`, add the following dependencies:


<DependenciesSnippet version="{{VERSION}}" language="toml" title="Cargo.toml" snippet="dependencies">{cargoToml}</DependenciesSnippet>

Let's now add some imports that we'll use later on.
Replace the content of `src/lib.rs` with the following:

<CodeSnippet language="rust" title="src/lib.rs" snippet="imports">{vdividerMod}</CodeSnippet>

## Interface

We'll first define the interface (also referred to as IO) exposed by our voltage divider.

<CodeSnippet language="rust" title="src/lib.rs" snippet="vdivider-io">{vdividerMod}</CodeSnippet>

## Voltage divider parameters

Now that we've defined an IO, we can define a **block**.
Substrate blocks are analogous to modules or cells in other generator frameworks.

<CodeSnippet language="rust" title="src/lib.rs" snippet="vdivider-struct">{vdividerMod}</CodeSnippet>

## Schematic generator

We now define the schematic of the voltage divider.

<CodeSnippet language="rust" title="src/lib.rs" snippet="vdivider-schematic">{vdividerMod}</CodeSnippet>

## Writing the netlist

We can now write a Rust unit test to write the netlist to a file.

<CodeSnippet language="rust" title="lib/tb.rs" snippet="tests">{vdividerMod}</CodeSnippet>

To run the test, run

```
cargo test netlist_vdivider
```

## Conclusion

If all goes well, the test above should write the voltage divider netlist to `tests/netlist_vdivider/vdivider.spice`.
A full, runnable example for this tutorial is available [here]({{GITHUB_URL}}/examples/spice_vdivider).

