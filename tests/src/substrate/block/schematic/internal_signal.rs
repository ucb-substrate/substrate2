use std::collections::HashSet;

use super::ResistorIoSchematic;
use arcstr::ArcStr;
use serde::{Deserialize, Serialize};
use substrate::block::Block;
use substrate::context::Context;
use substrate::io::{InOut, Input, Output, Signal};
use substrate::pdk::Pdk;
use substrate::schematic::{HasSchematic, HasSchematicImpl};
use substrate::Io;

use crate::substrate::pdk::ExamplePdkA;

use super::Resistor;

#[derive(Debug, Default, Clone, Io)]
pub struct BufferIo {
    pub input: Input<Signal>,
    pub output: Output<Signal>,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct Buffer {}

impl Block for Buffer {
    type Io = BufferIo;
    fn id() -> arcstr::ArcStr {
        arcstr::literal!("buffer")
    }
    fn name(&self) -> arcstr::ArcStr {
        arcstr::literal!("buffer")
    }
    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl HasSchematic for Buffer {
    type Data = ();
}

impl<PDK: Pdk> HasSchematicImpl<PDK> for Buffer {
    fn schematic(
        &self,
        io: &<<Self as Block>::Io as substrate::io::SchematicType>::Data,
        cell: &mut substrate::schematic::CellBuilder<PDK, Self>,
    ) -> substrate::error::Result<Self::Data> {
        let int = cell.signal("int", Signal);
        let r = cell.instantiate(Resistor { r: 10 });
        cell.connect(io.input, r.io.p);
        cell.connect(r.io.n, int);

        cell.instantiate_connected(
            Resistor { r: 10 },
            ResistorIoSchematic {
                p: int.into(),
                n: (*io.output).into(),
            },
        );
        Ok(())
    }
}

#[test]
fn internal_signal_names_preserved() {
    let mut ctx = Context::new(ExamplePdkA);
    let buffer = Buffer {};

    let lib = ctx.export_scir(buffer);
    assert_eq!(lib.cells().count(), 2);
    let issues = lib.validate();
    println!("Library:\n{:#?}", lib);
    println!("Issues = {:#?}", issues);
    assert_eq!(issues.num_errors(), 0);
    assert_eq!(issues.num_warnings(), 0);

    let vdiv = lib.cell_named("buffer");
    let sigs: HashSet<ArcStr> = vdiv.signals().map(|p| p.1.name.clone()).collect();
    assert_eq!(sigs.len(), 3);
    assert!(sigs.contains("io_input"));
    assert!(sigs.contains("io_output"));
    assert!(sigs.contains("int"));
}
