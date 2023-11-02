---
sidebar_position: 1
---

# Quickstart

Substrate is fully integrated with the Rust ecosystem, so all you need to do to get started is [install Rust](https://www.rust-lang.org/tools/install)! To get you up to speed with Substrate's basic features, we'll show you how make a simple voltage divider schematic.

In this tutorial, we'll breeze through the basics to give you a sense of what Substrate generators look like. If you're looking for a deeper dive into how analog design and simulation work in Substrate, check out the [Designing an inverter](inverter.md) tutorial.

## Creating a Substrate project

Ensure that you have a recent version of Rust installed (1.70.0 or beyond).
Add the Substrate registry to your Cargo config:

```toml title="~/.cargo/config.toml"
[registries]
substrate = { index = "https://github.com/substrate-labs/crates-index" }
```

You only need to do this the first time you set up Substrate.

Next, create a new Rust project:
```bash
cargo new --lib my_generator && cd my_generator
```

In your project's `Cargo.toml`, add the desired version of Substrate as a dependency:

```toml title="Cargo.toml"
[dependencies]
substrate = { version = "0.6", registry = "substrate" }
```
