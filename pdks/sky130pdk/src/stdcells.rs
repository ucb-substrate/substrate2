//! Standard cell definitions and utilities.

use crate::Sky130Pdk;
use arcstr::ArcStr;
use paste::paste;
use serde::{Deserialize, Serialize};
use spice::Spice;
use std::path::PathBuf;
use substrate::block::Block;
use substrate::io::schematic::HardwareType;
use substrate::io::{InOut, Input, Io, Output, Signal};
use substrate::schematic::{CellBuilder, ExportsNestedData, Schematic};

impl Sky130Pdk {
    pub(crate) fn stdcell_path(&self, lib: &str, name: &str) -> PathBuf {
        self.open_root_dir
            .as_ref()
            .expect("Requires Sky130 open PDK root directory to be specified")
            .join(format!("libraries/{lib}/latest/cells/{name}"))
    }
}

/// The power IO for Sky130 standard cells.
#[derive(Default, Debug, Clone, Copy, Io)]
pub struct PowerIo {
    /// The ground rail.
    pub vgnd: InOut<Signal>,
    /// The power rail.
    pub vpwr: InOut<Signal>,
    /// The nwell body contact.
    pub vnb: InOut<Signal>,
    /// The pwell body contact.
    pub vpb: InOut<Signal>,
}

impl PowerIoSchematic {
    /// Creates a `PowerIo` with `vnb` and `vpb` tied to `vdd` and `vss`, respectively.
    pub fn with_bodies_tied_to_rails(pwr: substrate::io::PowerIoSchematic) -> Self {
        Self {
            vgnd: pwr.vss,
            vnb: pwr.vss,
            vpb: pwr.vdd,
            vpwr: pwr.vdd,
        }
    }
}

macro_rules! define_stdcell {
    ($typ:ident, $name:ident, $doc:literal, [$($ports_upper:ident),*], [$($ports_lower:ident),*], [$($directions:ident),*], [$($port_docs:literal),*], [$($strengths:expr),*]) => {
paste! {
    #[derive(Debug, Default, Clone, Copy, Io)]
    #[doc = concat!("The IO of a `", stringify!($name), "` standard cell.")]
    pub struct [<$typ Io>] {
        /// The power interface.
        pub pwr: PowerIo,
        $(
            #[doc = $port_docs]
            pub $ports_lower: $directions<Signal>,
        )*
    }

    #[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
    #[doc = $doc]
    pub enum $typ {
        $(
            #[doc = concat!("A strength ", stringify!($strengths), " variant of this standard cell.")]
            [<S $strengths>],
        )*
    }

    impl $typ {
        #[doc = concat!("Returns the strength of this `", stringify!($name), "` standard cell.")]
        pub fn strength(&self) -> i64 {
            match self {
                $(Self::[<S $strengths>] => $strengths,)*
            }
        }
    }

    impl Block for $typ {
        type Io = [<$typ Io>];

        fn id() -> ArcStr {
            arcstr::literal!(stringify!($name))
        }

        fn name(&self) -> ArcStr {
            arcstr::format!("{}_{}", stringify!($name), self.strength())
        }

        fn io(&self) -> Self::Io {
            Default::default()
        }
    }

    impl ExportsNestedData for $typ {
        type NestedData = ();
    }

    impl Schematic<Sky130Pdk> for $typ {
        fn schematic(
            &self,
            io: &<<Self as Block>::Io as HardwareType>::Bundle,
            cell: &mut CellBuilder<Sky130Pdk>,
        ) -> substrate::error::Result<Self::NestedData> {
            let pdk = cell
                .ctx()
                .get_installation::<Sky130Pdk>()
                .expect("Requires Sky130 PDK installation");

            let lib = "sky130_fd_sc_hd";
            let name = stringify!($name);
            let cell_name = format!("{lib}__{name}_{}", self.strength());
            let mut scir = Spice::scir_cell_from_file(
                pdk.stdcell_path(lib, name)
                    .join(format!("{}.spice", cell_name)),
                &cell_name,
            )
            .convert_schema::<Sky130Pdk>()?;

            scir.connect("VGND", io.pwr.vgnd);
            scir.connect("VNB", io.pwr.vnb);
            scir.connect("VPB", io.pwr.vpb);
            scir.connect("VPWR", io.pwr.vpwr);
            $(scir.connect(stringify!($ports_upper), io.$ports_lower);)*

            cell.set_scir(scir);
            Ok(())
        }
    }
}
    };
}

define_stdcell!(
    And2,
    and2,
    "A 2-input AND gate.",
    [A, B, X],
    [a, b, x],
    [Input, Input, Output],
    ["Input A.", "Input B.", "The gate output."],
    [0, 1, 2, 4]
);
define_stdcell!(
    And3,
    and3,
    "A 3-input AND gate.",
    [A, B, C, X],
    [a, b, c, x],
    [Input, Input, Input, Output],
    ["Input A.", "Input B.", "Input C.", "The gate output."],
    [1, 2, 4]
);
define_stdcell!(
    Buf,
    buf,
    "A buffer.",
    [A, X],
    [a, x],
    [Input, Output],
    ["The buffer input.", "The buffer output."],
    [1, 2, 4, 6, 8, 12, 16]
);
define_stdcell!(
    Bufbuf,
    bufbuf,
    "A cascaded pair of buffers.",
    [A, X],
    [a, x],
    [Input, Output],
    ["The buffer input.", "The buffer output."],
    [8, 16]
);
define_stdcell!(
    Inv,
    inv,
    "An inverter.",
    [A, Y],
    [a, y],
    [Input, Output],
    ["The inverter input.", "The inverter output."],
    [1, 2, 4, 6, 8]
);
// TODO: Manually implement for tap since no need to nest power IO.
define_stdcell!(Tap, tap, "A tap to VDD and GND.", [], [], [], [], [1, 2]);
define_stdcell!(
    Mux2,
    mux2,
    "A 2-input multiplexer.",
    [A0, A1, S, X],
    [a0, a1, s, x],
    [Input, Input, Input, Output],
    [
        "Input 0.",
        "Input 1.",
        "The select bit.",
        "The multiplexer output."
    ],
    [1, 2, 4, 8]
);
define_stdcell!(
    Mux4,
    mux4,
    "A 4-input multiplexer.",
    [A0, A1, A2, A3, S0, S1, X],
    [a0, a1, a2, a3, s0, s1, x],
    [Input, Input, Input, Input, Input, Input, Output],
    [
        "Input 0.",
        "Input 1.",
        "Input 2.",
        "Input 3.",
        "Select bit 0.",
        "Select bit 1.",
        "The multiplexer output."
    ],
    [1, 2, 4]
);
define_stdcell!(
    Nand2,
    nand2,
    "A 2-input NAND gate.",
    [A, B, Y],
    [a, b, y],
    [Input, Input, Output],
    ["Input A.", "Input B.", "The gate output."],
    [1, 2, 4, 8]
);
define_stdcell!(
    Nand3,
    nand3,
    "A 3-input NAND gate.",
    [A, B, C, Y],
    [a, b, c, y],
    [Input, Input, Input, Output],
    ["Input A.", "Input B.", "Input C", "The gate output."],
    [1, 2, 4]
);
define_stdcell!(
    Nor2,
    nor2,
    "A 2-input NOR gate.",
    [A, B, Y],
    [a, b, y],
    [Input, Input, Output],
    ["Input A.", "Input B.", "The gate output."],
    [1, 2, 4, 8]
);
define_stdcell!(
    Nor3,
    nor3,
    "A 3-input NOR gate.",
    [A, B, C, Y],
    [a, b, c, y],
    [Input, Input, Input, Output],
    ["Input A.", "Input B.", "Input C.", "The gate output."],
    [1, 2, 4]
);
define_stdcell!(
    Or2,
    or2,
    "A 2-input OR gate.",
    [A, B, X],
    [a, b, x],
    [Input, Input, Output],
    ["Input A.", "Input B.", "The gate output."],
    [0, 1, 2, 4]
);
define_stdcell!(
    Or3,
    or3,
    "A 3-input OR gate.",
    [A, B, C, X],
    [a, b, c, x],
    [Input, Input, Input, Output],
    ["Input A.", "Input B.", "Input C.", "The gate output."],
    [1, 2, 4]
);
define_stdcell!(
    Xnor2,
    xnor2,
    "A 2-input XNOR gate.",
    [A, B, Y],
    [a, b, y],
    [Input, Input, Output],
    ["Input A.", "Input B.", "The gate output."],
    [1, 2, 4]
);
define_stdcell!(
    Xnor3,
    xnor3,
    "A 3-input XNOR gate.",
    [A, B, C, Y],
    [a, b, c, y],
    [Input, Input, Input, Output],
    ["Input A.", "Input B.", "Input C.", "The gate output."],
    [1, 2, 4]
);
define_stdcell!(
    Xor2,
    xor2,
    "A 2-input XOR gate.",
    [A, B, X],
    [a, b, x],
    [Input, Input, Output],
    ["Input A.", "Input B.", "The gate output."],
    [1, 2, 4]
);
define_stdcell!(
    Xor3,
    xor3,
    "A 3-input XOR gate.",
    [A, B, C, X],
    [a, b, c, x],
    [Input, Input, Input, Output],
    ["Input A.", "Input B.", "Input C.", "The gate output."],
    [1, 2, 4]
);
define_stdcell!(
    Diode,
    diode,
    "An antenna diode.",
    [DIODE],
    [diode],
    [InOut],
    ["The diode node."],
    [2]
);
define_stdcell!(
    Dfxtp,
    dfxtp,
    "A positive edge triggered delay flop.",
    [CLK, D, Q],
    [clk, d, q],
    [Input, Input, Output],
    ["The clock signal.", "The data input.", "The data output."],
    [1, 2, 4]
);
define_stdcell!(
    Dfrtp,
    dfrtp,
    "A positive edge triggered delay flop with inverted reset.",
    [CLK, D, RESET_B, Q],
    [clk, d, reset_b, q],
    [Input, Input, Input, Output],
    [
        "The clock signal.",
        "The data input.",
        "The inverted reset signal.",
        "The data output."
    ],
    [1, 2, 4]
);
