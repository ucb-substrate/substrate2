use std::collections::HashSet;

use anyhow::anyhow;
use arcstr::ArcStr;
use codegen::Io;
use serde::{Deserialize, Serialize};
use substrate::context::Context;
use substrate::schematic::CellBuilder;
use substrate::{
    block::Block,
    schematic::{conv::RawLib, NestedData, Schematic},
    types::{HasNameTree, InOut, NameTree, Output, Signal},
};

use crate::types::schematic::IoNodeBundle;

use super::Instance;

// TODO: uncomment
//#[derive(Default, NestedData)]
//pub struct SchematicInstances<T: Schematic> {
//    pub instances: Vec<Instance<T>>,
//}
//
//#[derive(NestedData)]
//pub enum EnumInstances<T: Schematic> {
//    One { one: Instance<T> },
//    Two(Instance<T>, Instance<T>),
//}
//
//#[derive(NestedData)]
//pub struct TwoInstances<T: Schematic>(pub Instance<T>, pub Instance<T>);

#[crate::test]
fn test_schematic_api() {
    pub struct Schema;

    #[derive(Clone)]
    pub enum Primitive {
        Resistor,
    }

    impl scir::schema::Schema for Schema {
        type Primitive = Primitive;
    }

    #[derive(Io, Clone, Default, Debug)]
    pub struct ResistorIo {
        pub p: InOut<Signal>,
        pub n: InOut<Signal>,
    }

    #[derive(Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "ResistorIo")]
    pub struct Resistor;

    impl Schematic for Resistor {
        type Schema = Schema;
        type NestedData = ();

        fn schematic(
            &self,
            io: &IoNodeBundle<Resistor>,
            cell: &mut super::CellBuilder<<Self as Schematic>::Schema>,
        ) -> crate::error::Result<Self::NestedData> {
            let mut prim = PrimitiveBinding::new(Primitive::Resistor);
            prim.connect("p", io.p);
            prim.connect("n", io.n);
            cell.set_primitive(prim);
            Ok(())
        }
    }

    #[derive(Io, Clone, Debug)]
    pub struct DecoupledIo {
        pub ready: Input<Signal>,
        pub valid: Output<Signal>,
        pub data: Output<Array<Signal>>,
    }

    impl DecoupledIo {
        fn new(width: usize) -> Self {
            Self {
                ready: Default::default(),
                valid: Default::default(),
                data: Output(Array::new(width, Default::default())),
            }
        }
    }

    #[derive(Io, Clone, Debug)]
    pub struct MultiDecoupledIo {
        pub d1: DecoupledIo,
        pub d2: Flipped<DecoupledIo>,
        pub d3: Input<DecoupledIo>,
        pub d4: Output<DecoupledIo>,
        pub d5: InOut<DecoupledIo>,
        pub ready: Input<Signal>,
        pub valid: Output<Signal>,
        pub data: Output<Array<Signal>>,
    }

    #[derive(Debug, Copy, Clone, Hash, PartialEq, Eq)]
    pub struct MultiDecoupledBlock;

    impl Block for MultiDecoupledBlock {
        type Io = MultiDecoupledIo;
        fn name(&self) -> arcstr::ArcStr {
            arcstr::literal!("multi_decoupled_block")
        }
        fn io(&self) -> Self::Io {
            MultiDecoupledIo {
                d1: DecoupledIo::new(5),
                d2: Flipped(DecoupledIo::new(4)),
                d3: Input(DecoupledIo::new(5)),
                d4: Output(DecoupledIo::new(4)),
                d5: InOut(DecoupledIo::new(5)),
                ready: Default::default(),
                valid: Default::default(),
                data: Output(Array::new(4, Default::default())),
            }
        }
    }

    impl DataView<DecoupledIoBundleKind> for MultiDecoupledIoBundleKind {
        fn view_nodes_as(nodes: &NodeBundle<Self>) -> NodeBundle<DecoupledIo> {
            NodeBundle::<DecoupledIo> {
                ready: nodes.ready,
                valid: nodes.valid,
                data: nodes.data.clone(),
            }
        }
    }

    impl Schematic for MultiDecoupledBlock {
        type Schema = Schema;
        type NestedData = ();

        fn schematic(
            &self,
            _io: &IoNodeBundle<Self>,
            _cell: &mut CellBuilder<<Self as Schematic>::Schema>,
        ) -> crate::error::Result<Self::NestedData> {
            Ok(())
        }
    }

    #[derive(Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "()")]
    pub struct SuperBlock;

    impl Schematic for SuperBlock {
        type Schema = Schema;
        type NestedData = ();
        fn schematic(
            &self,
            _io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
        ) -> crate::error::Result<Self::NestedData> {
            let b1 = cell.instantiate(MultiDecoupledBlock);
            let b2 = cell.instantiate(MultiDecoupledBlock);
            let wire = cell.signal(
                "abc",
                MultiDecoupledIo {
                    d1: DecoupledIo::new(4),
                    d2: Flipped(DecoupledIo::new(5)),
                    d3: Input(DecoupledIo::new(4)),
                    d4: Output(DecoupledIo::new(5)),
                    d5: InOut(DecoupledIo::new(4)),
                    ready: Default::default(),
                    valid: Default::default(),
                    data: Output(Array::new(5, Default::default())),
                },
            );

            assert!(b1.io().kind() == b2.io().kind());
            assert!(b1.io().d1.kind() == b2.io().d1.kind());
            assert!(b1.io().d1.kind() != b2.io().d2.kind());
            assert!(b1.io().d1.kind() == b2.io().d3.kind());
            assert!(b1.io().d1.kind() != b2.io().d4.kind());
            assert!(b1.io().d1.kind() == b2.io().d5.kind());
            assert!(b1.io().d2.kind() == b2.io().d4.kind());
            assert!(b1.io().d2.kind() == b2.io().view_as::<DecoupledIoBundleKind>().kind());

            cell.connect(&b1.io().d1, &b2.io().d1);
            cell.connect(&b1.io().d1, &b2.io().d1);
            cell.connect(&b1.io().d1, &b2.io().d1);
            cell.connect(&b1.io().d1, &b2.io().d3);
            cell.connect(&b1.io().d1, &b2.io().d5);
            cell.connect(&b1.io().d2, b2.io().view_as::<DecoupledIoBundleKind>());
            cell.connect(wire.d2, &b1.io().d1);

            Ok(())
        }
    }

    #[derive(Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "()")]
    pub struct NestedBlock<T>(T);

    impl<S: crate::schematic::schema::Schema, T: Schematic<Schema = S> + Clone> Schematic
        for NestedBlock<T>
    {
        type Schema = S;
        type NestedData = ();
        fn schematic(
            &self,
            _io: &IoNodeBundle<Self>,
            cell: &mut CellBuilder<<Self as Schematic>::Schema>,
        ) -> crate::error::Result<Self::NestedData> {
            let _b1 = cell.instantiate(self.0.clone());

            Ok(())
        }
    }

    #[derive(Io, Clone, Default, Debug)]
    pub struct VdividerIo {
        pub vdd: InOut<Signal>,
        pub vss: InOut<Signal>,
        pub dout: Output<Signal>,
    }

    pub struct CustomView;

    impl HasViewImpl<CustomView> for Signal {
        type View = i64;
    }

    const VDIVIDER_CUSTOM_VIEW: VdividerIoBundle<CustomView> = VdividerIoBundle {
        vdd: 1,
        vss: 2,
        dout: 3,
    };

    #[derive(Block, Debug, Copy, Clone, Hash, PartialEq, Eq)]
    #[substrate(io = "VdividerIo")]
    pub struct Vdivider;

    impl Schematic for Vdivider {
        type Schema = Schema;
        type NestedData = ();
        fn schematic(
            &self,
            io: &IoNodeBundle<Self>,
            cell: &mut super::CellBuilder<<Self as Schematic>::Schema>,
        ) -> crate::error::Result<Self::NestedData> {
            let r1 = cell.instantiate(Resistor);
            let r2 = cell.instantiate(Resistor);
            r1.try_data()?;
            r2.try_data()?;

            assert!(r1.io().kind() == r2.io().kind());

            cell.connect(&&io.vdd, &r1.io().p);
            cell.connect(&io.dout, &&&&&r1.io().n);
            cell.connect(
                NodeBundle::<ResistorIo> {
                    p: io.dout,
                    n: io.vss,
                },
                r2.io(),
            );

            Ok(())
        }
    }

    let ctx = Context::new();
    ctx.export_scir(Vdivider)
        .expect("failed to generate raw lib");
    ctx.export_scir(NestedBlock(NestedBlock(SuperBlock)))
        .expect("failed to generate raw lib");
}

#[test]
fn can_generate_vdivider_schematic() {
    let ctx = Context::new();
    let vdivider = Vdivider {
        r1: Resistor::new(300),
        r2: Resistor::new(100),
    };
    let RawLib { scir, conv: _ } = ctx.export_scir::<Spice, _>(vdivider).unwrap();
    assert_eq!(scir.cells().count(), 1);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("vdivider_300_100");
    let port_names: HashSet<ArcStr> = vdiv
        .ports()
        .map(|p| vdiv.signal(p.signal()).name.clone())
        .collect();
    assert_eq!(port_names.len(), 3);
    assert!(port_names.contains("pwr_vdd"));
    assert!(port_names.contains("pwr_vss"));
    assert!(port_names.contains("out"));
    assert_eq!(vdiv.ports().count(), 3);
    assert_eq!(vdiv.instances().count(), 2);
}

#[test]
fn can_generate_multi_top_scir() {
    let ctx = Context::new();
    let vdivider1 = Vdivider {
        r1: Resistor::new(300),
        r2: Resistor::new(100),
    };
    let vdivider2 = Vdivider {
        r1: Resistor::new(500),
        r2: Resistor::new(600),
    };
    let vdiv1 = ctx.generate_schematic::<Spice, _>(vdivider1);
    let vdiv2 = ctx.generate_schematic::<Spice, _>(vdivider2);
    let RawLib { scir, conv: _ } = ctx.export_scir_all(&[&vdiv1.raw(), &vdiv2.raw()]).unwrap();
    assert_eq!(scir.cells().count(), 2);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("vdivider_300_100");
    let port_names: HashSet<ArcStr> = vdiv
        .ports()
        .map(|p| vdiv.signal(p.signal()).name.clone())
        .collect();
    assert_eq!(port_names.len(), 3);
    assert!(port_names.contains("pwr_vdd"));
    assert!(port_names.contains("pwr_vss"));
    assert!(port_names.contains("out"));
    assert_eq!(vdiv.ports().count(), 3);
    assert_eq!(vdiv.instances().count(), 2);

    let vdiv = scir.cell_named("vdivider_500_600");
    let port_names: HashSet<ArcStr> = vdiv
        .ports()
        .map(|p| vdiv.signal(p.signal()).name.clone())
        .collect();
    assert_eq!(port_names.len(), 3);
    assert!(port_names.contains("pwr_vdd"));
    assert!(port_names.contains("pwr_vss"));
    assert!(port_names.contains("out"));
    assert_eq!(vdiv.ports().count(), 3);
    assert_eq!(vdiv.instances().count(), 2);
}

#[test]
fn can_generate_flattened_vdivider_schematic() {
    let ctx = Context::new();
    let vdivider = crate::shared::vdivider::flattened::Vdivider::new(300, 100);
    let RawLib { scir, conv: _ } = ctx.export_scir::<Spice, _>(vdivider).unwrap();
    assert_eq!(scir.cells().count(), 1);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("vdivider");
    let port_names: HashSet<ArcStr> = vdiv
        .ports()
        .map(|p| vdiv.signal(p.signal()).name.clone())
        .collect();
    assert_eq!(port_names.len(), 3);
    assert!(port_names.contains("pwr_vdd"));
    assert!(port_names.contains("pwr_vss"));
    assert!(port_names.contains("out"));
    assert_eq!(vdiv.ports().count(), 3);
    assert_eq!(vdiv.instances().count(), 2);
}

#[test]
fn can_generate_flattened_vdivider_array_schematic() {
    let ctx = Context::new();
    let vdiv1 = crate::shared::vdivider::flattened::Vdivider::new(300, 100);
    let vdiv2 = crate::shared::vdivider::flattened::Vdivider::new(600, 800);
    let vdiv3 = crate::shared::vdivider::flattened::Vdivider::new(20, 20);
    let vdivs = vec![vdiv1, vdiv2, vdiv3];
    let vdivider = crate::shared::vdivider::flattened::VdividerArray { vdividers: vdivs };
    let RawLib { scir, conv: _ } = ctx.export_scir::<Spice, _>(vdivider).unwrap();
    assert_eq!(scir.cells().count(), 1);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("flattened_vdivider_array_3");
    let port_names: HashSet<ArcStr> = vdiv
        .ports()
        .map(|p| vdiv.signal(p.signal()).name.clone())
        .collect();
    assert_eq!(port_names.len(), 6);
    assert!(port_names.contains("elements_0_vdd"));
    assert!(port_names.contains("elements_0_vss"));
    assert!(port_names.contains("elements_1_vdd"));
    assert!(port_names.contains("elements_1_vss"));
    assert!(port_names.contains("elements_2_vdd"));
    assert!(port_names.contains("elements_2_vss"));
    assert_eq!(vdiv.ports().count(), 6);
    assert_eq!(vdiv.instances().count(), 6);
}

#[test]
fn nested_io_naming() {
    let io = VdividerIo {
        pwr: PowerIo {
            vdd: InOut(Signal),
            vss: InOut(Signal),
        },
        out: Output(Signal),
    };

    let actual = NameTree::new("io", io.names().unwrap());
    let expected = NameTree::new(
        "io",
        vec![
            NameTree::new(
                "pwr",
                vec![NameTree::new("vdd", vec![]), NameTree::new("vss", vec![])],
            ),
            NameTree::new("out", vec![]),
        ],
    );
    assert_eq!(actual, expected);
}

#[test]
fn internal_signal_names_preserved() {
    let ctx = PdkContext::new(ExamplePdkA);
    let RawLib { scir, conv: _ } = ctx.export_scir::<ExamplePdkA, _>(Buffer::new(5)).unwrap();
    assert_eq!(scir.cells().count(), 4);
    let issues = scir.validate();
    println!("Library:\n{:#?}", scir);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = scir.cell_named("buffer_5");
    let sigs: HashSet<ArcStr> = vdiv.signals().map(|p| p.1.name.clone()).collect();
    assert_eq!(sigs.len(), 5);
    assert!(sigs.contains("vdd"));
    assert!(sigs.contains("vss"));
    assert!(sigs.contains("din"));
    assert!(sigs.contains("dout"));
    assert!(sigs.contains("x"));
}

#[test]
fn nested_node_naming() {
    let ctx = PdkContext::new(ExamplePdkA);
    let handle = ctx.generate_schematic::<ExamplePdkA, _>(BufferNxM::new(5, 5, 5));
    let cell = handle.cell();

    assert_eq!(
        cell.bubbled_inv1.pmos.as_ref().unwrap().io().g.path(),
        cell.bubbled_pmos_g.path()
    );

    assert_eq!(
        cell.bubbled_inv1.io().din.path(),
        cell.buffer_chains[0].bubbled_inv1.io().din.path()
    );
    assert_eq!(
        cell.bubbled_inv1.io().din.path(),
        cell.buffer_chains[0].buffers[0].inv1.io().din.path()
    );

    assert_eq!(
        cell.bubbled_pmos_g.path(),
        cell.buffer_chains[0].bubbled_pmos_g.path()
    );
    assert_eq!(
        cell.bubbled_pmos_g.path(),
        cell.buffer_chains[0].bubbled_inv1.pmos_g.path()
    );
    assert_eq!(
        cell.bubbled_pmos_g.path(),
        cell.buffer_chains[0].buffers[0].inv1.pmos_g.path()
    );

    assert_ne!(
        cell.bubbled_inv1.pmos.as_ref().unwrap().data().path(),
        cell.bubbled_inv1.pmos.as_ref().unwrap().path()
    );
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Block1;

impl Block for Block1 {
    type Io = ();

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("block1")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("block1")
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for Block1 {
    type NestedData = ();
}

#[impl_dispatch({ExamplePdkA; ExamplePdkB})]
impl<PDK> Schematic<PDK> for Block1 {
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as HardwareType>::Bundle,
        _cell: &mut CellBuilder<PDK>,
    ) -> substrate::error::Result<Self::NestedData> {
        Err(substrate::error::Error::Anyhow(
            anyhow!("failed to generate block 1").into(),
        ))
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub struct Block2;

impl Block for Block2 {
    type Io = ();

    fn id() -> arcstr::ArcStr {
        arcstr::literal!("block2")
    }

    fn name(&self) -> arcstr::ArcStr {
        arcstr::format!("block2")
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl ExportsNestedData for Block2 {
    type NestedData = ();
}

impl Schematic<ExamplePdkA> for Block2 {
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkA>,
    ) -> substrate::error::Result<Self::NestedData> {
        let handle = cell.generate(Block1);
        handle.try_cell()?;
        let _inst = cell.add(handle);
        Ok(())
    }
}

impl Schematic<ExamplePdkB> for Block2 {
    fn schematic(
        &self,
        _io: &<<Self as Block>::Io as HardwareType>::Bundle,
        cell: &mut CellBuilder<ExamplePdkB>,
    ) -> substrate::error::Result<Self::NestedData> {
        let handle = cell.generate_blocking(Block1)?;
        let _inst = cell.add(handle);
        Ok(())
    }
}

#[test]
fn error_propagation_works() {
    let ctx = PdkContext::new(ExamplePdkA);
    let handle = ctx.generate_schematic::<ExamplePdkA, _>(Block2);
    assert!(handle.try_cell().is_err());

    let ctx = PdkContext::new(ExamplePdkB);
    let handle = ctx.generate_schematic::<ExamplePdkB, _>(Block2);
    assert!(handle.try_cell().is_err());
}
