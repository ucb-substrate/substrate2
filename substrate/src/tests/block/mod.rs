pub mod resistor;
pub mod vdivider;

#[test]
fn makes_vdivider_schematic() {
    use crate::schematic::context::SchematicCtx;

    use self::{resistor::Resistor, vdivider::VDivider};

    let mut ctx = SchematicCtx::new();

    let vdiv = ctx.generate(VDivider { r1: 500, r2: 500 });

    let r1 = vdiv.instance_map().get_instance::<Resistor>("Xr1").unwrap();

    let r2 = vdiv.instance_map().get_instance::<Resistor>("Xr2").unwrap();

    assert!(vdiv.signal_map().connected(r1.intf().n, r2.intf().p));
    assert!(vdiv.signal_map().connected(vdiv.intf().vdd, r1.intf().p));
    assert!(!vdiv
        .signal_map()
        .connected(vdiv.intf().vdd, vdiv.intf().vss));

    println!("{:?}", vdiv);
}
