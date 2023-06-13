use crate::schematic::{cell::SchematicCell, context::SchematicCtx};

use self::vdivider::VDivider;

pub mod resistor;
pub mod vdivider;

#[test]
fn makes_vdivider_schematic() {
    let mut ctx = SchematicCtx::new();

    let vdiv = ctx.generate(VDivider { r1: 500, r2: 500 });

    println!("{:?}", vdiv);

    for instance in vdiv.instances() {}
}
