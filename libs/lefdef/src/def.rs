use geometry::point::Point;
use geometry::polygon::Polygon;
use geometry::rect::Rect;
use std::fmt::{Display, Formatter};
use std::io::Write;

pub type Ident = String;
pub type Pattern = String;
pub type NonEmptyVec<T> = Vec<T>;

pub struct Def {
    version: Option<Version>,
    divider_char: Option<DividerChar>,
    bus_bit_chars: Option<BusBitChars>,
    design: Ident,
    units: Option<Units>,
    die_area: Option<DieArea>,
    vias: Option<Vias>,
    components: Option<Components>,
    blockages: Option<Blockages>,
    special_nets: Option<SpecialNets>,
    nets: Option<Nets>,
}

pub trait WriteDef {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()>;
}

impl WriteDef for Def {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        if let Some(ref version) = self.version {
            writeln!(out, "VERSION {} ;", version.0)?;
        }
        if let Some(ref dc) = self.divider_char {
            writeln!(out, "DIVIDERCHAR \"{}\" ;", dc.divider)?;
        }
        if let Some(ref bc) = self.bus_bit_chars {
            writeln!(out, "BUSBITCHARS \"{}{}\" ;", bc.open, bc.close)?;
        }

        writeln!(out, "DESIGN {} ;", self.design)?;

        if let Some(ref units) = self.units {
            writeln!(out, "UNITS DISTANCE MICRONS {} ;", units.dbu_per_micron)?;
        }

        if let Some(ref d) = self.die_area {
            d.write(out)?;
        }

        if let Some(ref vias) = self.vias {
            vias.write(out)?;
        }

        if let Some(ref components) = self.components {
            components.write(out)?;
        }

        if let Some(ref blockages) = self.blockages {
            blockages.write(out)?;
        }

        if let Some(ref sn) = self.special_nets {
            sn.write(out)?;
        }

        writeln!(out, "END DESIGN {}", self.design)?;

        Ok(())
    }
}

/// Blockages.
pub struct Blockages {
    layer_blockages: Vec<LayerBlockage>,
    placement_blockages: Vec<PlacementBlockage>,
}

impl Blockages {
    pub fn is_empty(&self) -> bool {
        self.layer_blockages.is_empty() && self.placement_blockages.is_empty()
    }

    pub fn len(&self) -> usize {
        self.layer_blockages.len() + self.placement_blockages.len()
    }
}

impl WriteDef for Blockages {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        if self.is_empty() {
            return Ok(());
        }
        writeln!(out, "BLOCKAGES {} ;", self.len())?;
        for b in self.layer_blockages.iter() {
            writeln!(out, "  - LAYER {}", b.layer)?;
            if let Some(kind) = &b.kind {
                write!(out, "    ")?;
                kind.write(out)?;
                writeln!(out)?;
            }
            if let Some(s) = &b.spacing {
                write!(out, "    ")?;
                s.write(out)?;
                writeln!(out)?;
            }
            if let Some(x) = &b.mask {
                writeln!(out, "    + MASK {}", x.0)?;
            }
            for geometry in b.geometry.iter() {
                write!(out, "    ")?;
                geometry.write(out)?;
            }
            writeln!(out, " ;")?;
        }
        for b in self.placement_blockages.iter() {
            writeln!(out, "  - PLACEMENT")?;
            if let Some(kind) = &b.kind {
                write!(out, "    ")?;
                kind.write(out)?;
                writeln!(out)?;
            }
            if b.pushdown {
                write!(out, "    + PUSHDOWN")?;
            }
            if let Some(x) = &b.component {
                writeln!(out, "    + COMPONENT {x}")?;
            }
            for rect in b.rects.iter() {
                write!(out, "    ")?;
                rect.write(out)?;
                writeln!(out)?;
            }
            writeln!(out, " ;")?;
        }
        writeln!(out, "END BLOCKAGES")?;
        Ok(())
    }
}

pub enum Blockage {
    Layer(LayerBlockage),
    Placement(PlacementBlockage),
}

pub struct LayerBlockage {
    layer: Ident,
    kind: Option<LayerBlockageKind>,
    spacing: Option<LayerBlockageSpacing>,
    mask: Option<MaskNum>,
    geometry: Vec<Geometry>,
}

pub enum LayerBlockageKind {
    Component(Ident),
    Slots,
    Fills,
    Pushdown,
    ExceptPgNet,
}

impl LayerBlockageKind {
    pub fn kind_as_str(&self) -> &'static str {
        match self {
            LayerBlockageKind::Component(_) => "COMPONENT",
            LayerBlockageKind::Slots => "SLOTS",
            LayerBlockageKind::Fills => "FILLS",
            LayerBlockageKind::Pushdown => "PUSHDOWN",
            LayerBlockageKind::ExceptPgNet => "EXCEPTPGNET",
        }
    }
}

impl WriteDef for LayerBlockageKind {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            LayerBlockageKind::Component(c) => write!(out, "+ COMPONENT {}", c),
            other => write!(out, "+ {}", other.kind_as_str()),
        }
    }
}

pub enum LayerBlockageSpacing {
    Spacing(i64),
    DesignRuleWidth(i64),
}

impl WriteDef for LayerBlockageSpacing {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            LayerBlockageSpacing::Spacing(x) => write!(out, "+ SPACING {x}"),
            LayerBlockageSpacing::DesignRuleWidth(x) => write!(out, "+ DESIGNRULEWIDTH {x}"),
        }
    }
}

pub struct MaskNum(u32);

pub enum Geometry {
    Rect(Rect),
    Polygon(Polygon),
}

impl WriteDef for Rect {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "RECT ")?;
        write_pt(self.lower_left(), out)?;
        write!(out, " ")?;
        write_pt(self.upper_right(), out)?;
        Ok(())
    }
}

impl WriteDef for Polygon {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "POLYGON")?;
        for pt in self.points() {
            write!(out, " ")?;
            write_pt(*pt, out)?;
        }
        Ok(())
    }
}

impl WriteDef for Geometry {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            Geometry::Rect(r) => {
                r.write(out)?;
            }
            Geometry::Polygon(p) => {
                p.write(out)?;
            }
        }
        Ok(())
    }
}

pub struct PlacementBlockage {
    kind: Option<PlacementBlockageKind>,
    pushdown: bool,
    component: Option<Ident>,
    rects: Vec<Rect>,
}

pub enum PlacementBlockageKind {
    Soft,
    Partial(f64),
}

impl WriteDef for PlacementBlockageKind {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            PlacementBlockageKind::Soft => write!(out, "+ SOFT"),
            PlacementBlockageKind::Partial(x) => write!(out, "+ PARTIAL {x}"),
        }
    }
}

pub struct DividerChar {
    pub divider: char,
}

impl Default for DividerChar {
    fn default() -> Self {
        Self { divider: '/' }
    }
}

pub struct BusBitChars {
    pub open: char,
    pub close: char,
}

impl Default for BusBitChars {
    fn default() -> Self {
        Self {
            open: '[',
            close: ']',
        }
    }
}

pub struct ComponentMaskShift {
    /// Note: cannot be empty.
    layers: NonEmptyVec<Ident>,
}

pub struct Components {
    components: Vec<Component>,
}

pub struct Component {
    comp_name: Ident,
    model_name: Ident,
    eeq_master: Option<Ident>,
    source: Option<Source>,
    placement_status: Option<PlacementStatus>,
    mask_shift: Option<MaskNum>,
    halo: Option<ComponentPlaceHalo>,
    route_halo: Option<ComponentRouteHalo>,
    weight: Option<f64>,
    region: Option<Ident>,
    properties: Vec<Property>,
}

pub enum KnownPlacementKind {
    Fixed,
    Cover,
    Placed,
}

pub struct KnownPlacement {
    kind: KnownPlacementKind,
    pt: Point,
    orient: Orientation,
}

pub enum PlacementStatus {
    Fixed { pt: Point, orient: Orientation },
    Cover { pt: Point, orient: Orientation },
    Placed { pt: Point, orient: Orientation },
    Unplaced,
}

impl WriteDef for PlacementStatus {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            PlacementStatus::Fixed { pt, orient } => {
                write!(out, "+ FIXED ")?;
                write_pt(*pt, out)?;
                writeln!(out, "{orient}")?;
            }
            PlacementStatus::Cover { pt, orient } => {
                write!(out, "+ COVER ")?;
                write_pt(*pt, out)?;
                writeln!(out, "{orient}")?;
            }
            PlacementStatus::Placed { pt, orient } => {
                write!(out, "+ PLACED ")?;
                write_pt(*pt, out)?;
                writeln!(out, "{orient}")?;
            }
            PlacementStatus::Unplaced => {
                writeln!(out, "+ UNPLACED")?;
            }
        }
        Ok(())
    }
}

pub struct Property {
    name: String,
    val: String,
}

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub enum Orientation {
    /// North (R0).
    #[default]
    N,
    /// South (R180).
    S,
    /// West (R90).
    W,
    /// East (R270).
    E,
    /// Flipped north (MY).
    Fn,
    /// Flipped south (MX).
    Fs,
    /// Flipped west (MX90).
    Fw,
    /// Flipped east (MY90).
    Fe,
}

impl Orientation {
    pub fn as_str(&self) -> &'static str {
        match self {
            Orientation::N => "N",
            Orientation::S => "S",
            Orientation::W => "W",
            Orientation::E => "E",
            Orientation::Fn => "FN",
            Orientation::Fs => "FS",
            Orientation::Fw => "FW",
            Orientation::Fe => "FE",
        }
    }
}

impl Display for Orientation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Copy, Clone)]
pub enum Source {
    Netlist,
    Dist,
    User,
    Timing,
}

impl Source {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Source::Netlist => "NETLIST",
            Source::Dist => "DIST",
            Source::User => "USER",
            Source::Timing => "TIMING",
        }
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub enum NetSource {
    Dist,
    Netlist,
    Test,
    Timing,
    User,
}

impl NetSource {
    pub fn as_str(&self) -> &'static str {
        match *self {
            NetSource::Dist => "DIST",
            NetSource::Netlist => "NETLIST",
            NetSource::Test => "TEST",
            NetSource::Timing => "TIMING",
            NetSource::User => "USER",
        }
    }
}

impl Display for NetSource {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

pub struct ComponentPlaceHalo {
    soft: bool,
    left: i64,
    bottom: i64,
    right: i64,
    top: i64,
}

pub struct ComponentRouteHalo {
    halo_dist: i64,
    min_layer: Ident,
    max_layer: Ident,
}

pub struct DieArea {
    /// Must have at least 2 points.
    pts: Vec<Point>,
}

pub fn write_pt<W: Write>(pt: Point, out: &mut W) -> std::io::Result<()> {
    write!(out, "( {} {} )", pt.x, pt.y)
}

impl WriteDef for DieArea {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        assert!(self.pts.len() >= 2);
        write!(out, "DIEAREA")?;
        for pt in self.pts.iter() {
            write!(out, " ")?;
            write_pt(*pt, out)?;
        }
        writeln!(out, " ;")?;
        Ok(())
    }
}

pub struct Groups {
    groups: Vec<Group>,
}

pub struct Group {
    name: Ident,
    comp_name_patterns: Vec<Pattern>,
    region: Option<Ident>,
    properties: Vec<Property>,
}

pub struct Nets {
    nets: Vec<Net>,
}

pub struct Net {
    ident: NetIdent,
    shield_nets: Vec<Ident>,
    virtual_pins: Vec<VirtualPin>,
    subnets: Vec<Subnet>,
    xtalk: Option<u8>,
    nondefault_rule: Option<Ident>,
    wiring: Vec<RegularWiring>,
    source: Option<NetSource>,
    fixed_bump: bool,
    frequency: Option<f64>,
    original: Option<Ident>,
    net_type: NetType,
    pattern: Option<NetPattern>,
    est_cap: Option<f64>,
    weight: Option<u32>,
    properties: Vec<Property>,
}

pub enum NetType {
    Analog,
    Clock,
    Ground,
    Power,
    Reset,
    Scan,
    Signal,
    Tieoff,
}

pub enum NetPattern {
    Balanced,
    Steiner,
    Trunk,
    WiredLogic,
}

pub enum NetIdent {
    Named(NamedNetIdent),
    MustJoin { component: Ident, pin: Ident },
}

pub struct NetPin {
    kind: NetPinKind,
    synthesized: bool,
}

impl WriteDef for NetPin {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "( ")?;
        self.kind.write(out)?;
        if self.synthesized {
            write!(out, " + SYNTHESIZED")?;
        }
        write!(out, " )")?;
        Ok(())
    }
}

pub enum NetPinKind {
    ComponentPin { comp_name: Ident, pin_name: Ident },
    IoPin { name: Ident },
}

impl WriteDef for NetPinKind {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        todo!()
    }
}

pub struct VirtualPin {
    name: Ident,
    layer: Option<Ident>,
    p0: Point,
    p1: Point,
    placement: Option<KnownPlacement>,
}

pub struct Subnet {
    name: Ident,
    pins: Vec<SubnetPin>,
}

pub enum SubnetPin {
    Component { comp_name: Ident, pin_name: Ident },
    IoPin { name: Ident },
    VirtualPin { name: Ident },
}

pub struct RoutingXy {
    x: i64,
    y: i64,
    ext: i64,
}

pub struct RoutingPoints {
    start: RoutingXy,
    points: Vec<RoutingPoint>,
}

pub enum RoutingPoint {
    Point {
        mask: Option<MaskNum>,
        pt: RoutingXy,
    },
    Via {
        mask: Option<MaskNum>,
        via_name: Ident,
        orient: Orientation,
    },
    Rect {
        mask: Option<MaskNum>,
        dx1: i64,
        dy1: i64,
        dx2: i64,
        dy2: i64,
    },
    Virtual {
        x: i64,
        y: i64,
    },
}

pub enum RoutingStatus {
    Cover,
    Fixed,
    Routed,
    NoShield,
}

pub enum Taper {
    Default,
    Rule(Ident),
}

pub struct RegularWiring {
    status: RoutingStatus,
    entries: Vec<RegularWiringEntry>,
}

pub struct RegularWiringEntry {
    layer: Ident,
    taper: Option<Taper>,
    style: Option<u32>,
    points: RoutingPoints,
}

pub struct SpecialNets {
    nets: Vec<SpecialNet>,
}

impl WriteDef for SpecialNets {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        if self.nets.is_empty() {
            return Ok(());
        }
        writeln!(out, "SPECIALNETS {} ;", self.nets.len())?;
        for n in self.nets.iter() {
            writeln!(out, "  - {}", n.name.name)?;
            for pin in n.name.pins {
                pin.write(out)?;
            }
            if let Some(eeq) = &c.eeq_master {
                writeln!(out, "    + EEQMASTER {}", eeq)?;
            }
            if let Some(s) = &c.source {
                writeln!(out, "    + SOURCE {}", s)?;
            }
            if let Some(x) = &c.placement_status {
                write!(out, "    ")?;
                x.write(out)?;
            }
            if let Some(x) = &c.mask_shift {
                writeln!(out, "    + MASKSHIFT {}", x.0)?;
            }
            if let Some(x) = &c.halo {
                let soft = if x.soft { " SOFT" } else { "" };
                writeln!(
                    out,
                    "    + HALO{} {} {} {} {}",
                    soft, x.left, x.bottom, x.right, x.top
                )?;
            }
            if let Some(x) = &c.route_halo {
                writeln!(
                    out,
                    "    + ROUTEHALO {} {} {}",
                    x.halo_dist, x.min_layer, x.max_layer
                )?;
            }
            if let Some(x) = &c.weight {
                writeln!(out, "    + WEIGHT {}", x)?;
            }
            if let Some(x) = &c.region {
                writeln!(out, "    + REGION {}", x)?;
            }
            if !c.properties.is_empty() {
                write!(out, "    + PROPERTY")?;
                for p in c.properties.iter() {
                    write!(out, "\"{}\" \"{}\"", p.name, p.val)?;
                }
            }
            writeln!(out, "    ;")?;
        }
        writeln!(out, "END COMPONENTS")?;
        Ok(())
    }
}

pub struct NamedNetIdent {
    name: Ident,
    pins: Vec<NetPin>,
}

pub struct SpecialNet {
    name: NamedNetIdent,
    /// Voltage in mV.
    ///
    /// Example: + VOLTAGE 3300 means 3.3V.
    voltage: Option<i64>,
    wiring: Vec<SpecialWiring>,
    source: Source,
    fixed_bump: bool,
    original: Option<Ident>,
    net_type: NetType,
    pattern: Option<NetPattern>,
    est_cap: Option<f64>,
    weight: Option<u32>,
    properties: Vec<Property>,
}

pub enum SpecialWiring {
    Geometry(GeometrySpecialWiring),
    Path(PathSpecialWiring),
}

pub struct GeometrySpecialWiring {
    status: Option<SpecialRoutingStatus>,
    shape: Option<ShapeType>,
    mask: Option<MaskNum>,
    entries: Vec<GeometrySpecialWiringEntry>,
}

pub struct LayerRect {
    pub layer: Ident,
    pub mask: Option<MaskNum>,
    pub rect: Rect,
}

pub struct LayerPolygon {
    pub layer: Ident,
    pub mask: Option<MaskNum>,
    pub polygon: Polygon,
}

pub enum GeometrySpecialWiringEntry {
    Rect(LayerRect),
    Polygon(LayerPolygon),
    Via {
        name: Ident,
        orient: Option<Orientation>,
        point: Point,
    },
}

pub enum ShapeType {
    Ring,
    PadRing,
    BlockRing,
    Stripe,
    FollowPin,
    IoWire,
    CoreWire,
    BlockWire,
    BlockageWire,
    FillWire,
    FillWireOpc,
    DrcFill,
}

pub struct PathSpecialWiring {
    status: SpecialRoutingStatus,
    entries: NonEmptyVec<PathSpecialWiringEntry>,
}

pub struct PathSpecialWiringEntry {
    layer: Ident,
    width: i64,
    shape: Option<ShapeType>,
    style: Option<u32>,
    points: RoutingPoints,
}

pub enum SpecialRoutingStatus {
    Cover,
    Fixed,
    Routed,
    Shield { net: Ident },
}

pub struct Vias {
    vias: Vec<Via>,
}

impl WriteDef for LayerRect {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "+ RECT {}", self.layer)?;
        if let Some(mask) = &self.mask {
            write!(out, "+ MASK {}", mask.0)?;
        }
        write!(out, " ")?;
        write_pt(self.rect.lower_left(), out)?;
        write!(out, " ")?;
        write_pt(self.rect.upper_right(), out)?;
        Ok(())
    }
}

impl WriteDef for LayerPolygon {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "+ POLYGON {}", self.layer)?;
        if let Some(mask) = &self.mask {
            write!(out, "+ MASK {}", mask.0)?;
        }
        for pt in self.polygon.points() {
            write!(out, " ")?;
            write_pt(*pt, out)?;
        }
        Ok(())
    }
}

impl WriteDef for ViaGeometry {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            ViaGeometry::Rect(x) => x.write(out),
            ViaGeometry::Polygon(x) => x.write(out),
        }
    }
}

impl WriteDef for Vias {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        if self.vias.is_empty() {
            return Ok(());
        }
        writeln!(out, "VIAS {} ;", self.vias.len())?;
        for via in self.vias.iter() {
            writeln!(out, "  - {}", via.name)?;
            match &via.definition {
                ViaDef::Fixed(v) => {
                    for shape in &v.geometry {
                        write!(out, "    ")?;
                        shape.write(out)?;
                        writeln!(out)?;
                    }
                }
                ViaDef::ViaRule(v) => {
                    writeln!(out, "    + VIARULE {}", v.via_rule_name)?;
                    writeln!(out, "    + CUTSIZE {} {}", v.cut_size_x, v.cut_size_y)?;
                    writeln!(
                        out,
                        "    + LAYERS {} {} {}",
                        v.bot_metal_layer, v.cut_layer, v.top_metal_layer
                    )?;
                    writeln!(
                        out,
                        "    + CUTSPACING {} {}",
                        v.cut_spacing_x, v.cut_spacing_y
                    )?;
                    writeln!(
                        out,
                        "    + ENCLOSURE {} {} {} {}",
                        v.bot_enc_x, v.bot_enc_y, v.top_enc_x, v.top_enc_y
                    )?;
                    if let Some((r, c)) = &v.rowcol {
                        writeln!(out, "    + ROWCOL {} {}", *r, *c)?;
                    }
                    if let Some(ref pt) = v.origin {
                        writeln!(out, "    + ORIGIN {} {}", pt.x, pt.y)?;
                    }
                    if let Some(ref ofs) = v.offset {
                        writeln!(
                            out,
                            "    + OFFSET {} {} {} {}",
                            ofs.bot_ofs_x, ofs.bot_ofs_y, ofs.top_ofs_x, ofs.top_ofs_y
                        )?;
                    }
                    if let Some(ref pat) = v.pattern {
                        writeln!(out, "    + PATTERN {}", pat)?;
                    }
                }
            }
            writeln!(out, "    ;")?;
        }
        writeln!(out, "END VIAS")?;
        Ok(())
    }
}

impl WriteDef for Components {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        if self.components.is_empty() {
            return Ok(());
        }
        writeln!(out, "COMPONENTS {} ;", self.components.len())?;
        for c in self.components.iter() {
            writeln!(out, "  - {} {}", c.comp_name, c.model_name)?;
            if let Some(eeq) = &c.eeq_master {
                writeln!(out, "    + EEQMASTER {}", eeq)?;
            }
            if let Some(s) = &c.source {
                writeln!(out, "    + SOURCE {}", s)?;
            }
            if let Some(x) = &c.placement_status {
                write!(out, "    ")?;
                x.write(out)?;
            }
            if let Some(x) = &c.mask_shift {
                writeln!(out, "    + MASKSHIFT {}", x.0)?;
            }
            if let Some(x) = &c.halo {
                let soft = if x.soft { " SOFT" } else { "" };
                writeln!(
                    out,
                    "    + HALO{} {} {} {} {}",
                    soft, x.left, x.bottom, x.right, x.top
                )?;
            }
            if let Some(x) = &c.route_halo {
                writeln!(
                    out,
                    "    + ROUTEHALO {} {} {}",
                    x.halo_dist, x.min_layer, x.max_layer
                )?;
            }
            if let Some(x) = &c.weight {
                writeln!(out, "    + WEIGHT {}", x)?;
            }
            if let Some(x) = &c.region {
                writeln!(out, "    + REGION {}", x)?;
            }
            if !c.properties.is_empty() {
                write!(out, "    + PROPERTY")?;
                for p in c.properties.iter() {
                    write!(out, "\"{}\" \"{}\"", p.name, p.val)?;
                }
            }
            writeln!(out, "    ;")?;
        }
        writeln!(out, "END COMPONENTS")?;
        Ok(())
    }
}

pub struct Via {
    pub name: Ident,
    pub definition: ViaDef,
}

pub enum ViaDef {
    Fixed(FixedVia),
    ViaRule(Box<ViaRuleVia>),
}

pub struct ViaRuleVia {
    pub via_rule_name: Ident,
    pub cut_size_x: i64,
    pub cut_size_y: i64,
    pub bot_metal_layer: Ident,
    pub cut_layer: Ident,
    pub top_metal_layer: Ident,
    pub cut_spacing_x: i64,
    pub cut_spacing_y: i64,
    pub bot_enc_x: i64,
    pub bot_enc_y: i64,
    pub top_enc_x: i64,
    pub top_enc_y: i64,
    pub rowcol: Option<(u32, u32)>,
    pub origin: Option<Point>,
    pub offset: Option<ViaOffset>,
    pub pattern: Option<String>,
}

pub struct ViaOffset {
    pub bot_ofs_x: i64,
    pub bot_ofs_y: i64,
    pub top_ofs_x: i64,
    pub top_ofs_y: i64,
}

pub struct FixedVia {
    pub geometry: Vec<ViaGeometry>,
}

pub enum ViaGeometry {
    Rect(LayerRect),
    Polygon(LayerPolygon),
}

pub struct Units {
    pub dbu_per_micron: i64,
}

pub struct Version(pub String);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_write_def() {
        let def = Def {
            version: Some(Version("5.8".to_string())),
            divider_char: Some(DividerChar { divider: '/' }),
            bus_bit_chars: Some(BusBitChars {
                open: '[',
                close: ']',
            }),
            design: "manual_routing".to_string(),
            units: Some(Units {
                dbu_per_micron: 1000,
            }),
            die_area: Some(DieArea {
                pts: vec![Point::zero(), Point::new(400000, 200000)],
            }),
            vias: Some(Vias {
                vias: vec![Via {
                    name: "v4_1".to_string(),
                    definition: ViaDef::ViaRule(Box::new(ViaRuleVia {
                        via_rule_name: "v4".to_string(),
                        cut_size_x: 4800,
                        cut_size_y: 4800,
                        bot_metal_layer: "m4".to_string(),
                        cut_layer: "v4".to_string(),
                        top_metal_layer: "m5".to_string(),
                        cut_spacing_x: 4000,
                        cut_spacing_y: 4000,
                        bot_enc_x: 2400,
                        bot_enc_y: 1200,
                        top_enc_x: 3000,
                        top_enc_y: 2000,
                        rowcol: Some((1, 2)),
                        origin: None,
                        offset: None,
                        pattern: None,
                    })),
                }],
            }),
            components: None,
            blockages: None,
            special_nets: None,
            nets: None,
        };

        let mut buf: Vec<u8> = Vec::new();
        def.write(&mut buf).expect("failed to write def");
        let s = String::from_utf8(buf).expect("failed to convert from utf8");
        println!("{s}");
    }
}
