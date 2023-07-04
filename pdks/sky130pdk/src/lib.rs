use std::path::PathBuf;

use corner::*;
use substrate::pdk::Pdk;
use substrate::{LayerFamily, Layers};

pub mod corner;
pub mod mos;

#[derive(Debug, Clone)]
pub struct Sky130Pdk {
    root_dir: PathBuf,
    corners: Sky130Corners,
}

impl Sky130Pdk {
    #[inline]
    pub fn new(root_dir: impl Into<PathBuf>) -> Self {
        Self {
            root_dir: root_dir.into(),
            corners: Default::default(),
        }
    }
}

impl Pdk for Sky130Pdk {
    type Layers = Sky130Layers;
    type Corner = Sky130Corner;

    fn corner(&self, name: &str) -> Option<Self::Corner> {
        match name {
            "tt" => Some(self.corners.tt),
            "sf" => Some(self.corners.sf),
            "fs" => Some(self.corners.fs),
            "ff" => Some(self.corners.ff),
            "ss" => Some(self.corners.ss),
            _ => None,
        }
    }
}

#[derive(Layers)]
pub struct Sky130Layers {
    #[layer(gds = "235/4")]
    pub pr_boundary: PrBoundary,

    #[layer_family]
    pub pwell: Pwell,

    #[layer_family]
    pub nwell: Nwell,

    #[layer_family]
    pub dnwell: Dnwell,

    #[layer_family]
    pub vhvi: Vhvi,

    #[layer_family]
    pub diff: Diff,

    #[layer_family]
    pub tap: Tap,

    #[layer_family]
    pub psdm: Psdm,

    #[layer_family]
    pub nsdm: Nsdm,

    #[layer_family]
    pub poly: Poly,

    #[layer_family]
    pub ldntm: Ldntm,

    #[layer_family]
    pub lvtn: Lvtn,

    #[layer_family]
    pub hvtp: Hvtp,

    #[layer_family]
    pub hvtr: Hvtr,

    #[layer_family]
    pub tunm: Tunm,

    #[layer_family]
    pub licon1: Licon1,

    /// Nitride poly cut.
    #[layer_family]
    pub npc: Npc,

    #[layer_family]
    pub li1: Li1,

    #[layer_family]
    pub mcon: Mcon,

    #[layer_family]
    pub met1: Met1,

    #[layer_family]
    pub via: Via,

    #[layer_family]
    pub met2: Met2,

    #[layer_family]
    pub via2: Via2,

    #[layer_family]
    pub met3: Met3,

    #[layer_family]
    pub via3: Via3,

    #[layer_family]
    pub met4: Met4,

    #[layer_family]
    pub via4: Via4,

    #[layer_family]
    pub met5: Met5,

    #[layer_family]
    pub pad: Pad,

    #[layer_family]
    pub rpm: Rpm,

    #[layer_family]
    pub urpm: Urpm,

    #[layer_family]
    pub hvi: Hvi,

    #[layer_family]
    pub ncm: Ncm,

    #[layer(gds = "22/20")]
    pub cfom: CfomDrawing,
    #[layer(gds = "23/0")]
    pub cfom_mask: CfomMask,
    #[layer(gds = "22/21")]
    pub cfom_mask_add: CfomMaskAdd,
    #[layer(gds = "22/22")]
    pub cfom_mask_drop: CfomMaskDrop,

    #[layer(gds = "115/44")]
    pub cli1m: Cli1mDrawing,
    #[layer(gds = "56/0")]
    pub cli1m_mask: Cli1mMask,
    #[layer(gds = "115/43")]
    pub cli1m_mask_add: Cli1mMaskAdd,
    #[layer(gds = "115/42")]
    pub cli1m_mask_drop: Cli1mMaskDrop,

    #[layer(gds = "81/14")]
    pub areaid_low_tap_density: AreaIdLowTapDensity,
    #[layer(gds = "81/1")]
    pub areaid_seal: AreaIdSeal,
    #[layer(gds = "81/2")]
    pub areaid_core: AreaIdCore,
    #[layer(gds = "81/3")]
    pub areaid_frame: AreaIdFrame,
    #[layer(gds = "81/19")]
    pub areaid_esd: AreaIdEsd,
    #[layer(gds = "81/4")]
    pub areaid_standardc: AreaIdStandardc,
    #[layer(gds = "81/79")]
    pub areaid_analog: AreaIdAnalog,

    #[layer(gds = "236/0")]
    pub outline: Outline,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Pwell {
    #[layer(gds = "64/44", primary)]
    pub pwell_drawing: PwellDrawing,
    #[layer(gds = "122/16", pin)]
    pub pwell_pin: PwellPin,
    #[layer(gds = "64/59", label)]
    pub pwell_label: PwellLabel,
    #[layer(gds = "64/13")]
    pub pwell_res: PwellRes,
    #[layer(gds = "64/14")]
    pub pwell_cut: PwellCut,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Nwell {
    #[layer(gds = "64/20", primary)]
    pub nwell_drawing: NwellDrawing,
    #[layer(gds = "84/23")]
    pub nwell_net: NwellNet,
    #[layer(gds = "64/16", pin)]
    pub nwell_pin: NwellPin,
    #[layer(gds = "64/5", label)]
    pub nwell_label: NwellLabel,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Dnwell {
    #[layer(gds = "64/18", primary)]
    pub dnwell_drawing: DnwellDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Vhvi {
    #[layer(gds = "74/21", primary)]
    pub vhvi_drawing: VhviDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Diff {
    #[layer(gds = "65/20", primary)]
    pub diff_drawing: DiffDrawing,
    #[layer(gds = "65/13")]
    pub diff_res: DiffRes,
    #[layer(gds = "65/14")]
    pub diff_cut: DiffCut,
    #[layer(gds = "65/16", pin)]
    pub diff_pin: DiffPin,
    #[layer(gds = "65/6", label)]
    pub diff_label: DiffLabel,
    #[layer(gds = "65/23")]
    pub diff_net: DiffNet,
    #[layer(gds = "65/4")]
    pub diff_boundary: DiffBoundary,
    #[layer(gds = "65/8")]
    pub diff_hv: DiffHv,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Tap {
    #[layer(gds = "65/44", primary)]
    pub tap_drawing: TapDrawing,
    #[layer(gds = "65/48", pin)]
    pub tap_pin: TapPin,
    #[layer(gds = "65/5", label)]
    pub tap_label: TapLabel,
    #[layer(gds = "65/41")]
    pub tap_net: TapNet,
    #[layer(gds = "65/60")]
    pub tap_boundary: TapBoundary,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Psdm {
    #[layer(gds = "94/20", primary)]
    pub psdm_drawing: PsdmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Nsdm {
    #[layer(gds = "93/44", primary)]
    pub nsdm_drawing: NsdmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Poly {
    #[layer(gds = "66/20", primary)]
    pub poly_drawing: PolyDrawing,
    #[layer(gds = "66/16", pin)]
    pub poly_pin: PolyPin,
    #[layer(gds = "66/13")]
    pub poly_res: PolyRes,
    #[layer(gds = "66/14")]
    pub poly_cut: PolyCut,
    #[layer(gds = "66/9")]
    pub poly_gate: PolyGate,
    #[layer(gds = "66/5", label)]
    pub poly_label: PolyLabel,
    #[layer(gds = "66/4")]
    pub poly_boundary: PolyBoundary,
    #[layer(gds = "66/25")]
    pub poly_probe: PolyProbe,
    #[layer(gds = "66/23")]
    pub poly_net: PolyNet,
    #[layer(gds = "66/83")]
    pub poly_model: PolyModel,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Ldntm {
    #[layer(gds = "11/44", primary)]
    pub ldntm_drawing: LdntmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Lvtn {
    #[layer(gds = "125/44", primary)]
    pub lvtn_drawing: LvtnDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Hvtp {
    #[layer(gds = "78/44", primary)]
    pub hvtp_drawing: HvtpDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Hvtr {
    #[layer(gds = "18/20", primary)]
    pub hvtr_drawing: HvtrDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Tunm {
    #[layer(gds = "80/20", primary)]
    pub tunm_drawing: TunmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Licon1 {
    #[layer(gds = "66/44", primary)]
    pub licon1_drawing: Licon1Drawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Npc {
    #[layer(gds = "95/20", primary)]
    pub npc_drawing: NpcDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Li1 {
    #[layer(gds = "67/20", primary)]
    pub li1_drawing: Li1Drawing,
    #[layer(gds = "67/16", pin)]
    pub li1_pin: Li1Pin,
    #[layer(gds = "67/13")]
    pub li1_res: Li1Res,
    #[layer(gds = "67/14")]
    pub li1_cut: Li1Cut,
    #[layer(gds = "67/5", label)]
    pub li1_label: Li1Label,
    #[layer(gds = "67/23")]
    pub li1_net: Li1Net,
    #[layer(gds = "67/4")]
    pub li1_boundary: Li1Boundary,
    #[layer(gds = "67/10")]
    pub li1_blockage: Li1Blockage,
    #[layer(gds = "67/15")]
    pub li1_short: Li1Short,
    #[layer(gds = "67/25")]
    pub li1_probe: Li1Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Mcon {
    #[layer(gds = "67/44", primary)]
    pub mcon_drawing: MconDrawing,
    #[layer(gds = "67/60")]
    pub mcon_boundary: MconBoundary,
    #[layer(gds = "67/48", pin)]
    pub mcon_pin: MconPin,
    #[layer(gds = "67/41")]
    pub mcon_net: MconNet,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met1 {
    #[layer(gds = "68/20", primary)]
    pub met1_drawing: Met1Drawing,
    #[layer(gds = "68/16", pin)]
    pub met1_pin: Met1Pin,
    #[layer(gds = "68/13")]
    pub met1_res: Met1Res,
    #[layer(gds = "68/14")]
    pub met1_cut: Met1Cut,
    #[layer(gds = "68/5", label)]
    pub met1_label: Met1Label,
    #[layer(gds = "68/23")]
    pub met1_net: Met1Net,
    #[layer(gds = "68/4")]
    pub met1_boundary: Met1Boundary,
    #[layer(gds = "68/10")]
    pub met1_blockage: Met1Blockage,
    #[layer(gds = "68/15")]
    pub met1_short: Met1Short,
    #[layer(gds = "68/25")]
    pub met1_probe: Met1Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Via {
    #[layer(gds = "68/44", primary)]
    pub via_drawing: ViaDrawing,
    #[layer(gds = "68/58", pin)]
    pub via_pin: ViaPin,
    #[layer(gds = "68/41")]
    pub via_net: ViaNet,
    #[layer(gds = "68/60")]
    pub via_boundary: ViaBoundary,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met2 {
    #[layer(gds = "69/20", primary)]
    pub met2_drawing: Met2Drawing,
    #[layer(gds = "69/16", pin)]
    pub met2_pin: Met2Pin,
    #[layer(gds = "69/13")]
    pub met2_res: Met2Res,
    #[layer(gds = "69/14")]
    pub met2_cut: Met2Cut,
    #[layer(gds = "69/5", label)]
    pub met2_label: Met2Label,
    #[layer(gds = "69/23")]
    pub met2_net: Met2Net,
    #[layer(gds = "69/4")]
    pub met2_boundary: Met2Boundary,
    #[layer(gds = "69/10")]
    pub met2_blockage: Met2Blockage,
    #[layer(gds = "69/15")]
    pub met2_short: Met2Short,
    #[layer(gds = "69/25")]
    pub met2_probe: Met2Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Via2 {
    #[layer(gds = "69/44", primary)]
    pub via2_drawing: Via2Drawing,
    #[layer(gds = "69/58")]
    pub via2_pin: Via2Pin,
    #[layer(gds = "69/41")]
    pub via2_net: Via2Net,
    #[layer(gds = "69/60")]
    pub via2_boundary: Via2Boundary,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met3 {
    #[layer(gds = "70/20", primary)]
    pub met3_drawing: Met3Drawing,
    #[layer(gds = "70/16", pin)]
    pub met3_pin: Met3Pin,
    #[layer(gds = "70/13")]
    pub met3_res: Met3Res,
    #[layer(gds = "70/14")]
    pub met3_cut: Met3Cut,
    #[layer(gds = "70/5", label)]
    pub met3_label: Met3Label,
    #[layer(gds = "70/23")]
    pub met3_net: Met3Net,
    #[layer(gds = "70/4")]
    pub met3_boundary: Met3Boundary,
    #[layer(gds = "70/10")]
    pub met3_blockage: Met3Blockage,
    #[layer(gds = "70/15")]
    pub met3_short: Met3Short,
    #[layer(gds = "70/17")]
    pub met3_fuse: Met3Fuse,
    #[layer(gds = "70/25")]
    pub met3_probe: Met3Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Via3 {
    #[layer(gds = "70/44", primary)]
    pub via3_drawing: Via3Drawing,
    #[layer(gds = "70/48", pin)]
    pub via3_pin: Via3Pin,
    #[layer(gds = "70/41")]
    pub via3_net: Via3Net,
    #[layer(gds = "70/60")]
    pub via3_boundary: Via3Boundary,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met4 {
    #[layer(gds = "71/20", primary)]
    pub met4_drawing: Met4Drawing,
    #[layer(gds = "71/16", pin)]
    pub met4_pin: Met4Pin,
    #[layer(gds = "71/13")]
    pub met4_res: Met4Res,
    #[layer(gds = "71/14")]
    pub met4_cut: Met4Cut,
    #[layer(gds = "71/5", label)]
    pub met4_label: Met4Label,
    #[layer(gds = "71/23")]
    pub met4_net: Met4Net,
    #[layer(gds = "71/4")]
    pub met4_boundary: Met4Boundary,
    #[layer(gds = "71/10")]
    pub met4_blockage: Met4Blockage,
    #[layer(gds = "71/15")]
    pub met4_short: Met4Short,
    #[layer(gds = "71/17")]
    pub met4_fuse: Met4Fuse,
    #[layer(gds = "71/25")]
    pub met4_probe: Met4Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Via4 {
    #[layer(gds = "71/44", primary)]
    pub via4_drawing: Via4Drawing,
    #[layer(gds = "71/48", pin)]
    pub via4_pin: Via4Pin,
    #[layer(gds = "71/41")]
    pub via4_net: Via4Net,
    #[layer(gds = "71/60")]
    pub via4_boundary: Via4Boundary,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met5 {
    #[layer(gds = "72/20", primary)]
    pub met5_drawing: Met5Drawing,
    #[layer(gds = "72/16", pin)]
    pub met5_pin: Met5Pin,
    #[layer(gds = "72/13")]
    pub met5_res: Met5Res,
    #[layer(gds = "72/14")]
    pub met5_cut: Met5Cut,
    #[layer(gds = "72/5", label)]
    pub met5_label: Met5Label,
    #[layer(gds = "72/23")]
    pub met5_net: Met5Net,
    #[layer(gds = "72/4")]
    pub met5_boundary: Met5Boundary,
    #[layer(gds = "72/10")]
    pub met5_blockage: Met5Blockage,
    #[layer(gds = "72/15")]
    pub met5_short: Met5Short,
    #[layer(gds = "72/17")]
    pub met5_fuse: Met5Fuse,
    #[layer(gds = "72/25")]
    pub met5_probe: Met5Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Pad {
    #[layer(gds = "76/20", primary)]
    pub pad_drawing: PadDrawing,
    #[layer(gds = "76/16", pin)]
    pub pad_pin: PadPin,
    #[layer(gds = "76/5", label)]
    pub pad_label: PadLabel,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Rpm {
    #[layer(gds = "86/20", primary)]
    pub rpm_drawing: RpmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Urpm {
    #[layer(gds = "79/20", primary)]
    pub urpm_drawing: UrpmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Hvi {
    #[layer(gds = "75/20", primary)]
    pub hvi_drawing: HviDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Ncm {
    #[layer(gds = "92/44", primary)]
    pub ncm_drawing: NcmDrawing,
}
