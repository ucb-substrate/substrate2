//! The set of PDK layers.
#![allow(missing_docs)]
use substrate::pdk::layers::{LayerFamily, Layers};

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
    pub drawing: PwellDrawing,
    #[layer(gds = "122/16", pin)]
    pub pin: PwellPin,
    #[layer(gds = "64/59", label)]
    pub label: PwellLabel,
    #[layer(gds = "64/13")]
    pub res: PwellRes,
    #[layer(gds = "64/14")]
    pub cut: PwellCut,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Nwell {
    #[layer(gds = "64/20", primary)]
    pub drawing: NwellDrawing,
    #[layer(gds = "84/23")]
    pub net: NwellNet,
    #[layer(gds = "64/16", pin)]
    pub pin: NwellPin,
    #[layer(gds = "64/5", label)]
    pub label: NwellLabel,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Dnwell {
    #[layer(gds = "64/18", primary)]
    pub drawing: DnwellDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Vhvi {
    #[layer(gds = "74/21", primary)]
    pub drawing: VhviDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Diff {
    #[layer(gds = "65/20", primary)]
    pub drawing: DiffDrawing,
    #[layer(gds = "65/13")]
    pub res: DiffRes,
    #[layer(gds = "65/14")]
    pub cut: DiffCut,
    #[layer(gds = "65/16", pin)]
    pub pin: DiffPin,
    #[layer(gds = "65/6", label)]
    pub label: DiffLabel,
    #[layer(gds = "65/23")]
    pub net: DiffNet,
    #[layer(gds = "65/4")]
    pub boundary: DiffBoundary,
    #[layer(gds = "65/8")]
    pub hv: DiffHv,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Tap {
    #[layer(gds = "65/44", primary)]
    pub drawing: TapDrawing,
    #[layer(gds = "65/48", pin)]
    pub pin: TapPin,
    #[layer(gds = "65/5", label)]
    pub label: TapLabel,
    #[layer(gds = "65/41")]
    pub net: TapNet,
    #[layer(gds = "65/60")]
    pub boundary: TapBoundary,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Psdm {
    #[layer(gds = "94/20", primary)]
    pub drawing: PsdmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Nsdm {
    #[layer(gds = "93/44", primary)]
    pub drawing: NsdmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Poly {
    #[layer(gds = "66/20", primary)]
    pub drawing: PolyDrawing,
    #[layer(gds = "66/16", pin)]
    pub pin: PolyPin,
    #[layer(gds = "66/13")]
    pub res: PolyRes,
    #[layer(gds = "66/14")]
    pub cut: PolyCut,
    #[layer(gds = "66/9")]
    pub gate: PolyGate,
    #[layer(gds = "66/5", label)]
    pub label: PolyLabel,
    #[layer(gds = "66/4")]
    pub boundary: PolyBoundary,
    #[layer(gds = "66/25")]
    pub probe: PolyProbe,
    #[layer(gds = "66/23")]
    pub net: PolyNet,
    #[layer(gds = "66/83")]
    pub model: PolyModel,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Ldntm {
    #[layer(gds = "11/44", primary)]
    pub drawing: LdntmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Lvtn {
    #[layer(gds = "125/44", primary)]
    pub drawing: LvtnDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Hvtp {
    #[layer(gds = "78/44", primary)]
    pub drawing: HvtpDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Hvtr {
    #[layer(gds = "18/20", primary)]
    pub drawing: HvtrDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Tunm {
    #[layer(gds = "80/20", primary)]
    pub drawing: TunmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Licon1 {
    #[layer(gds = "66/44", primary)]
    pub drawing: Licon1Drawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Npc {
    #[layer(gds = "95/20", primary)]
    pub drawing: NpcDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Li1 {
    #[layer(gds = "67/20", primary)]
    pub drawing: Li1Drawing,
    #[layer(gds = "67/16", pin)]
    pub pin: Li1Pin,
    #[layer(gds = "67/13")]
    pub res: Li1Res,
    #[layer(gds = "67/14")]
    pub cut: Li1Cut,
    #[layer(gds = "67/5", label)]
    pub label: Li1Label,
    #[layer(gds = "67/23")]
    pub net: Li1Net,
    #[layer(gds = "67/4")]
    pub boundary: Li1Boundary,
    #[layer(gds = "67/10")]
    pub blockage: Li1Blockage,
    #[layer(gds = "67/15")]
    pub short: Li1Short,
    #[layer(gds = "67/25")]
    pub probe: Li1Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Mcon {
    #[layer(gds = "67/44", primary)]
    pub drawing: MconDrawing,
    #[layer(gds = "67/60")]
    pub boundary: MconBoundary,
    #[layer(gds = "67/48", pin)]
    pub pin: MconPin,
    #[layer(gds = "67/41")]
    pub net: MconNet,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met1 {
    #[layer(gds = "68/20", primary)]
    pub drawing: Met1Drawing,
    #[layer(gds = "68/16", pin)]
    pub pin: Met1Pin,
    #[layer(gds = "68/13")]
    pub res: Met1Res,
    #[layer(gds = "68/14")]
    pub cut: Met1Cut,
    #[layer(gds = "68/5", label)]
    pub label: Met1Label,
    #[layer(gds = "68/23")]
    pub net: Met1Net,
    #[layer(gds = "68/4")]
    pub boundary: Met1Boundary,
    #[layer(gds = "68/10")]
    pub blockage: Met1Blockage,
    #[layer(gds = "68/15")]
    pub short: Met1Short,
    #[layer(gds = "68/25")]
    pub probe: Met1Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Via {
    #[layer(gds = "68/44", primary)]
    pub drawing: ViaDrawing,
    #[layer(gds = "68/58", pin)]
    pub pin: ViaPin,
    #[layer(gds = "68/41")]
    pub net: ViaNet,
    #[layer(gds = "68/60")]
    pub boundary: ViaBoundary,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met2 {
    #[layer(gds = "69/20", primary)]
    pub drawing: Met2Drawing,
    #[layer(gds = "69/16", pin)]
    pub pin: Met2Pin,
    #[layer(gds = "69/13")]
    pub res: Met2Res,
    #[layer(gds = "69/14")]
    pub cut: Met2Cut,
    #[layer(gds = "69/5", label)]
    pub label: Met2Label,
    #[layer(gds = "69/23")]
    pub net: Met2Net,
    #[layer(gds = "69/4")]
    pub boundary: Met2Boundary,
    #[layer(gds = "69/10")]
    pub blockage: Met2Blockage,
    #[layer(gds = "69/15")]
    pub short: Met2Short,
    #[layer(gds = "69/25")]
    pub probe: Met2Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Via2 {
    #[layer(gds = "69/44", primary)]
    pub drawing: Via2Drawing,
    #[layer(gds = "69/58", pin)]
    pub pin: Via2Pin,
    #[layer(gds = "69/41")]
    pub net: Via2Net,
    #[layer(gds = "69/60")]
    pub boundary: Via2Boundary,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met3 {
    #[layer(gds = "70/20", primary)]
    pub drawing: Met3Drawing,
    #[layer(gds = "70/16", pin)]
    pub pin: Met3Pin,
    #[layer(gds = "70/13")]
    pub res: Met3Res,
    #[layer(gds = "70/14")]
    pub cut: Met3Cut,
    #[layer(gds = "70/5", label)]
    pub label: Met3Label,
    #[layer(gds = "70/23")]
    pub net: Met3Net,
    #[layer(gds = "70/4")]
    pub boundary: Met3Boundary,
    #[layer(gds = "70/10")]
    pub blockage: Met3Blockage,
    #[layer(gds = "70/15")]
    pub short: Met3Short,
    #[layer(gds = "70/17")]
    pub fuse: Met3Fuse,
    #[layer(gds = "70/25")]
    pub probe: Met3Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Via3 {
    #[layer(gds = "70/44", primary)]
    pub drawing: Via3Drawing,
    #[layer(gds = "70/48", pin)]
    pub pin: Via3Pin,
    #[layer(gds = "70/41")]
    pub net: Via3Net,
    #[layer(gds = "70/60")]
    pub boundary: Via3Boundary,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met4 {
    #[layer(gds = "71/20", primary)]
    pub drawing: Met4Drawing,
    #[layer(gds = "71/16", pin)]
    pub pin: Met4Pin,
    #[layer(gds = "71/13")]
    pub res: Met4Res,
    #[layer(gds = "71/14")]
    pub cut: Met4Cut,
    #[layer(gds = "71/5", label)]
    pub label: Met4Label,
    #[layer(gds = "71/23")]
    pub net: Met4Net,
    #[layer(gds = "71/4")]
    pub boundary: Met4Boundary,
    #[layer(gds = "71/10")]
    pub blockage: Met4Blockage,
    #[layer(gds = "71/15")]
    pub short: Met4Short,
    #[layer(gds = "71/17")]
    pub fuse: Met4Fuse,
    #[layer(gds = "71/25")]
    pub probe: Met4Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Via4 {
    #[layer(gds = "71/44", primary)]
    pub drawing: Via4Drawing,
    #[layer(gds = "71/48", pin)]
    pub pin: Via4Pin,
    #[layer(gds = "71/41")]
    pub net: Via4Net,
    #[layer(gds = "71/60")]
    pub boundary: Via4Boundary,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Met5 {
    #[layer(gds = "72/20", primary)]
    pub drawing: Met5Drawing,
    #[layer(gds = "72/16", pin)]
    pub pin: Met5Pin,
    #[layer(gds = "72/13")]
    pub res: Met5Res,
    #[layer(gds = "72/14")]
    pub cut: Met5Cut,
    #[layer(gds = "72/5", label)]
    pub label: Met5Label,
    #[layer(gds = "72/23")]
    pub net: Met5Net,
    #[layer(gds = "72/4")]
    pub boundary: Met5Boundary,
    #[layer(gds = "72/10")]
    pub blockage: Met5Blockage,
    #[layer(gds = "72/15")]
    pub short: Met5Short,
    #[layer(gds = "72/17")]
    pub fuse: Met5Fuse,
    #[layer(gds = "72/25")]
    pub probe: Met5Probe,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Pad {
    #[layer(gds = "76/20", primary)]
    pub drawing: PadDrawing,
    #[layer(gds = "76/16", pin)]
    pub pin: PadPin,
    #[layer(gds = "76/5", label)]
    pub pad_label: PadLabel,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Rpm {
    #[layer(gds = "86/20", primary)]
    pub drawing: RpmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Urpm {
    #[layer(gds = "79/20", primary)]
    pub drawing: UrpmDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Hvi {
    #[layer(gds = "75/20", primary)]
    pub drawing: HviDrawing,
}

#[derive(LayerFamily, Clone, Copy)]
pub struct Ncm {
    #[layer(gds = "92/44", primary)]
    pub drawing: NcmDrawing,
}
