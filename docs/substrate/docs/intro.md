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
    type Io = ();
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("inverter")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("inverter_{}", self.strength)
    }
    fn io(&self) -> Self::Io {
        ()
    }
}
```

## Layout

```rust
// hidden-rust-doc-start
# use serde::{Serialize, Deserialize};
# use geometry::prelude::Rect;
# use substrate::block::Block;
# use substrate::layout::{draw::DrawContainer, element::Shape, HasLayout, HasLayoutImpl};
# use substrate::pdk::Pdk;
# 
# #[derive(Serialize, Deserialize, Debug, Clone, Hash, PartialEq, Eq)]
# pub struct Inverter {
#     strength: usize,
# }
# impl Inverter {
#     pub fn new(strength: usize) -> Self {
#         Self { strength }
#     }
# }
# impl Block for Inverter {
#     type Io = ();
#     fn id() -> arcstr::ArcStr {
#         arcstr::literal!("inverter")
#     }
#     fn name(&self) -> arcstr::ArcStr {
#         arcstr::format!("inverter_{}", self.strength)
#     }
#     fn io(&self) -> Self::Io {
#         ()
#     }
# }
# pub struct ExamplePdk;
# impl Pdk for ExamplePdk {}
// hidden-rust-doc-end
impl HasLayout for Inverter {
    type Data = ();
}

impl HasLayoutImpl<ExamplePdk> for Inverter {
    fn layout(
        &self,
        cell: &mut substrate::layout::builder::CellBuilder<ExamplePdk, Self>,
    ) -> substrate::error::Result<Self::Data> {
        cell.draw(Shape::new(Rect::from_sides(0, 0, 100, 200)));
        Ok(())
    }
}
```


