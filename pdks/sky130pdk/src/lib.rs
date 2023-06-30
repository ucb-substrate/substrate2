use substrate::pdk::Pdk;
use substrate::Layers;

#[derive(Debug, Default, Clone)]
pub struct Sky130Pdk {}

impl Sky130Pdk {
    #[inline]
    pub fn new() -> Self {
        Self {}
    }
}

impl Pdk for Sky130Pdk {
    type Layers = Sky130Layers;
}

#[derive(Layers)]
pub struct Sky130Layers {
    #[layer(gds = "235/4")]
    pub pr_boundary: PrBoundary,

    #[layer(gds = "64/44")]
    #[pin(pin = "pwell_pin", label = "pwell_label")]
    pub pwell: Pwell,
    #[layer(alias = "pwell")]
    pub pwell_drawing: Pwell,
    #[layer(gds = "122/16")]
    pub pwell_pin: PwellPin,
    #[layer(gds = "64/59")]
    pub pwell_label: PwellLabel,
    #[layer(gds = "64/13")]
    pub pwell_res: PwellRes,
    #[layer(gds = "64/14")]
    pub pwell_cut: PwellCut,

    #[layer(gds = "64/20")]
    #[pin(pin = "nwell_pin", label = "nwell_label")]
    pub nwell: Nwell,
    #[layer(alias = "nwell")]
    pub nwell_drawing: Nwell,
    #[layer(gds = "84/23")]
    pub nwell_net: NwellNet,
    #[layer(gds = "64/16")]
    pub nwell_pin: NwellPin,
    #[layer(gds = "64/5")]
    pub nwell_label: NwellLabel,

    #[layer(gds = "64/18")]
    pub dnwell: Dnwell,
    #[layer(alias = "dnwell")]
    pub dnwell_drawing: Dnwell,

    #[layer(gds = "74/21")]
    pub vhvi: Vhvi,
    #[layer(alias = "vhvi")]
    pub vhvi_drawing: Vhvi,

    #[layer(gds = "65/20")]
    #[pin(pin = "diff_pin", label = "diff_label")]
    pub diff: Diff,
    #[layer(alias = "diff")]
    pub diff_drawing: Diff,
    #[layer(gds = "65/13")]
    pub diff_res: DiffRes,
    #[layer(gds = "65/14")]
    pub diff_cut: DiffCut,
    #[layer(gds = "65/16")]
    pub diff_pin: DiffPin,
    #[layer(gds = "65/6")]
    pub diff_label: DiffLabel,
    #[layer(gds = "65/23")]
    pub diff_net: DiffNet,
    #[layer(gds = "65/4")]
    pub diff_boundary: DiffBoundary,
    #[layer(gds = "65/8")]
    pub diff_hv: DiffHv,

    #[layer(gds = "65/44")]
    #[pin(pin = "tap_pin", label = "tap_label")]
    pub tap: Tap,
    #[layer(alias = "tap")]
    pub tap_drawing: Tap,
    #[layer(gds = "65/48")]
    pub tap_pin: TapPin,
    #[layer(gds = "65/5")]
    pub tap_label: TapLabel,
    #[layer(gds = "65/41")]
    pub tap_net: TapNet,
    #[layer(gds = "65/60")]
    pub tap_boundary: TapBoundary,

    #[layer(gds = "94/20")]
    pub psdm: Psdm,
    #[layer(alias = "psdm")]
    pub psdm_drawing: Psdm,

    #[layer(gds = "93/44")]
    pub nsdm: Nsdm,
    #[layer(alias = "nsdm")]
    pub nsdm_drawing: Nsdm,

    #[layer(gds = "66/20")]
    #[pin(pin = "poly_pin", label = "poly_label")]
    pub poly: Poly,
    #[layer(alias = "poly")]
    pub poly_drawing: Poly,
    #[layer(gds = "66/16")]
    pub poly_pin: PolyPin,
    #[layer(gds = "66/13")]
    pub poly_res: PolyRes,
    #[layer(gds = "66/14")]
    pub poly_cut: PolyCut,
    #[layer(gds = "66/9")]
    pub poly_gate: PolyGate,
    #[layer(gds = "66/5")]
    pub poly_label: PolyLabel,
    #[layer(gds = "66/4")]
    pub poly_boundary: PolyBoundary,
    #[layer(gds = "66/25")]
    pub poly_probe: PolyProbe,
    #[layer(gds = "66/23")]
    pub poly_net: PolyNet,
    #[layer(gds = "66/83")]
    pub poly_model: PolyModel,

    #[layer(gds = "11/44")]
    pub ldntm: Ldntm,
    #[layer(alias = "ldntm")]
    pub ldntm_drawing: Ldntm,

    #[layer(gds = "125/44")]
    pub lvtn: Lvtn,
    #[layer(alias = "lvtn")]
    pub lvtn_drawing: Lvtn,

    #[layer(gds = "78/44")]
    pub hvtp: Hvtp,
    #[layer(alias = "hvtp")]
    pub hvtp_drawing: Hvtp,

    #[layer(gds = "18/20")]
    pub hvtr: Hvtr,
    #[layer(alias = "hvtr")]
    pub hvtr_drawing: Hvtr,

    #[layer(gds = "80/20")]
    pub tunm: Tunm,
    #[layer(alias = "tunm")]
    pub tunm_drawing: Tunm,

    #[layer(gds = "66/44")]
    pub licon1: Licon1,
    #[layer(alias = "licon1")]
    pub licon1_drawing: Licon1,

    /// Nitride poly cut.
    ///
    /// Required to form poly gate contacts.
    #[layer(gds = "95/20")]
    pub npc: Npc,
    #[layer(alias = "npc")]
    pub npc_drawing: Npc,

    #[layer(gds = "67/20")]
    #[pin(pin = "li1_pin", label = "li1_label")]
    pub li1: Li1,
    #[layer(alias = "li1")]
    pub li1_drawing: Li1,
    #[layer(gds = "67/16")]
    pub li1_pin: Li1Pin,
    #[layer(gds = "67/13")]
    pub li1_res: Li1Res,
    #[layer(gds = "67/14")]
    pub li1_cut: Li1Cut,
    #[layer(gds = "67/5")]
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

    #[layer(gds = "67/44")]
    #[pin(pin = "mcon_pin", label = "mcon_pin")]
    pub mcon: Mcon,
    #[layer(alias = "mcon")]
    pub mcon_drawing: Mcon,
    #[layer(gds = "67/60")]
    pub mcon_boundary: MconBoundary,
    #[layer(gds = "67/48")]
    pub mcon_pin: MconPin,
    #[layer(gds = "67/41")]
    pub mcon_net: MconNet,

    #[layer(gds = "68/20")]
    #[pin(pin = "met1_pin", label = "met1_label")]
    pub met1: Met1,
    #[layer(alias = "met1")]
    pub met1_drawing: Met1,
    #[layer(gds = "68/16")]
    pub met1_pin: Met1Pin,
    #[layer(gds = "68/13")]
    pub met1_res: Met1Res,
    #[layer(gds = "68/14")]
    pub met1_cut: Met1Cut,
    #[layer(gds = "68/5")]
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

    #[layer(gds = "68/44")]
    #[pin(pin = "via_pin", label = "via_pin")]
    pub via: Via,
    #[layer(alias = "via")]
    pub via_drawing: Via,
    #[layer(gds = "68/58")]
    pub via_pin: ViaPin,
    #[layer(gds = "68/41")]
    pub via_net: ViaNet,
    #[layer(gds = "68/60")]
    pub via_boundary: ViaBoundary,

    #[layer(gds = "69/20")]
    #[pin(pin = "met2_pin", label = "met2_label")]
    pub met2: Met2,
    #[layer(alias = "met2")]
    pub met2_drawing: Met2,
    #[layer(gds = "69/16")]
    pub met2_pin: Met2Pin,
    #[layer(gds = "69/13")]
    pub met2_res: Met2Res,
    #[layer(gds = "69/14")]
    pub met2_cut: Met2Cut,
    #[layer(gds = "69/5")]
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

    #[layer(gds = "69/44")]
    #[pin(pin = "via2_pin", label = "via2_pin")]
    pub via2: Via2,
    #[layer(alias = "via2")]
    pub via2_drawing: Via2,
    #[layer(gds = "69/58")]
    pub via2_pin: Via2Pin,
    #[layer(gds = "69/41")]
    pub via2_net: Via2Net,
    #[layer(gds = "69/60")]
    pub via2_boundary: Via2Boundary,

    #[layer(gds = "70/20")]
    #[pin(pin = "met3_pin", label = "met3_label")]
    pub met3: Met3,
    #[layer(alias = "met3")]
    pub met3_drawing: Met3,
    #[layer(gds = "70/16")]
    pub met3_pin: Met3Pin,
    #[layer(gds = "70/13")]
    pub met3_res: Met3Res,
    #[layer(gds = "70/14")]
    pub met3_cut: Met3Cut,
    #[layer(gds = "70/5")]
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

    #[layer(gds = "70/44")]
    #[pin(pin = "via3", label = "via3")]
    pub via3: Via3,
    #[layer(alias = "via3")]
    pub via3_drawing: Via3,
    #[layer(gds = "70/48")]
    pub via3_pin: Via3Pin,
    #[layer(gds = "70/41")]
    pub via3_net: Via3Net,
    #[layer(gds = "70/60")]
    pub via3_boundary: Via3Boundary,

    #[layer(gds = "71/20")]
    #[pin(pin = "met4_pin", label = "met4_label")]
    pub met4: Met4,
    #[layer(alias = "met4")]
    pub met4_drawing: Met4,
    #[layer(gds = "71/16")]
    pub met4_pin: Met4Pin,
    #[layer(gds = "71/13")]
    pub met4_res: Met4Res,
    #[layer(gds = "71/14")]
    pub met4_cut: Met4Cut,
    #[layer(gds = "71/5")]
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

    #[layer(gds = "71/44")]
    #[pin(pin = "via4", label = "via4")]
    pub via4: Via4,
    #[layer(alias = "via4")]
    pub via4_drawing: Via4,
    #[layer(gds = "71/48")]
    pub via4_pin: Via4Pin,
    #[layer(gds = "71/41")]
    pub via4_net: Via4Net,
    #[layer(gds = "71/60")]
    pub via4_boundary: Via4Boundary,

    #[layer(gds = "72/20")]
    #[pin(pin = "met5_pin", label = "met5_label")]
    pub met5: Met5,
    #[layer(alias = "met5")]
    pub met5_drawing: Met5,
    #[layer(gds = "72/16")]
    pub met5_pin: Met5Pin,
    #[layer(gds = "72/13")]
    pub met5_res: Met5Res,
    #[layer(gds = "72/14")]
    pub met5_cut: Met5Cut,
    #[layer(gds = "72/5")]
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

    #[layer(gds = "76/20")]
    #[pin(pin = "pad_pin", label = "pad_label")]
    pub pad: Pad,
    #[layer(alias = "pad")]
    pub pad_drawing: Pad,
    #[layer(gds = "76/16")]
    pub pad_pin: PadPin,
    #[layer(gds = "76/5")]
    pub pad_label: PadLabel,

    #[layer(gds = "86/20")]
    pub rpm: Rpm,
    #[layer(alias = "rpm")]
    pub rpm_drawing: Rpm,

    #[layer(gds = "79/20")]
    pub urpm: Urpm,
    #[layer(alias = "urpm")]
    pub urpm_drawing: Urpm,

    #[layer(gds = "75/20")]
    pub hvi: Hvi,
    #[layer(alias = "hvi")]
    pub hvi_drawing: Hvi,

    #[layer(gds = "92/44")]
    pub ncm: Ncm,
    #[layer(alias = "ncm")]
    pub ncm_drawing: Ncm,

    #[layer(gds = "22/20")]
    pub cfom_drawing: CfomDrawing,
    #[layer(gds = "23/0")]
    pub cfom_mask: CfomMask,
    #[layer(gds = "22/21")]
    pub cfom_mask_add: CfomMaskAdd,
    #[layer(gds = "22/22")]
    pub cfom_mask_drop: CfomMaskDrop,

    #[layer(gds = "115/44")]
    pub cli1m_drawing: Cli1mDrawing,
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
