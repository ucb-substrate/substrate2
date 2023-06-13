use substrate::schematic::{context::SchematicCtx, instance::Instance};

use crate::substrate::block::vdivider::VDividerInstance;

use self::vdivider::VDivider;

pub mod resistor;
pub mod vdivider;

#[test]
fn makes_vdivider_schematic() {
    let mut ctx = SchematicCtx::new();

    let vdiv = ctx.generate("vdiv", VDivider { r1: 500, r2: 500 });

    let inst = Into::<Instance>::into(vdiv);
    println!("{:?}", inst);
}
