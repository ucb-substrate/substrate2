use geometry::orientation::NamedOrientation;
use geometry::point::Point;
use geometry::polygon::Polygon;
use geometry::rect::Rect;
use std::fmt::{Display, Formatter};
use std::io::Write;
use std::path::Path;

pub type Ident = String;
pub type Pattern = String;
pub type NonEmptyVec<T> = Vec<T>;

#[derive(Debug, Clone)]
pub struct Def {
    pub version: Option<Version>,
    pub divider_char: Option<DividerChar>,
    pub bus_bit_chars: Option<BusBitChars>,
    pub design: Ident,
    pub units: Option<Units>,
    pub die_area: Option<DieArea>,
    pub vias: Option<Vias>,
    pub components: Option<Components>,
    pub blockages: Option<Blockages>,
    pub special_nets: Option<SpecialNets>,
    pub nets: Option<Nets>,
}

impl Def {
    pub fn new(name: impl Into<Ident>) -> Self {
        Self {
            version: Some(Version("5.8".into())),
            divider_char: Some(DividerChar { divider: '/' }),
            bus_bit_chars: Some(BusBitChars {
                open: '[',
                close: ']',
            }),
            design: name.into(),
            units: None,
            die_area: None,
            vias: None,
            components: None,
            blockages: None,
            special_nets: None,
            nets: None,
        }
    }

    /// Writes this DEF to a file.
    ///
    /// The parent directory will be created if it does not exist.
    pub fn write_to_file(&self, path: impl AsRef<Path>) -> std::io::Result<()> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::File::create(path)?;
        self.write(&mut file)?;
        Ok(())
    }
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

        if let Some(ref n) = self.nets {
            n.write(out)?;
        }

        write!(out, "END DESIGN {}", self.design)?;

        Ok(())
    }
}

/// Blockages.
#[derive(Debug, Clone)]
pub struct Blockages {
    pub layer_blockages: Vec<LayerBlockage>,
    pub placement_blockages: Vec<PlacementBlockage>,
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

#[derive(Debug, Clone)]
pub enum Blockage {
    Layer(LayerBlockage),
    Placement(PlacementBlockage),
}

#[derive(Debug, Clone)]
pub struct LayerBlockage {
    pub layer: Ident,
    pub kind: Option<LayerBlockageKind>,
    pub spacing: Option<LayerBlockageSpacing>,
    pub mask: Option<MaskNum>,
    pub geometry: Vec<Geometry>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct MaskNum(pub u32);

#[derive(Debug, Clone)]
pub enum Geometry {
    Rect(Rect),
    Polygon(Polygon),
}

impl WriteDef for Rect {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "RECT ")?;
        self.lower_left().write(out)?;
        write!(out, " ")?;
        self.upper_right().write(out)?;
        Ok(())
    }
}

impl WriteDef for Polygon {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "POLYGON")?;
        for pt in self.points() {
            write!(out, " ")?;
            pt.write(out)?;
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

#[derive(Debug, Clone)]
pub struct PlacementBlockage {
    pub kind: Option<PlacementBlockageKind>,
    pub pushdown: bool,
    pub component: Option<Ident>,
    pub rects: Vec<Rect>,
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct DividerChar {
    pub divider: char,
}

impl Default for DividerChar {
    fn default() -> Self {
        Self { divider: '/' }
    }
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Default)]
pub struct Components {
    pub components: Vec<Component>,
}

impl Components {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, component: Component) {
        self.components.push(component);
    }
}

#[derive(Debug, Clone)]
pub struct Component {
    pub comp_name: Ident,
    pub model_name: Ident,
    pub eeq_master: Option<Ident>,
    pub source: Option<Source>,
    pub placement_status: Option<PlacementStatus>,
    pub mask_shift: Option<MaskNum>,
    pub halo: Option<ComponentPlaceHalo>,
    pub route_halo: Option<ComponentRouteHalo>,
    pub weight: Option<f64>,
    pub region: Option<Ident>,
    pub properties: Vec<Property>,
}

impl Component {
    pub fn new(comp_name: impl Into<Ident>, model_name: impl Into<Ident>) -> Self {
        Self {
            comp_name: comp_name.into(),
            model_name: model_name.into(),
            eeq_master: None,
            source: None,
            placement_status: None,
            mask_shift: None,
            halo: None,
            route_halo: None,
            weight: None,
            region: None,
            properties: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum KnownPlacementKind {
    Fixed,
    Cover,
    Placed,
}

impl KnownPlacementKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            KnownPlacementKind::Fixed => "FIXED",
            KnownPlacementKind::Cover => "COVER",
            KnownPlacementKind::Placed => "PLACED",
        }
    }
}

impl Display for KnownPlacementKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl WriteDef for KnownPlacementKind {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct KnownPlacement {
    pub kind: KnownPlacementKind,
    pub pt: Point,
    pub orient: Orientation,
}

impl WriteDef for KnownPlacement {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "{} ", self.kind)?;
        self.pt.write(out)?;
        write!(out, " {}", self.orient)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
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
                pt.write(out)?;
                writeln!(out, " {orient}")?;
            }
            PlacementStatus::Cover { pt, orient } => {
                write!(out, "+ COVER ")?;
                pt.write(out)?;
                writeln!(out, " {orient}")?;
            }
            PlacementStatus::Placed { pt, orient } => {
                write!(out, "+ PLACED ")?;
                pt.write(out)?;
                writeln!(out, " {orient}")?;
            }
            PlacementStatus::Unplaced => {
                writeln!(out, "+ UNPLACED")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Property {
    pub name: String,
    pub val: String,
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
    /// Produces the orientation corresponding to the given [`NamedOrientation`].
    pub fn from_named(value: NamedOrientation) -> Option<Self> {
        match value {
            NamedOrientation::R0 => Some(Self::N),
            NamedOrientation::ReflectVert => Some(Self::Fs),
            NamedOrientation::ReflectHoriz => Some(Self::Fn),
            NamedOrientation::R90 | NamedOrientation::R270Cw => Some(Self::W),
            NamedOrientation::R180 | NamedOrientation::R180Cw => Some(Self::S),
            NamedOrientation::R270 | NamedOrientation::R90Cw => Some(Self::E),
            NamedOrientation::FlipYx => Some(Self::Fw),
            NamedOrientation::FlipMinusYx => Some(Self::Fe),
            _ => None,
        }
    }
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

#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ComponentPlaceHalo {
    pub soft: bool,
    pub left: i64,
    pub bottom: i64,
    pub right: i64,
    pub top: i64,
}

#[derive(Debug, Clone)]
pub struct ComponentRouteHalo {
    pub halo_dist: i64,
    pub min_layer: Ident,
    pub max_layer: Ident,
}

#[derive(Debug, Clone)]
pub struct DieArea {
    /// Must have at least 2 points.
    pub pts: Vec<Point>,
}

impl WriteDef for Point {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "( {} {} )", self.x, self.y)
    }
}

impl WriteDef for DieArea {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        assert!(self.pts.len() >= 2);
        write!(out, "DIEAREA")?;
        for pt in self.pts.iter() {
            write!(out, " ")?;
            pt.write(out)?;
        }
        writeln!(out, " ;")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Groups {
    pub groups: Vec<Group>,
}

#[derive(Debug, Clone)]
pub struct Group {
    pub name: Ident,
    pub comp_name_patterns: Vec<Pattern>,
    pub region: Option<Ident>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
pub struct Nets {
    pub nets: Vec<Net>,
}

impl WriteDef for Nets {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        if self.nets.is_empty() {
            return Ok(());
        }
        writeln!(out, "NETS {} ;", self.nets.len())?;
        for n in self.nets.iter() {
            write!(out, "  - ")?;
            n.ident.write(out)?;
            for x in n.shield_nets.iter() {
                writeln!(out, "    + SHIELDNET {}", x)?;
            }
            for x in n.virtual_pins.iter() {
                write!(out, "    ")?;
                x.write(out)?;
                writeln!(out)?;
            }
            for s in n.subnets.iter() {
                write!(out, "    ")?;
                s.write(out)?;
                writeln!(out)?;
            }
            if let Some(x) = n.xtalk {
                writeln!(out, "    + XTALK {x}")?;
            }
            if let Some(x) = &n.nondefault_rule {
                writeln!(out, "    + NONDEFAULTRULE {x}")?;
            }
            for w in n.wiring.iter() {
                w.write(out)?;
            }
            if let Some(s) = &n.source {
                writeln!(out, "    + SOURCE {}", s)?;
            }
            if n.fixed_bump {
                writeln!(out, "    + FIXEDBUMP")?;
            }
            if let Some(x) = &n.frequency {
                writeln!(out, "    + FREQUENCY {x}")?;
            }
            if let Some(o) = &n.original {
                writeln!(out, "    + ORIGINAL {}", o)?;
            }
            if let Some(x) = &n.net_type {
                writeln!(out, "    + USE {}", x)?;
            }
            if let Some(x) = &n.pattern {
                writeln!(out, "    + PATTERN {}", x)?;
            }
            if let Some(x) = n.est_cap {
                writeln!(out, "    + ESTCAP {}", x)?;
            }
            if let Some(x) = &n.weight {
                writeln!(out, "    + WEIGHT {}", x)?;
            }
            if !n.properties.is_empty() {
                write!(out, "    + PROPERTY")?;
                for p in n.properties.iter() {
                    write!(out, "\"{}\" \"{}\"", p.name, p.val)?;
                }
            }
            writeln!(out, "    ;")?;
        }
        writeln!(out, "END NETS")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Net {
    pub ident: NetIdent,
    pub shield_nets: Vec<Ident>,
    pub virtual_pins: Vec<VirtualPin>,
    pub subnets: Vec<Subnet>,
    pub xtalk: Option<u8>,
    pub nondefault_rule: Option<Ident>,
    pub wiring: Vec<RegularWiring>,
    pub source: Option<NetSource>,
    pub fixed_bump: bool,
    pub frequency: Option<f64>,
    pub original: Option<Ident>,
    pub net_type: Option<NetType>,
    pub pattern: Option<NetPattern>,
    pub est_cap: Option<f64>,
    pub weight: Option<u32>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
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

impl NetType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NetType::Analog => "ANALOG",
            NetType::Clock => "CLOCK",
            NetType::Ground => "GROUND",
            NetType::Power => "POWER",
            NetType::Reset => "RESET",
            NetType::Scan => "SCAN",
            NetType::Signal => "SIGNAL",
            NetType::Tieoff => "TIEOFF",
        }
    }
}

impl Display for NetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl WriteDef for NetType {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub enum NetPattern {
    Balanced,
    Steiner,
    Trunk,
    WiredLogic,
}

impl NetPattern {
    pub fn as_str(&self) -> &'static str {
        match self {
            NetPattern::Balanced => "BALANCED",
            NetPattern::Steiner => "STEINER",
            NetPattern::Trunk => "TRUNK",
            NetPattern::WiredLogic => "WIREDLOGIC",
        }
    }
}

impl Display for NetPattern {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl WriteDef for NetPattern {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub enum NetIdent {
    Named(NamedNetIdent),
    MustJoin { component: Ident, pin: Ident },
}

impl WriteDef for NetIdent {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            NetIdent::Named(n) => n.write(out)?,
            NetIdent::MustJoin { component, pin } => {
                write!(out, "MUSTJOIN ( {} {} )", component, pin)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NetPin {
    pub kind: NetPinKind,
    pub synthesized: bool,
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

#[derive(Debug, Clone)]
pub enum NetPinKind {
    ComponentPin { comp_name: Ident, pin_name: Ident },
    IoPin { name: Ident },
}

impl WriteDef for NetPinKind {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            NetPinKind::ComponentPin {
                comp_name,
                pin_name,
            } => {
                write!(out, "{comp_name} {pin_name}")
            }
            NetPinKind::IoPin { name } => {
                write!(out, "PIN {name}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct VirtualPin {
    pub name: Ident,
    pub layer: Option<Ident>,
    pub p0: Point,
    pub p1: Point,
    pub placement: Option<KnownPlacement>,
}

impl WriteDef for VirtualPin {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "+ VPIN {}", self.name)?;
        if let Some(x) = &self.layer {
            write!(out, " LAYER {}", x)?;
        }
        write!(out, " ")?;
        self.p0.write(out)?;
        write!(out, " ")?;
        self.p1.write(out)?;
        if let Some(x) = &self.placement {
            write!(out, " ")?;
            x.write(out)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct Subnet {
    pub name: Ident,
    pub pins: Vec<SubnetPin>,
}

impl WriteDef for Subnet {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        writeln!(out, "+ SUBNET {}", self.name)?;
        for pin in self.pins.iter() {
            pin.write(out)?;
            writeln!(out)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum SubnetPin {
    Component { comp_name: Ident, pin_name: Ident },
    IoPin { name: Ident },
    VirtualPin { name: Ident },
}

impl WriteDef for SubnetPin {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            SubnetPin::Component {
                comp_name,
                pin_name,
            } => {
                write!(out, "( {comp_name} {pin_name} )")?;
            }
            SubnetPin::IoPin { name } => {
                write!(out, "PIN {name}")?;
            }
            SubnetPin::VirtualPin { name } => {
                write!(out, "VPIN {name}")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RoutingXy {
    pub x: i64,
    pub y: i64,
    pub ext: Option<i64>,
}

impl WriteDef for RoutingXy {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        if let Some(ext) = self.ext {
            write!(out, "( {} {} {} )", self.x, self.y, ext)
        } else {
            write!(out, "( {} {} )", self.x, self.y)
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegularRoutingPoints {
    pub start: RoutingXy,
    pub points: Vec<RegularRoutingPoint>,
}

impl WriteDef for RegularRoutingPoints {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        self.start.write(out)?;
        writeln!(out)?;

        for pt in self.points.iter() {
            pt.write(out)?;
            writeln!(out)?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SpecialRoutingPoints {
    pub start: RoutingXy,
    pub points: Vec<SpecialRoutingPoint>,
}

impl WriteDef for SpecialRoutingPoints {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        self.start.write(out)?;
        writeln!(out)?;
        for pt in self.points.iter() {
            pt.write(out)?;
            writeln!(out)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum RegularRoutingPoint {
    Point {
        mask: Option<MaskNum>,
        pt: RoutingXy,
    },
    Via {
        mask: Option<MaskNum>,
        via_name: Ident,
        orient: Option<Orientation>,
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

impl WriteDef for RegularRoutingPoint {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            RegularRoutingPoint::Point { mask, pt } => {
                if let Some(mask) = mask {
                    write!(out, "MASK {} ", mask.0)?;
                }
                pt.write(out)?;
            }
            RegularRoutingPoint::Via {
                mask,
                via_name,
                orient,
            } => {
                if let Some(mask) = mask {
                    write!(out, "MASK {} ", mask.0)?;
                }
                write!(out, "{via_name}")?;
                if let Some(o) = orient {
                    write!(out, " {o}")?;
                }
            }
            RegularRoutingPoint::Rect {
                mask,
                dx1,
                dy1,
                dx2,
                dy2,
            } => {
                if let Some(mask) = mask {
                    write!(out, "MASK {} ", mask.0)?;
                }

                write!(out, "RECT ( {dx1} {dy1}  {dx2} {dy2} )")?;
            }
            RegularRoutingPoint::Virtual { x, y } => {
                write!(out, "VIRTUAL ( {x} {y} )")?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ViaArray {
    pub nx: u32,
    pub ny: u32,
    pub step_x: i64,
    pub step_y: i64,
}

#[derive(Debug, Clone)]
pub enum SpecialRoutingPoint {
    Point {
        mask: Option<MaskNum>,
        pt: RoutingXy,
    },
    Via {
        mask: Option<MaskNum>,
        via_name: Ident,
        orient: Option<Orientation>,
        array: Option<ViaArray>,
    },
}

impl WriteDef for SpecialRoutingPoint {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            SpecialRoutingPoint::Point { mask, pt } => {
                if let Some(mask) = mask {
                    write!(out, "MASK {} ", mask.0)?;
                }
                pt.write(out)?;
            }
            SpecialRoutingPoint::Via {
                mask,
                via_name,
                orient,
                array,
            } => {
                if let Some(mask) = mask {
                    write!(out, "MASK {} ", mask.0)?;
                }
                write!(out, "{}", via_name)?;
                if let Some(orient) = orient {
                    write!(out, " {}", orient)?;
                }
                if let Some(array) = array {
                    write!(
                        out,
                        " DO {} BY {} STEP {} {}",
                        array.nx, array.ny, array.step_x, array.step_y
                    )?;
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum RoutingStatus {
    Cover,
    Fixed,
    Routed,
    NoShield,
}

impl RoutingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            RoutingStatus::Cover => "COVER",
            RoutingStatus::Fixed => "FIXED",
            RoutingStatus::Routed => "ROUTED",
            RoutingStatus::NoShield => "NOSHIELD",
        }
    }
}

impl Display for RoutingStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl WriteDef for RoutingStatus {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub enum Taper {
    Default,
    Rule(Ident),
}

impl WriteDef for Taper {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            Taper::Default => {
                write!(out, "TAPER")
            }
            Taper::Rule(r) => {
                write!(out, "TAPERRULE {r}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RegularWiring {
    pub status: RoutingStatus,
    pub entries: NonEmptyVec<RegularWiringEntry>,
}

impl WriteDef for RegularWiring {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        writeln!(out, "+ {}", self.status)?;

        let first = self
            .entries
            .first()
            .expect("regular wiring entries must not be empty");
        first.write(out)?;
        writeln!(out)?;

        for entry in self.entries.iter().skip(1) {
            write!(out, "NEW ")?;
            entry.write(out)?;
            writeln!(out)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct RegularWiringEntry {
    pub layer: Ident,
    pub taper: Option<Taper>,
    pub style: Option<u32>,
    pub points: RegularRoutingPoints,
}

impl WriteDef for RegularWiringEntry {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "{}", self.layer)?;
        if let Some(t) = &self.taper {
            write!(out, " ")?;
            t.write(out)?;
        }
        if let Some(s) = self.style {
            write!(out, " STYLE {s}")?;
        }
        writeln!(out)?;
        self.points.write(out)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SpecialNets {
    pub nets: Vec<SpecialNet>,
}

impl WriteDef for SpecialNets {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        if self.nets.is_empty() {
            return Ok(());
        }
        writeln!(out, "SPECIALNETS {} ;", self.nets.len())?;
        for n in self.nets.iter() {
            writeln!(out, "  - {}", n.name.name)?;
            for pin in n.name.pins.iter() {
                pin.write(out)?;
            }
            if let Some(voltage) = n.voltage {
                writeln!(out, "    + VOLTAGE {voltage}")?;
            }
            for w in n.wiring.iter() {
                w.write(out)?;
            }
            if let Some(s) = &n.source {
                writeln!(out, "    + SOURCE {}", s)?;
            }
            if n.fixed_bump {
                writeln!(out, "    + FIXEDBUMP")?;
            }
            if let Some(o) = &n.original {
                writeln!(out, "    + ORIGINAL {}", o)?;
            }
            if let Some(x) = &n.net_type {
                writeln!(out, "    + USE {}", x)?;
            }
            if let Some(x) = &n.pattern {
                writeln!(out, "    + PATTERN {}", x)?;
            }
            if let Some(x) = n.est_cap {
                writeln!(out, "    + ESTCAP {}", x)?;
            }
            if let Some(x) = &n.weight {
                writeln!(out, "    + WEIGHT {}", x)?;
            }
            if !n.properties.is_empty() {
                write!(out, "    + PROPERTY")?;
                for p in n.properties.iter() {
                    write!(out, "\"{}\" \"{}\"", p.name, p.val)?;
                }
            }
            writeln!(out, "    ;")?;
        }
        writeln!(out, "END SPECIALNETS")?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct NamedNetIdent {
    pub name: Ident,
    pub pins: Vec<NetPin>,
}

impl WriteDef for NamedNetIdent {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        writeln!(out, "{}", self.name)?;
        for pin in self.pins.iter() {
            pin.write(out)?;
            writeln!(out)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SpecialNet {
    pub name: NamedNetIdent,
    /// Voltage in mV.
    ///
    /// Example: + VOLTAGE 3300 means 3.3V.
    pub voltage: Option<i64>,
    pub wiring: Vec<SpecialWiring>,
    pub source: Option<Source>,
    pub fixed_bump: bool,
    pub original: Option<Ident>,
    pub net_type: Option<NetType>,
    pub pattern: Option<NetPattern>,
    pub est_cap: Option<f64>,
    pub weight: Option<u32>,
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone)]
pub enum SpecialWiring {
    Geometry(GeometrySpecialWiring),
    Path(PathSpecialWiring),
}

impl WriteDef for SpecialWiring {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            SpecialWiring::Geometry(x) => x.write(out),
            SpecialWiring::Path(x) => x.write(out),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GeometrySpecialWiring {
    pub status: Option<SpecialRoutingStatus>,
    pub shape: Option<ShapeType>,
    pub mask: Option<MaskNum>,
    pub entry: GeometrySpecialWiringEntry,
}

impl WriteDef for GeometrySpecialWiring {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        if let Some(s) = &self.status {
            s.write(out)?;
            writeln!(out)?;
        }
        if let Some(s) = &self.shape {
            s.write(out)?;
            writeln!(out)?;
        }
        if let Some(m) = &self.mask {
            writeln!(out, "+ MASK {}", m.0)?;
        }
        self.entry.write(out)?;
        writeln!(out)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct LayerRect {
    pub layer: Ident,
    pub mask: Option<MaskNum>,
    pub rect: Rect,
}

#[derive(Debug, Clone)]
pub struct LayerPolygon {
    pub layer: Ident,
    pub mask: Option<MaskNum>,
    pub polygon: Polygon,
}

#[derive(Debug, Clone)]
pub struct PlacedVia {
    pub via_name: Ident,
    pub orient: Option<Orientation>,
    pub points: Vec<Point>,
}

impl WriteDef for PlacedVia {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "+ VIA {}", self.via_name)?;
        if let Some(orient) = self.orient {
            write!(out, " {}", orient)?;
        }
        for pt in self.points.iter() {
            write!(out, " ")?;
            pt.write(out)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum GeometrySpecialWiringEntry {
    Rect(LayerRect),
    Polygon(LayerPolygon),
    Via(PlacedVia),
}

impl WriteDef for GeometrySpecialWiringEntry {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            GeometrySpecialWiringEntry::Rect(r) => r.write(out),
            GeometrySpecialWiringEntry::Polygon(p) => p.write(out),
            GeometrySpecialWiringEntry::Via(v) => v.write(out),
        }
    }
}

#[derive(Debug, Clone)]
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

impl ShapeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ShapeType::Ring => "RING",
            ShapeType::PadRing => "PADRING",
            ShapeType::BlockRing => "BLOCKRING",
            ShapeType::Stripe => "STRIPE",
            ShapeType::FollowPin => "FOLLOWPIN",
            ShapeType::IoWire => "IOWIRE",
            ShapeType::CoreWire => "COREWIRE",
            ShapeType::BlockWire => "BLOCKWIRE",
            ShapeType::BlockageWire => "BLOCKAGEWIRE",
            ShapeType::FillWire => "FILLWIRE",
            ShapeType::FillWireOpc => "FILLWIREOPC",
            ShapeType::DrcFill => "DRCFILL",
        }
    }
}

impl Display for ShapeType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl WriteDef for ShapeType {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct PathSpecialWiring {
    pub status: SpecialRoutingStatus,
    pub entries: NonEmptyVec<PathSpecialWiringEntry>,
}

impl WriteDef for PathSpecialWiring {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        self.status.write(out)?;
        writeln!(out)?;
        let first = self
            .entries
            .first()
            .expect("path special wiring entries must not be empty");
        first.write(out)?;
        writeln!(out)?;

        for entry in self.entries.iter().skip(1) {
            write!(out, "NEW ")?;
            entry.write(out)?;
            writeln!(out)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PathSpecialWiringEntry {
    pub layer: Ident,
    pub width: i64,
    pub shape: Option<ShapeType>,
    pub style: Option<u32>,
    pub points: SpecialRoutingPoints,
}

impl WriteDef for PathSpecialWiringEntry {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        writeln!(out, "{} {}", self.layer, self.width)?;
        if let Some(shape) = &self.shape {
            shape.write(out)?;
            writeln!(out)?;
        }
        if let Some(style) = self.style {
            writeln!(out, "+ STYLE {style}")?;
        }
        self.points.write(out)?;
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum SpecialRoutingStatus {
    Cover,
    Fixed,
    Routed,
    Shield { net: Ident },
}

impl WriteDef for SpecialRoutingStatus {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        match self {
            SpecialRoutingStatus::Cover => write!(out, "+ COVER"),
            SpecialRoutingStatus::Fixed => write!(out, "+ FIXED"),
            SpecialRoutingStatus::Routed => write!(out, "+ ROUTED"),
            SpecialRoutingStatus::Shield { net } => write!(out, "+ SHIELD {net}"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Vias {
    pub vias: Vec<Via>,
}

impl WriteDef for LayerRect {
    fn write<W: Write>(&self, out: &mut W) -> std::io::Result<()> {
        write!(out, "+ RECT {}", self.layer)?;
        if let Some(mask) = &self.mask {
            write!(out, "+ MASK {}", mask.0)?;
        }
        write!(out, " ")?;
        self.rect.lower_left().write(out)?;
        write!(out, " ")?;
        self.rect.upper_right().write(out)?;
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
            pt.write(out)?;
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

#[derive(Debug, Clone)]
pub struct Via {
    pub name: Ident,
    pub definition: ViaDef,
}

#[derive(Debug, Clone)]
pub enum ViaDef {
    Fixed(FixedVia),
    ViaRule(Box<ViaRuleVia>),
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct ViaOffset {
    pub bot_ofs_x: i64,
    pub bot_ofs_y: i64,
    pub top_ofs_x: i64,
    pub top_ofs_y: i64,
}

#[derive(Debug, Clone)]
pub struct FixedVia {
    pub geometry: Vec<ViaGeometry>,
}

#[derive(Debug, Clone)]
pub enum ViaGeometry {
    Rect(LayerRect),
    Polygon(LayerPolygon),
}

#[derive(Debug, Clone)]
pub struct Units {
    pub dbu_per_micron: i64,
}

#[derive(Debug, Clone)]
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
            special_nets: Some(SpecialNets {
                nets: vec![SpecialNet {
                    name: NamedNetIdent {
                        name: "vss".to_string(),
                        pins: vec![],
                    },
                    voltage: None,
                    wiring: vec![SpecialWiring::Geometry(GeometrySpecialWiring {
                        status: Some(SpecialRoutingStatus::Fixed),
                        shape: None,
                        mask: None,
                        entry: GeometrySpecialWiringEntry::Rect(LayerRect {
                            layer: "m3".to_string(),
                            mask: None,
                            rect: Rect::new(Point::new(0, 0), Point::new(600, 100)),
                        }),
                    })],
                    source: None,
                    fixed_bump: false,
                    original: None,
                    net_type: Some(NetType::Ground),
                    pattern: None,
                    est_cap: None,
                    weight: None,
                    properties: vec![],
                }],
            }),
            nets: None,
        };

        let mut buf: Vec<u8> = Vec::new();
        def.write(&mut buf).expect("failed to write def");
        let s = String::from_utf8(buf).expect("failed to convert from utf8");
        println!("{s}");
    }
}
