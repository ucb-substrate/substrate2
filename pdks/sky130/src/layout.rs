//! Layout for sky130.

use std::collections::HashMap;

use arcstr::ArcStr;
use gds::GdsUnits;
use gdsconv::GdsLayer;
use geometry::prelude::{Contains, Transformation};
use geometry::{bbox::Bbox, dir::Dir, rect::Rect, span::Span};
use layir::{Cell, Direction, Element, Instance, LibraryBuilder, Port, Shape, Text};
use serde::{Deserialize, Serialize};
use substrate::types::codegen::PortGeometryBundle;
use substrate::{
    block::Block,
    layout::{
        tracks::{Tracks, UniformTracks},
        CellBuilder, Layout,
    },
    types::{layout::PortGeometry, InOut, Io, Signal},
};

use crate::{layers::Sky130Layer, Sky130};

/// The units used for importing/exporting GDS libraries.
pub const GDS_UNITS: GdsUnits = GdsUnits::new(1., 1e-9);

/// Convert a sky130 layout library to a GDS layout library.
// TODO: cell IDs are not preserved
pub fn to_gds(lib: &layir::Library<Sky130Layer>) -> (layir::Library<GdsLayer>, GdsUnits) {
    let mut olib = LibraryBuilder::<GdsLayer>::new();
    let cells = lib.topological_order();
    for cell in cells {
        let cell = lib.cell(cell);
        let mut ocell = Cell::new(cell.name());
        for elt in cell.elements() {
            ocell.add_element(elt.map_layer(Sky130Layer::gds_layer));
        }
        for (_, inst) in cell.instances() {
            let name = lib.cell(inst.child()).name();
            let child_id = olib.cell_id_named(name);
            ocell.add_instance(Instance::with_transformation(
                child_id,
                inst.name(),
                inst.transformation(),
            ));
        }
        for (name, oport) in cell.ports() {
            let mut port = oport.map_layer(|layer| Sky130Layer::gds_pin_layer(layer).unwrap());
            oport
                .elements()
                .filter_map(|e| match e {
                    Element::Text(_) => None,
                    Element::Shape(s) => Some(s),
                })
                .for_each(|s| {
                    let center = s.bbox_rect().center();
                    // places 2 labels: one on the pin layer, and one on the label layer
                    for layer in [
                        Sky130Layer::gds_pin_layer(s.layer()).unwrap(),
                        Sky130Layer::gds_label_layer(s.layer()).unwrap(),
                    ] {
                        port.add_element(Element::Text(Text::with_transformation(
                            layer,
                            name.clone(),
                            Transformation::translate(center.x, center.y),
                        )));
                    }
                });
            ocell.add_port(name, port);
        }
        olib.add_cell(ocell);
    }
    (olib.build().unwrap(), GDS_UNITS)
}

/// Convert a GDS layout library to a sky130 layout library.
pub fn from_gds(lib: &layir::Library<GdsLayer>) -> layir::Library<Sky130Layer> {
    let mut olib = LibraryBuilder::<Sky130Layer>::new();
    let cells = lib.topological_order();
    for cell in cells {
        let cell = lib.cell(cell);
        let mut ocell = Cell::new(cell.name());
        for elt in cell.elements() {
            ocell.add_element(elt.map_layer(|layer| Sky130Layer::from_gds_layer(layer).unwrap()));
        }
        for (_, inst) in cell.instances() {
            let name = lib.cell(inst.child()).name();
            let child_id = olib.cell_id_named(name);
            ocell.add_instance(Instance::with_transformation(
                child_id,
                inst.name(),
                inst.transformation(),
            ));
        }
        for (name, oport) in cell.ports() {
            let port = oport.map_layer(|layer| Sky130Layer::from_gds_layer(layer).unwrap());
            ocell.add_port(name, port);
        }
        let mut pin_correspondences: HashMap<
            Sky130Layer,
            (Vec<Shape<Sky130Layer>>, Vec<Text<Sky130Layer>>),
        > = HashMap::new();
        for elt in cell.elements() {
            match elt {
                Element::Shape(s) => {
                    if let Some(layer) = Sky130Layer::from_gds_pin_layer(s.layer()) {
                        let entry = pin_correspondences.entry(layer).or_default();
                        entry.0.push(s.with_layer(layer));
                    }
                }
                Element::Text(t) => {
                    if let Some(layer) = Sky130Layer::from_gds_pin_layer(t.layer())
                        .or_else(|| Sky130Layer::from_gds_label_layer(t.layer()))
                    {
                        let entry = pin_correspondences.entry(layer).or_default();
                        entry.1.push(t.with_layer(layer));
                    }
                }
            }
        }
        for (_, (shapes, texts)) in pin_correspondences {
            for shape in shapes {
                let mut name = None;
                let mut pin_texts = Vec::new();
                for text in texts.iter() {
                    if shape
                        .shape()
                        .contains(&text.transformation().offset_point())
                        .intersects()
                    {
                        // Identify pin shapes with multiple labels.
                        if let Some(ref name) = name {
                            if name != text.text() {
                                panic!("pin with multiple names");
                            }
                        }
                        name = Some(text.text().clone());
                        pin_texts.push(text.clone());
                    }
                }

                if let Some(name) = name {
                    if ocell.try_port(&name).is_none() {
                        ocell.add_port(&name, Port::new(Direction::InOut));
                    }
                    let port = ocell.port_mut(&name);
                    port.add_element(shape);
                    for text in pin_texts {
                        // If a text overlaps multiple pin shapes, the imported library will have
                        // multiple texts: one for each pin shape. We may want to change this
                        // behavior in the future.
                        port.add_element(text);
                    }
                } else {
                    panic!("pin shape with no label");
                }
            }
        }
        olib.add_cell(ocell);
    }

    olib.build().unwrap()
}

struct TapTileData {
    li: Rect,
    tap: Rect,
    bbox: Rect,
}

/// A tile containing taps.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct TapTile {
    /// x dimension, in number of li1 tracks
    xtracks: i64,
    /// y dimension, in number of m1 tracks
    ytracks: i64,
}

impl TapTile {
    /// Create a new tap tile with the given dimensions.
    pub fn new(xtracks: i64, ytracks: i64) -> Self {
        Self { xtracks, ytracks }
    }

    fn name(&self) -> ArcStr {
        arcstr::format!("tap_tile_x{}_y{}", self.xtracks, self.ytracks)
    }
}

impl TapTile {
    fn layout(&self, cell: &mut CellBuilder<Sky130>) -> substrate::error::Result<TapTileData> {
        let m0tracks = UniformTracks::new(170, 260);
        let m1tracks = UniformTracks::new(400, 140);

        let li_hspan = m0tracks.track(0).union(m0tracks.track(self.xtracks - 1));
        let li_vspan = Span::new(
            m1tracks.track(0).center(),
            m1tracks.track(self.ytracks - 1).center(),
        )
        .expand_all(85);
        let inner = Rect::from_spans(li_hspan, li_vspan);
        let li = inner.expand_dir(Dir::Horiz, 80);
        cell.draw(Shape::new(Sky130Layer::Li1, li))?;

        for x in 0..self.xtracks {
            for y in 0..self.ytracks {
                let cut = Rect::from_spans(
                    m0tracks.track(x),
                    Span::from_center_span(m1tracks.track(y).center(), 170),
                );
                cell.draw(Shape::new(Sky130Layer::Licon1, cut))?;
            }
        }

        let tap = inner.expand_dir(Dir::Vert, 65).expand_dir(Dir::Horiz, 120);
        cell.draw(Shape::new(Sky130Layer::Tap, tap))?;

        let bbox = cell.bbox_rect();
        cell.draw(Shape::new(Sky130Layer::Outline, bbox))?;

        Ok(TapTileData { li, tap, bbox })
    }
}

/// A tile containing an N+ tap for biasing an N-well.
/// These can be used to connect to the body terminals of PMOS devices.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct NtapTile {
    tile: TapTile,
}

impl NtapTile {
    /// Create a new ntap tile with the given dimensions.
    pub fn new(xtracks: i64, ytracks: i64) -> Self {
        Self {
            tile: TapTile::new(xtracks, ytracks),
        }
    }
}

/// The IO of an [`NtapTile`].
#[derive(Clone, Default, Debug, Io)]
pub struct NtapIo {
    /// The n-well net.
    pub vpb: InOut<Signal>,
}

impl Block for NtapTile {
    type Io = NtapIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("n{}", self.tile.name())
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Layout for NtapTile {
    type Schema = Sky130;
    type Bundle = NtapIoView<PortGeometryBundle<Sky130>>;
    type Data = ();
    fn layout(
        &self,
        cell: &mut substrate::layout::CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        let data = self.tile.layout(cell)?;
        let vpb = Shape::new(Sky130Layer::Li1, data.li);
        cell.draw(vpb.clone())?;

        let nsdm = data.tap.expand_all(130);
        let nsdm = nsdm.with_hspan(data.bbox.hspan().union(nsdm.hspan()));
        cell.draw(Shape::new(Sky130Layer::Nsdm, nsdm))?;

        let nwell = data.tap.expand_all(180);
        let nwell = nwell
            .with_hspan(data.bbox.hspan().union(nwell.hspan()))
            .with_vspan(data.bbox.vspan().union(nwell.vspan()));
        cell.draw(Shape::new(Sky130Layer::Nwell, nwell))?;

        Ok((
            NtapIoView {
                vpb: PortGeometry::new(vpb),
            },
            (),
        ))
    }
}

/// A tile containing a P+ tap for biasing an P-well or P-substrate.
/// These can be used to connect to the body terminals of NMOS devices.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub struct PtapTile {
    tile: TapTile,
}

impl PtapTile {
    /// Create a new ntap tile with the given dimensions.
    pub fn new(xtracks: i64, ytracks: i64) -> Self {
        Self {
            tile: TapTile::new(xtracks, ytracks),
        }
    }
}

/// The IO of a [`PtapTile`].
#[derive(Io, Clone, Default, Debug)]
pub struct PtapIo {
    /// The p-well net.
    pub vnb: InOut<Signal>,
}

impl Block for PtapTile {
    type Io = PtapIo;

    fn name(&self) -> ArcStr {
        arcstr::format!("p{}", self.tile.name())
    }

    fn io(&self) -> Self::Io {
        Default::default()
    }
}

impl Layout for PtapTile {
    type Schema = Sky130;
    type Bundle = PtapIoView<PortGeometryBundle<Sky130>>;
    type Data = ();
    fn layout(
        &self,
        cell: &mut substrate::layout::CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        let data = self.tile.layout(cell)?;
        let vnb = Shape::new(Sky130Layer::Li1, data.li);
        cell.draw(vnb.clone())?;

        let psdm = data.tap.expand_all(130);
        let psdm = psdm.with_hspan(data.bbox.hspan().union(psdm.hspan()));
        cell.draw(Shape::new(Sky130Layer::Psdm, psdm))?;
        Ok((
            PtapIoView {
                vnb: PortGeometry::new(vnb),
            },
            (),
        ))
    }
}
