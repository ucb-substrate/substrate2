---
sidebar_position: 1
---

# Substrate Intro

Substrate is a Rust framework for generating analog circuits. Its goal is to create a user-friendly, modern interface for building process portable, reusable, and understandable circuit generators.

To demonstrate some of Substrate's cool features, we'll design an inverter in the Sky130 process.

## Blocks

```rust
use serde::{Serialize, Deserialize};
use substrate::block::Block;

#[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
pub struct Inverter {
    strength: usize,
}
impl Inverter {
    pub fn new(strength: usize) -> Self {
        Self { strength }
    }
}
impl Block for Inverter {
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("inverter_{}", self.strength)
    }
}
```
