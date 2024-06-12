//! Utilities for GDS conversion.
//!
//! Converts between Substrate's layout data-model and [`gds`] structures.

use std::collections::HashSet;
use std::{collections::HashMap, sync::Arc};

use arcstr::ArcStr;
use gds::{GdsUnits, HasLayer};
use geometry::prelude::Polygon;
use geometry::span::Span;
use geometry::transform::Transformation;
use geometry::{
    prelude::{Corner, Orientation, Point},
    rect::Rect,
};
use indexmap::IndexMap;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use slotmap::{new_key_type, SlotMap};
use tracing::{span, Level};
use uniquify::Names;

use crate::io::layout::{BundleBuilder, HardwareType, PortGeometry};
use crate::layout::error::GdsExportError;
use crate::pdk::layers::LayerInfo;
use crate::{
    io::{layout::IoShape, NameBuf},
    pdk::layers::{GdsLayerSpec, HasPin, LayerContext, LayerId},
};

use super::error::{GdsImportError, GdsImportResult};
use super::LayoutContext;
use super::{
    element::{CellId, Element, RawCell, RawInstance, Shape, Text},
    error::GdsExportResult,
};

new_key_type! {
    /// A key used for identifying elements when importing a GDSII file.
    pub struct ElementKey;
}

/// An exporter for GDS files.
///
/// Takes a [`RawCell`] and converts it to a [`gds::GdsLibrary`].
pub struct GdsExporter<'a> {
    cells: Vec<Arc<RawCell>>,
    layers: &'a LayerContext,
    cell_db: Names<CellId>,
    gds: gds::GdsLibrary,
}

impl<'a> GdsExporter<'a> {
    /// Creates a new GDS exporter.
    ///
    /// Requires the cell to be exported and a [`LayerContext`] for mapping Substrate layers to GDS
    /// layers.
    pub fn new(cells: Vec<Arc<RawCell>>, layers: &'a LayerContext) -> Self {
        Self {
            cells,
            layers,
            cell_db: Default::default(),
            gds: gds::GdsLibrary::new("TOP"),
        }
    }

    /// Creates a new GDS exporter with the given units.
    ///
    /// Requires the cell to be exported and a [`LayerContext`] for mapping Substrate layers to GDS
    /// layers.
    pub fn with_units(cells: Vec<Arc<RawCell>>, layers: &'a LayerContext, units: GdsUnits) -> Self {
        Self {
            cells,
            layers,
            cell_db: Default::default(),
            gds: gds::GdsLibrary::with_units("TOP", units),
        }
    }

    /// Exports the contents of `self` as a [`gds::GdsLibrary`].
    pub fn export(mut self) -> GdsExportResult<gds::GdsLibrary> {
        for cell in self.cells.clone() {
            cell.clone().export(&mut self)?;
        }
        Ok(self.gds)
    }

    fn get_name(&self, cell: &RawCell) -> Option<ArcStr> {
        self.cell_db.name(&cell.id)
    }

    fn assign_name(&mut self, cell: &RawCell) -> ArcStr {
        self.cell_db.assign_name(cell.id, &cell.name)
    }

    fn get_layer(&self, id: LayerId) -> Option<GdsLayerSpec> {
        self.layers.get_gds_layer_from_id(id)
    }
}

#[allow(clippy::from_over_into)]
impl Into<gds::GdsLayerSpec> for GdsLayerSpec {
    fn into(self) -> gds::GdsLayerSpec {
        gds::GdsLayerSpec {
            layer: self.0 as i16,
            xtype: self.1 as i16,
        }
    }
}

/// An object that can be exported as a GDS element.
trait ExportGds {
    /// The GDS type that this object corresponds to.
    type Output;

    /// Exports `self` as its GDS counterpart, accessing and mutating state in `exporter` as needed.
    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output>;
}

impl ExportGds for RawCell {
    type Output = gds::GdsStruct;

    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let name = exporter.assign_name(self);
        let name_str: &str = self.name.as_ref();

        let span = span!(Level::INFO, "cell", name = name_str);
        let _guard = span.enter();

        let mut cell = gds::GdsStruct::new(name);

        cell.elems.extend(self.port_map().export(exporter)?);

        for element in self.elements.iter() {
            if let Some(elem) = element.export(exporter)? {
                cell.elems.push(elem);
            }
        }

        exporter.gds.structs.push(cell.clone());

        Ok(cell)
    }
}

impl ExportGds for HashMap<NameBuf, PortGeometry> {
    type Output = Vec<gds::GdsElement>;

    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let mut elements = Vec::new();
        for (name_buf, geometry) in self {
            elements.extend((name_buf, &geometry.primary).export(exporter)?);
            for shape in geometry.unnamed_shapes.iter() {
                elements.extend((name_buf, shape).export(exporter)?);
            }
            for (_, shape) in geometry.named_shapes.iter() {
                elements.extend((name_buf, shape).export(exporter)?);
            }
        }
        Ok(elements)
    }
}

impl ExportGds for IndexMap<NameBuf, PortGeometry> {
    type Output = Vec<gds::GdsElement>;

    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let mut elements = Vec::new();
        for (name_buf, geometry) in self {
            elements.extend((name_buf, &geometry.primary).export(exporter)?);
            for shape in geometry.unnamed_shapes.iter() {
                elements.extend((name_buf, shape).export(exporter)?);
            }
            for (_, shape) in geometry.named_shapes.iter() {
                elements.extend((name_buf, shape).export(exporter)?);
            }
        }
        Ok(elements)
    }
}

/// A trait that describes where to place a label for a given shape.
trait PlaceLabels {
    /// Computes a [`Point`] that lies within `self`.
    ///
    /// Allows for placing labels on an arbitrary shape.
    fn label_loc(&self) -> Point;
}

impl PlaceLabels for Shape {
    fn label_loc(&self) -> Point {
        self.shape().label_loc()
    }
}

impl PlaceLabels for geometry::shape::Shape {
    fn label_loc(&self) -> Point {
        match self {
            geometry::shape::Shape::Rect(ref r) => r.label_loc(),
            geometry::shape::Shape::Polygon(ref p) => p.label_loc(),
        }
    }
}

impl PlaceLabels for Rect {
    fn label_loc(&self) -> Point {
        self.center()
    }
}

impl PlaceLabels for Polygon {
    fn label_loc(&self) -> Point {
        self.center()
    }
}

impl ExportGds for (&NameBuf, &IoShape) {
    type Output = Vec<gds::GdsElement>;

    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let (name_buf, shape) = *self;
        let mut elements = Vec::new();
        if let Some(element) =
            Shape::new(shape.layer().pin(), shape.shape().clone()).export(exporter)?
        {
            elements.push(element);
        }
        if let Some(element) = Text::new(
            shape.layer().label(),
            name_buf.to_string(),
            Transformation::from_offset(shape.shape().label_loc()),
        )
        .export(exporter)?
        {
            elements.push(element.into());
        }
        Ok(elements)
    }
}

impl ExportGds for Element {
    type Output = Option<gds::GdsElement>;

    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let span = span!(Level::INFO, "element", element = ?self);
        let _guard = span.enter();

        Ok(match self {
            Element::Instance(instance) => Some(instance.export(exporter)?.into()),
            Element::Shape(shape) => shape.export(exporter)?,
            Element::Text(text) => text.export(exporter)?.map(|text| text.into()),
        })
    }
}

impl ExportGds for RawInstance {
    type Output = gds::GdsStructRef;

    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let span = span!(Level::INFO, "instance", instance = ?self);
        let _guard = span.enter();

        let cell_name = if let Some(name) = exporter.get_name(&self.cell) {
            name
        } else {
            self.cell.export(exporter)?.name
        };

        Ok(gds::GdsStructRef {
            name: cell_name,
            xy: self.trans.offset_point().export(exporter)?,
            strans: Some(self.trans.orientation().export(exporter)?),
            ..Default::default()
        })
    }
}

impl ExportGds for Shape {
    type Output = Option<gds::GdsElement>;

    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let span = span!(Level::INFO, "shape", shape = ?self);
        let _guard = span.enter();

        Ok(if let Some(layer) = self.layer().export(exporter)? {
            Some(match self.shape() {
                geometry::shape::Shape::Rect(r) => gds::GdsBoundary {
                    layer: layer.layer,
                    datatype: layer.xtype,
                    xy: r.export(exporter)?,
                    ..Default::default()
                }
                .into(),
                geometry::shape::Shape::Polygon(p) => gds::GdsBoundary {
                    layer: layer.layer,
                    datatype: layer.xtype,
                    xy: p.export(exporter)?,
                    ..Default::default()
                }
                .into(),
            })
        } else {
            None
        })
    }
}

impl ExportGds for Text {
    type Output = Option<gds::GdsTextElem>;

    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let span = span!(Level::INFO, "text", text = ?self);
        let _guard = span.enter();

        Ok(if let Some(layer) = self.layer().export(exporter)? {
            Some(gds::GdsTextElem {
                string: self.text().clone(),
                layer: layer.layer,
                texttype: layer.xtype,
                xy: self.trans.offset_point().export(exporter)?,
                strans: Some(self.trans.orientation().export(exporter)?),
                ..Default::default()
            })
        } else {
            None
        })
    }
}

impl ExportGds for Rect {
    type Output = Vec<gds::GdsPoint>;

    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let span = span!(Level::INFO, "rect", rect = ?self);
        let _guard = span.enter();

        let bl = self.corner(Corner::LowerLeft).export(exporter)?;
        let br = self.corner(Corner::LowerRight).export(exporter)?;
        let ur = self.corner(Corner::UpperRight).export(exporter)?;
        let ul = self.corner(Corner::UpperLeft).export(exporter)?;
        Ok(vec![bl.clone(), br, ur, ul, bl])
    }
}

impl ExportGds for Polygon {
    type Output = Vec<gds::GdsPoint>;

    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let span = span!(Level::INFO, "polygon", polygon = ?self);
        let _guard = span.enter();

        let mut points: Vec<gds::GdsPoint> =
            self.points()
                .iter()
                .map(|p| p.export(exporter))
                .collect::<Result<Vec<gds::GdsPoint>, GdsExportError>>()?;
        let point0 = self.points()[0].export(exporter)?;

        points.push(point0);
        Ok(points)
    }
}

impl ExportGds for Orientation {
    type Output = gds::GdsStrans;

    fn export(&self, _exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let span = span!(Level::INFO, "orientation", orientation = ?self);
        let _guard = span.enter();

        Ok(gds::GdsStrans {
            reflected: self.reflect_vert(),
            angle: Some(self.angle()),
            ..Default::default()
        })
    }
}

impl ExportGds for Point {
    type Output = gds::GdsPoint;

    fn export(&self, _exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let span = span!(Level::INFO, "point", point = ?self);
        let _guard = span.enter();

        let x = self.x.try_into().map_err(|e| {
            tracing::event!(
                Level::ERROR,
                "failed to convert coordinate to i32: {}",
                self.x
            );
            e
        })?;
        let y = self.y.try_into().map_err(|e| {
            tracing::event!(
                Level::ERROR,
                "failed to convert coordinate to i32: {}",
                self.x
            );
            e
        })?;
        Ok(gds::GdsPoint::new(x, y))
    }
}

impl ExportGds for LayerId {
    type Output = Option<gds::GdsLayerSpec>;

    fn export(&self, exporter: &mut GdsExporter<'_>) -> GdsExportResult<Self::Output> {
        let span = span!(Level::INFO, "layer ID", layer_id = ?self);
        let _guard = span.enter();

        let spec = exporter.get_layer(*self).map(|spec| spec.into());

        if spec.is_none() {
            tracing::event!(
                Level::WARN,
                "skipping export of layer {:?} as no corresponding GDS layer was found",
                self
            );
        }

        Ok(spec)
    }
}

/// An importer for GDS files.
pub struct GdsImporter<'a> {
    cells: HashMap<ArcStr, Arc<RawCell>>,
    gds: &'a gds::GdsLibrary,
    layouts: &'a mut LayoutContext,
    layers: &'a mut LayerContext,
    units: Option<Decimal>,
}

/// An imported GDS file, after conversion to Substrate [`RawCell`]s.
#[derive(Debug, Clone)]
pub struct ImportedGds {
    /// A mapping from cell name to imported cell.
    pub cells: HashMap<ArcStr, Arc<RawCell>>,
}

impl<'a> GdsImporter<'a> {
    /// Creates a new GDS importer.
    pub fn new(
        gds: &'a gds::GdsLibrary,
        layouts: &'a mut LayoutContext,
        layers: &'a mut LayerContext,
        units: Option<Decimal>,
    ) -> Self {
        Self {
            cells: Default::default(),
            gds,
            layouts,
            layers,
            units,
        }
    }

    /// Imports a [`gds::GdsLibrary`].
    pub fn import(mut self) -> GdsImportResult<ImportedGds> {
        self.run_preimport_checks()?;
        for strukt in GdsDepOrder::new(self.gds).total_order() {
            self.import_and_add(strukt)?;
        }
        Ok(ImportedGds { cells: self.cells })
    }

    /// Imports a single cell and all of its dependencies into the provided cell.
    pub fn import_cell(&mut self, name: impl Into<ArcStr>) -> GdsImportResult<Arc<RawCell>> {
        let name = name.into();
        self.run_preimport_checks()?;

        let mut cell = None;
        for strukt in GdsDepOrder::new(self.gds).cell_order(name.clone()) {
            if strukt.name == name {
                cell = Some(self.import_and_add(strukt)?);
            } else {
                self.import_and_add(strukt)?;
            }
        }

        match cell {
            Some(cell) => Ok(cell),
            None => Err(GdsImportError::CellNotFound(name)),
        }
    }
    /// Runs relevant checks before importing from a GDS library.
    fn run_preimport_checks(&mut self) -> GdsImportResult<()> {
        // Unsupported GDSII features, if ever added, shall be imported here:
        // if gdslib.libdirsize.is_some()
        //     || gdslib.srfname.is_some()
        //     || gdslib.libsecur.is_some()
        //     || gdslib.reflibs.is_some()
        //     || gdslib.fonts.is_some()
        //     || gdslib.attrtable.is_some()
        //     || gdslib.generations.is_some()
        //     || gdslib.format_type.is_some()
        // {
        //     return self.fail("Unsupported GDSII Feature");
        // }
        // And convert each of its `structs` into our `cells`

        self.check_units(&self.gds.units)
    }
    /// Checks that the database units match up with the units specified by the PDK.
    fn check_units(&mut self, units: &gds::GdsUnits) -> GdsImportResult<()> {
        let gdsunit = units.db_unit();

        if let Some(expected_units) = self.units {
            if (Decimal::try_from(gdsunit).unwrap() - expected_units).abs() / expected_units
                > dec!(1e-3)
            {
                return Err(GdsImportError::MismatchedUnits(
                    Decimal::try_from(gdsunit).unwrap(),
                    expected_units,
                ));
            }
        }
        Ok(())
    }
    /// Imports and adds a cell if not already defined
    fn import_and_add(&mut self, strukt: &gds::GdsStruct) -> GdsImportResult<Arc<RawCell>> {
        let name = &strukt.name;
        // Check whether we're already defined, and bail if so
        if self.cells.get(name).is_some() {
            return Err(GdsImportError::DuplicateCell(name.clone()));
        }

        let id = self.layouts.get_id();

        // Add it to our library
        let mut cell = RawCell::new(id, name);
        self.import_gds_struct(strukt, &mut cell)?;
        // TODO: self.data.layouts_mut().set_cell(cell);
        let cell = Arc::new(cell);
        // And add the cell to our name-map
        self.cells.insert(name.clone(), cell.clone());
        Ok(cell)
    }
    /// Imports a GDS Cell ([gds::GdsStruct]) into a [Cell]
    fn import_gds_struct(
        &mut self,
        strukt: &gds::GdsStruct,
        cell: &mut RawCell,
    ) -> GdsImportResult<()> {
        let span = span!(Level::INFO, "cell", name=%cell.name);
        let _guard = span.enter();
        // Importing each layout requires at least two passes over its elements.
        // In the first pass we add each [Instance] and geometric element,
        // And keep a list of [gds::GdsTextElem] on the side.
        let mut texts: Vec<&gds::GdsTextElem> = Vec::new();
        let mut elems: SlotMap<ElementKey, Shape> = SlotMap::with_key();
        // Also keep a hash of by-layer elements, to aid in text-assignment in our second pass
        let mut layers: HashMap<LayerId, Vec<ElementKey>> = HashMap::new();
        for elem in &strukt.elems {
            use gds::GdsElement::*;
            let e = match elem {
                GdsBoundary(ref x) => Some(self.import_boundary(x)?),
                GdsPath(ref x) => Some(self.import_path(x)?),
                GdsBox(ref x) => Some(self.import_box(x)?),
                GdsArrayRef(ref x) => {
                    let elems = self.import_instance_array(x)?;
                    cell.elements.reserve(elems.len());
                    for elem in elems {
                        cell.add_element(elem);
                    }
                    None
                }
                GdsStructRef(ref x) => {
                    cell.add_element(self.import_instance(x)?);
                    None
                }
                GdsTextElem(ref x) => {
                    texts.push(x);
                    None
                }
                // GDSII "Node" elements are fairly rare, and are not supported.
                // (Maybe some day we'll even learn what they are.)
                GdsNode(ref elem) => {
                    tracing::warn!(?elem, "ignoring unsupported GDS Node element");
                    None
                }
            };
            // If we got a new element, add it to our per-layer hash
            if let Some(e) = e {
                let layer = e.layer();
                let key = elems.insert(e);
                if let Some(ref mut bucket) = layers.get_mut(&layer) {
                    bucket.push(key);
                } else {
                    layers.insert(layer, vec![key]);
                }
            }
        }
        // Pass two: sort out whether each [gds::GdsTextElem] is a net-label,
        // And if so, assign it as a net-name on each intersecting [Element].
        // Text elements which do not overlap a geometric element on the same layer
        // are converted to annotations.
        for textelem in &texts {
            // Import the GDS text element into a Substrate text element, creating missing layers
            // as necessary.
            let text_elem = self.import_text_elem(textelem)?;

            let net_name = ArcStr::from(textelem.string.to_lowercase());
            let text_layer = self
                .layers
                .get_gds_layer(textelem.layerspec().try_into()?)
                .unwrap();
            let loc = self.import_point(&textelem.xy)?;

            let family = self.layers.layer_family_for_layer_id(text_layer);
            let pin_layer = family.and_then(|f| f.pin);
            let extract_pins =
                Some(text_layer) == family.and_then(|f| f.label) && pin_layer.is_some();

            if extract_pins {
                tracing::debug!("importing port `{}`", net_name);
                let pin_layer = pin_layer.unwrap();
                let family = family.unwrap();
                let mut port = crate::io::Signal.builder();
                let mut has_geometry = false;
                if let Some(layer) = layers.get_mut(&pin_layer) {
                    // Layer exists in geometry; see which elements intersect with this text
                    for ekey in layer.iter() {
                        let elem = elems.get_mut(*ekey);
                        if elem.is_none() {
                            continue;
                        }

                        let elem = elem.unwrap();

                        use crate::geometry::contains::Contains;

                        if elem.shape().contains(&loc).is_full() {
                            port.push(IoShape::new(
                                family.primary,
                                pin_layer,
                                text_layer,
                                elem.shape().clone(),
                            ));
                            has_geometry = true;

                            // This pin shape is stored in a port.
                            // No need to also include it as a regular element.
                            elems.remove(*ekey);
                        }
                    }
                }
                if !has_geometry {
                    tracing::warn!("ignoring empty port: `{}`", net_name);
                    continue;
                }
                // Unwrapping is OK because in the lines above, we continue if no geometry was found.
                // Thus, this port should have at least one shape, and `port.build()` should not
                // error.
                cell.merge_port(net_name, port.build().unwrap());
            } else {
                // Import the text element as is
                cell.add_element(text_elem);
            }
        }
        // Pull the elements out of the local slot-map, into the vector that [Layout] wants
        for elem in elems.drain().map(|(_k, v)| v) {
            cell.add_element(elem);
        }
        Ok(())
    }
    /// Imports a [gds::GdsBoundary] into a [Shape]
    fn import_boundary(&mut self, x: &gds::GdsBoundary) -> GdsImportResult<Shape> {
        let span = span!(Level::INFO, "boundary", value=?x);
        let _guard = span.enter();

        let mut pts: Vec<Point> = self.import_point_vec(&x.xy)?;
        if pts[0] != *pts.last().unwrap() {
            return Err(GdsImportError::InvalidGdsBoundary);
        }
        // Pop the redundant last entry
        pts.pop();
        // Check for Rectangles; they help
        let inner = if pts.len() == 4
            && ((pts[0].x == pts[1].x // Clockwise
            && pts[1].y == pts[2].y
            && pts[2].x == pts[3].x
            && pts[3].y == pts[0].y)
                || (pts[0].y == pts[1].y // Counter-clockwise
            && pts[1].x == pts[2].x
            && pts[2].y == pts[3].y
            && pts[3].x == pts[0].x))
        {
            // That makes this a Rectangle.
            geometry::shape::Shape::Rect(Rect::new(pts[0], pts[2]))
        } else {
            // Otherwise, it's a polygon
            geometry::shape::Shape::Polygon(Polygon::from_verts(pts))
        };

        // Grab (or create) its [Layer]
        let layer = self.import_element_layer(x)?;
        // Create the Element, and insert it in our slotmap
        let shape = Shape::new(layer, inner);
        Ok(shape)
    }
    /// Imports a [gds::GdsBox] into a [Shape]
    fn import_box(&mut self, gds_box: &gds::GdsBox) -> GdsImportResult<Shape> {
        let span = span!(Level::INFO, "box", value=?gds_box);
        let _guard = span.enter();

        // GDS stores *five* coordinates per box (for whatever reason).
        // This does not check fox "box validity", and imports the
        // first and third of those five coordinates,
        // which are by necessity for a valid [GdsBox] located at opposite corners.
        let inner = geometry::shape::Shape::Rect(Rect::new(
            self.import_point(&gds_box.xy[0])?,
            self.import_point(&gds_box.xy[2])?,
        ));

        // Grab (or create) its [Layer]
        let layer = self.import_element_layer(gds_box)?;
        // Create the Element, and insert it in our slotmap
        let shape = Shape::new(layer, inner);
        Ok(shape)
    }
    /// Import a [gds::GdsPath] into an [Element]
    fn import_path(&mut self, x: &gds::GdsPath) -> GdsImportResult<Shape> {
        let span = span!(Level::INFO, "path");
        let _guard = span.enter();

        let pts = self.import_point_vec(&x.xy)?;
        let width = if let Some(w) = x.width {
            w as usize
        } else {
            return Err(GdsImportError::Unsupported(arcstr::literal!(
                "GDS path width must be specified"
            )));
        };

        let layer = self.import_element_layer(x)?;

        if pts.iter().all(|pt| pt.x == pts[0].x) {
            Ok(Shape::new(
                layer,
                Rect::from_spans(
                    Span::from_center_span(pts[0].x, width as i64),
                    Span::new(
                        pts.iter().map(|pt| pt.y).min().unwrap(),
                        pts.iter().map(|pt| pt.y).max().unwrap(),
                    ),
                ),
            ))
        } else if pts.iter().all(|pt| pt.y == pts[0].y) {
            Ok(Shape::new(
                layer,
                Rect::from_spans(
                    Span::new(
                        pts.iter().map(|pt| pt.x).min().unwrap(),
                        pts.iter().map(|pt| pt.x).max().unwrap(),
                    ),
                    Span::from_center_span(pts[0].y, width as i64),
                ),
            ))
        } else {
            Err(GdsImportError::Unsupported(arcstr::literal!(
                "2D GDS paths not yet supported"
            )))
        }
    }
    /// Import a [gds::GdsTextElem] cell/struct-instance into an [TextElement].
    fn import_text_elem(&mut self, sref: &gds::GdsTextElem) -> GdsImportResult<Text> {
        let string = ArcStr::from(sref.string.to_lowercase());
        let span = span!(Level::INFO, "text element", text = %string);
        let _guard = span.enter();

        // Convert its location
        let loc = self.import_point(&sref.xy)?;
        let layer = self.import_element_layer(sref)?;
        Ok(Text::new(layer, string, Transformation::from_offset(loc)))
    }
    /// Import a [gds::GdsStructRef] cell/struct-instance into an [Instance]
    fn import_instance(&mut self, sref: &gds::GdsStructRef) -> GdsImportResult<RawInstance> {
        let span = span!(Level::INFO, "instance", name = %sref.name, loc = %sref.xy);
        let _guard = span.enter();

        // Look up the cell-key, which must be imported by now
        let cell = self
            .cells
            .get(&sref.name)
            .ok_or_else(|| GdsImportError::CellNotFound(sref.name.clone()))?
            .clone();
        // Convert its location
        let loc = self.import_point(&sref.xy)?;
        Ok(RawInstance::new(
            cell,
            Transformation::from_offset_and_orientation(
                loc,
                sref.strans
                    .as_ref()
                    .map(|value| self.import_orientation(value))
                    .map_or(Ok(None), |v| v.map(Some))?
                    .unwrap_or_default(),
            ),
        ))
    }
    /// Imports a (two-dimensional) [`gds::GdsArrayRef`] into [`Instance`]s.
    ///
    /// Returns the newly-created [`Instance`]s as a vector.
    /// Instance names are of the form `{array.name}[{col}][{row}]`.
    ///
    /// GDSII arrays are described by three spatial points:
    /// The origin, extent in "rows", and extent in "columns".
    /// In principle these need not be the same as "x" and "y" spacing,
    /// i.e. there might be "diamond-shaped" array specifications.
    ///
    /// Here, arrays are supported if they are "specified rectangular",
    /// i.e. that (a) the first two points align in `y`, and (b) the second two points align in `x`.
    ///
    /// Further support for such "non-rectangular-specified" arrays may (or may not) become a future addition,
    /// based on observed GDSII usage.
    fn import_instance_array(
        &mut self,
        aref: &gds::GdsArrayRef,
    ) -> GdsImportResult<Vec<RawInstance>> {
        let span = span!(Level::INFO, "instance array");
        let _guard = span.enter();

        // Look up the cell, which must be imported by now
        let cell = self
            .cells
            .get(&aref.name)
            .ok_or_else(|| GdsImportError::CellNotFound(aref.name.clone()))?;
        let cell = Arc::clone(cell);

        // Convert its three (x,y) coordinates
        let p0 = self.import_point(&aref.xy[0])?;
        let p1 = self.import_point(&aref.xy[1])?;
        let p2 = self.import_point(&aref.xy[2])?;
        // Check for (thus far) unsupported non-rectangular arrays
        if p0.y != p1.y || p0.x != p2.x {
            return Err(GdsImportError::Unsupported(arcstr::literal!(
                "unsupported non-rectangular GDS array"
            )));
        }
        // Sort out the inter-element spacing
        let xstep = (p1.x - p0.x) / i64::from(aref.cols);
        let ystep = (p2.y - p0.y) / i64::from(aref.rows);

        // Incorporate the reflection/ rotation settings
        let mut orientation = Orientation::default();
        if let Some(strans) = &aref.strans {
            orientation = self.import_orientation(strans)?;
        }

        // Create the Instances
        let mut insts = Vec::with_capacity((aref.rows * aref.cols) as usize);
        for ix in 0..i64::from(aref.cols) {
            let x = p0.x + ix * xstep;
            for iy in 0..i64::from(aref.rows) {
                let y = p0.y + iy * ystep;
                insts.push(RawInstance::new(
                    cell.clone(),
                    Transformation::from_offset_and_orientation(Point::new(x, y), orientation),
                ));
            }
        }
        Ok(insts)
    }
    /// Imports a [`Point`].
    fn import_point(&self, pt: &gds::GdsPoint) -> GdsImportResult<Point> {
        let x = pt.x.into();
        let y = pt.y.into();
        Ok(Point::new(x, y))
    }
    /// Imports a vector of [`Point`]s.
    fn import_point_vec(&mut self, pts: &[gds::GdsPoint]) -> GdsImportResult<Vec<Point>> {
        pts.iter()
            .map(|p| self.import_point(p))
            .collect::<Result<Vec<_>, _>>()
    }
    /// Imports an orientation.
    fn import_orientation(&mut self, strans: &gds::GdsStrans) -> GdsImportResult<Orientation> {
        let span = span!(Level::INFO, "orientation", value=?strans);
        let _guard = span.enter();

        if strans.abs_mag || strans.abs_angle {
            return Err(GdsImportError::Unsupported(arcstr::literal!(
                "absolute magnitude/absolute angle are unsupported"
            )));
        }
        if strans.mag.is_some() {
            return Err(GdsImportError::Unsupported(arcstr::literal!(
                "orientation magnitude unsupported"
            )));
        }

        let orientation =
            Orientation::from_reflect_and_angle(strans.reflected, strans.angle.unwrap_or_default());
        Ok(orientation)
    }
    /// Gets the [`LayerSpec`] for a GDS element implementing its [`gds::HasLayer`] trait.
    /// Layers are created if they do not already exist,
    /// although this may eventually be a per-importer setting.
    fn import_element_layer(&mut self, elem: &impl gds::HasLayer) -> GdsImportResult<LayerId> {
        let spec = elem.layerspec();
        let span = span!(Level::INFO, "layer", spec=?spec);
        let _guard = span.enter();
        let spec = spec.try_into()?;
        let layers = &mut self.layers;
        Ok(if let Some(layer_spec) = layers.get_gds_layer(spec) {
            layer_spec
        } else {
            self.layers.new_layer_with_id(|id| LayerInfo {
                id,
                name: arcstr::format!("gds_{}_{}", spec.0, spec.1),
                gds: Some(spec),
            })
        })
    }
}

/// A helper for retrieving GDS dependencies in reverse topological order.
///
/// Creates a vector of references Gds structs, ordered by their instance dependencies.
/// Each item in the ordered return value is guaranteed *not* to instantiate any item which comes later.
#[derive(Debug)]
pub struct GdsDepOrder<'a> {
    strukts: HashMap<ArcStr, &'a gds::GdsStruct>,
    stack: Vec<&'a gds::GdsStruct>,
    seen: HashSet<ArcStr>,
}

impl<'a> GdsDepOrder<'a> {
    /// Creates a new [`GdsDepOrder`] for a [`gds::GdsLibrary`].
    fn new(gdslib: &'a gds::GdsLibrary) -> Self {
        // First create a map from names to structs
        let mut strukts = HashMap::new();
        for s in &gdslib.structs {
            strukts.insert(s.name.clone(), s);
        }
        Self {
            strukts,
            stack: Vec::new(),
            seen: HashSet::new(),
        }
    }
    /// Returns a reverse topological sort of all structs in `gdslib`.
    fn total_order(mut self) -> Vec<&'a gds::GdsStruct> {
        let strukts = self
            .strukts
            .values()
            .copied()
            .collect::<Vec<&gds::GdsStruct>>();
        for s in strukts {
            self.push(s)
        }
        self.stack
    }
    /// Returns all dependencies of a given cell in reverse topological order.
    fn cell_order(mut self, cell: impl Into<ArcStr>) -> Vec<&'a gds::GdsStruct> {
        if let Some(strukt) = self.strukts.get(&cell.into()) {
            self.push(strukt);
        }
        self.stack
    }
    /// Adds all of `strukt`'s dependencies, and then `strukt` itself, to the stack.
    fn push(&mut self, strukt: &'a gds::GdsStruct) {
        if !self.seen.contains(&strukt.name) {
            for elem in &strukt.elems {
                use gds::GdsElement::*;
                match elem {
                    GdsStructRef(ref x) => self.push(self.strukts.get(&x.name).unwrap()),
                    GdsArrayRef(ref x) => self.push(self.strukts.get(&x.name).unwrap()),
                    _ => (),
                };
            }
            self.seen.insert(strukt.name.clone());
            self.stack.push(strukt);
        }
    }
}
