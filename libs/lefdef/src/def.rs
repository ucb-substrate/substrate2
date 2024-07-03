use geometry::point::Point;
use geometry::polygon::Polygon;
use geometry::rect::Rect;

pub type Ident = String;
pub type Pattern = String;
pub type NonEmptyVec<T> = Vec<T>;

pub struct Def {
    blockages: Option<Blockages>,
    design: Ident,
}

pub struct Blockages {
    blockages: Vec<Blockage>,
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

pub enum LayerBlockageSpacing {
    Spacing(i64),
    DesignRuleWidth(i64),
}

pub struct MaskNum(u32);

pub enum Geometry {
    Rect(Rect),
    Polygon(Polygon),
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

pub struct BusBitChars {
    open: char,
    close: char,
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
    source: Option<ComponentSource>,
    placement_status: Option<PlacementStatus>,
    mask_shift: Option<u32>,
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
    Fixed {
        pt: Point,
        orient: Orientation,
    },
    Cover {
        pt: Point,
        orient: Orientation,
    },
    Placed {
        pt: Point,
        orient: Orientation,
    },
    Unplaced,
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

pub enum ComponentSource {
    Netlist,
    Dist,
    User,
    Timing,
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
}

pub enum NetIdent {
    Named {
        name: Ident,
        pins: Vec<NetPin>,
    },
    MustJoin {
        component: Ident,
        pin: Ident,
    }
}

pub struct NetPin {
    kind: NetPinKind,
    synthesized: bool,
}
pub enum NetPinKind {
    ComponentPin {
        comp_name: Ident,
        pin_name: Ident,
    },
    IoPin {
        name: Ident,
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
    Component {
        comp_name: Ident,
        pin_name: Ident,
    },
    IoPin {
        name: Ident,
    },
    VirtualPin {
        name: Ident,
    }
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
        mask: Option<u32>,
        pt: RoutingXy,
    },
    Via {
        mask: Option<u32>,
        via_name: Ident,
        orient: Orientation,
    },
    Rect {
        mask: Option<u32>,
        dx1: i64,
        dy1: i64,
        dx2: i64,
        dy2: i64,
    },
    Virtual {
        x: i64,
        y: i64,
    }
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