use crate::netlist::{
    HasSpiceLikeNetlist, Include, NetlistKind, NetlistOptions, NetlisterInstance, RenameGround,
};

use crate::{BlackboxContents, BlackboxElement, Primitive, Spice};
use arcstr::ArcStr;
use itertools::Itertools;
use scir::schema::Schema;
use scir::{Cell, Concat, Direction, IndexOwned, Instance, LibraryBuilder, SignalInfo, Slice};
use std::collections::HashMap;
use std::io::Write;

#[test]
fn scir_netlists_correctly() {
    let mut lib = LibraryBuilder::new();
    let mut top = Cell::new("top");
    let vdd = top.add_node("vdd");
    let vss = top.add_node("vss");
    top.expose_port(vdd, Direction::InOut);
    top.expose_port(vss, Direction::InOut);

    let r_blackbox = lib.add_primitive(Primitive::BlackboxInstance {
        contents: BlackboxContents {
            elems: vec![
                "R".into(),
                BlackboxElement::InstanceName,
                " ".into(),
                BlackboxElement::Port("P".into()),
                " ".into(),
                BlackboxElement::Port("N".into()),
                " 3000".into(),
            ],
        },
    });
    let mut inst = Instance::new("blackbox", r_blackbox);
    inst.connect("P", vdd);
    inst.connect("N", vss);
    top.add_instance(inst);

    let top = lib.add_cell(top);
    lib.set_top(top);
    let lib = lib.build().unwrap();

    let mut buf: Vec<u8> = Vec::new();
    let netlister = NetlisterInstance::new(&Spice, &lib, &mut buf, Default::default());
    netlister.export().unwrap();
    let string = String::from_utf8(buf).unwrap();
    println!("{}", string);

    // TODO: more robust assertions about the output
    // Once we have a SPICE parser, we can parse the SPICE back to SCIR
    // and assert that the input SCIR is equivalent to the output SCIR.

    assert_eq!(string.matches("SUBCKT top vdd vss").count(), 1);
    assert_eq!(string.matches("ENDS").count(), 1);
    assert_eq!(string.matches("Rblackbox vdd vss 3000").count(), 1);
}

#[test]
fn spice_like_netlist() {
    pub struct SpiceLikeSchema {
        bus_delimiter: (char, char),
    }

    impl Schema for SpiceLikeSchema {
        type Primitive = ArcStr;
    }

    impl HasSpiceLikeNetlist for SpiceLikeSchema {
        fn write_include<W: Write>(&self, out: &mut W, include: &Include) -> std::io::Result<()> {
            if let Some(section) = &include.section {
                write!(out, ".LIB {:?} {}", include.path, section)?;
            } else {
                write!(out, ".INCLUDE {:?}", include.path)?;
            }
            Ok(())
        }

        fn write_start_subckt<W: Write>(
            &self,
            out: &mut W,
            name: &ArcStr,
            ports: &[&SignalInfo],
        ) -> std::io::Result<()> {
            let (start, end) = self.bus_delimiter;
            write!(out, ".SUBCKT {}", name)?;
            for sig in ports {
                if let Some(width) = sig.width {
                    for i in 0..width {
                        write!(out, " {}{}{}{}", sig.name, start, i, end)?;
                    }
                } else {
                    write!(out, " {}", sig.name)?;
                }
            }
            Ok(())
        }

        fn write_end_subckt<W: Write>(&self, out: &mut W, name: &ArcStr) -> std::io::Result<()> {
            write!(out, ".ENDS {}", name)
        }

        fn write_slice<W: Write>(
            &self,
            out: &mut W,
            slice: Slice,
            info: &SignalInfo,
        ) -> std::io::Result<()> {
            let (start, end) = self.bus_delimiter;
            if let Some(range) = slice.range() {
                for i in range.indices() {
                    if i > range.start() {
                        write!(out, " ")?;
                    }
                    write!(out, "{}{}{}{}", &info.name, start, i, end)?;
                }
            } else {
                write!(out, "{}", &info.name)?;
            }
            Ok(())
        }

        fn write_instance<W: Write>(
            &self,
            out: &mut W,
            name: &ArcStr,
            connections: Vec<ArcStr>,
            child: &ArcStr,
        ) -> std::io::Result<ArcStr> {
            write!(out, "{}", name)?;

            for connection in connections {
                write!(out, " {}", connection)?;
            }

            write!(out, " {}", child)?;

            Ok(name.clone())
        }

        fn write_primitive_inst<W: Write>(
            &self,
            out: &mut W,
            name: &ArcStr,
            connections: HashMap<ArcStr, Vec<ArcStr>>,
            primitive: &<Self as Schema>::Primitive,
        ) -> std::io::Result<ArcStr> {
            write!(out, "{}", name)?;

            let connections = connections
                .into_iter()
                .sorted_by_key(|(name, _)| name.clone())
                .collect::<Vec<_>>();

            for (_, connection) in connections {
                for signal in connection {
                    write!(out, " {}", signal)?;
                }
            }

            write!(out, " {}", primitive)?;

            Ok(name.clone())
        }
    }

    const N: usize = 3;

    let mut lib = LibraryBuilder::<SpiceLikeSchema>::new();

    let resistor = lib.add_primitive("ideal_resistor".into());

    let mut dut = Cell::new("dut");

    let p = dut.add_bus("p", N);
    let n = dut.add_bus("n", N);

    for i in 0..N {
        let mut resistor = Instance::new(format!("inst_{i}"), resistor);
        resistor.connect("p", p.index(i));
        resistor.connect("n", n.index(i));
        dut.add_instance(resistor);
    }

    dut.expose_port(p, Direction::InOut);
    dut.expose_port(n, Direction::InOut);

    let dut = lib.add_cell(dut);

    let mut tb = Cell::new("tb");

    let vdd = tb.add_node("vdd");
    let vss = tb.add_node("vss");

    let mut dut = Instance::new("dut", dut);
    dut.connect("p", Concat::new(vec![vdd.into(); 3]));
    dut.connect("n", Concat::new(vec![vss.into(); 3]));
    tb.add_instance(dut);

    tb.expose_port(vss, Direction::InOut);
    let tb = lib.add_cell(tb);

    lib.set_top(tb);

    let lib = lib.build().unwrap();

    let schema = SpiceLikeSchema {
        bus_delimiter: ('<', '|'),
    };
    let mut buf = Vec::new();
    let netlister = NetlisterInstance::new(
        &schema,
        &lib,
        &mut buf,
        NetlistOptions::new(NetlistKind::Testbench(RenameGround::Yes("0".into())), &[]),
    );

    netlister.export().unwrap();

    let netlist = std::str::from_utf8(&buf).unwrap();

    println!("{:?}", netlist);
    for fragment in [
        r#".SUBCKT dut p<0| p<1| p<2| n<0| n<1| n<2|

  inst_0 n<0| p<0| ideal_resistor
  inst_1 n<1| p<1| ideal_resistor
  inst_2 n<2| p<2| ideal_resistor

.ENDS dut"#,
        "dut vdd vdd vdd 0 0 0 dut",
    ] {
        println!("checking for {:?}", fragment);
        assert!(netlist.contains(fragment));
    }

    let mut buf = Vec::new();
    let netlister = NetlisterInstance::new(&schema, &lib, &mut buf, Default::default());

    netlister.export().unwrap();

    let netlist = std::str::from_utf8(&buf).unwrap();

    println!("{:?}", netlist);
    for fragment in [
        r#".SUBCKT dut p<0| p<1| p<2| n<0| n<1| n<2|

  inst_0 n<0| p<0| ideal_resistor
  inst_1 n<1| p<1| ideal_resistor
  inst_2 n<2| p<2| ideal_resistor

.ENDS dut"#,
        r#".SUBCKT tb vss

  dut vdd vdd vdd vss vss vss dut

.ENDS tb"#,
    ] {
        println!("{:?}", fragment);
        assert!(netlist.contains(fragment));
    }
}
