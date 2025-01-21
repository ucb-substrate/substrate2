use std::collections::{HashMap, HashSet};

use arcstr::ArcStr;
use gds::{GdsLibrary, GdsUnits};
use geometry::{
    point::Point,
    prelude::{Orientation, Polygon, Transformation},
    rect::Rect,
    span::Span,
    transform::Rotation,
};
use layir::{Cell, CellId, Instance, Library, LibraryBuilder, Shape, Text};
use slotmap::new_key_type;
use tracing::{span, Level};

use crate::GdsLayer;

new_key_type! {
    /// A key used for identifying elements when importing a GDSII file.
    pub struct ElementKey;
}

pub struct GdsImportOpts {
    pub units: Option<GdsUnits>,
}

pub fn import_gds(lib: &GdsLibrary, opts: GdsImportOpts) -> Result<Library<GdsLayer>> {
    let importer = GdsImporter::new(lib, opts);
    importer.import()
}

/// An importer for GDS files.
pub struct GdsImporter<'a> {
    lib: LibraryBuilder<GdsLayer>,
    gds: &'a gds::GdsLibrary,
    opts: GdsImportOpts,
}

/// An error encountered while converting a GDS library to LayIR.
#[derive(Debug, Clone)]
pub struct GdsImportError;

type Result<T> = std::result::Result<T, GdsImportError>;

impl<'a> GdsImporter<'a> {
    /// Creates a new GDS importer.
    pub fn new(gds: &'a gds::GdsLibrary, opts: GdsImportOpts) -> Self {
        Self {
            lib: LibraryBuilder::new(),
            gds,
            opts,
        }
    }

    /// Imports a [`gds::GdsLibrary`].
    pub fn import(mut self) -> Result<Library<GdsLayer>> {
        self.run_preimport_checks()?;
        for strukt in GdsDepOrder::new(self.gds).total_order() {
            self.import_and_add(strukt)?;
        }
        let lib = self.lib.build().map_err(|_| GdsImportError)?;
        Ok(lib)
    }

    /// Imports a single cell and all of its dependencies into the provided cell.
    pub fn import_cell(mut self, name: impl Into<ArcStr>) -> Result<Library<GdsLayer>> {
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

        if cell.is_none() {
            tracing::event!(Level::ERROR, cell_name = %name, "cell not found: `{}`", name);
            return Err(GdsImportError);
        }

        let lib = self.lib.build().map_err(|_| GdsImportError)?;
        Ok(lib)
    }
    /// Runs relevant checks before importing from a GDS library.
    fn run_preimport_checks(&mut self) -> Result<()> {
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
    fn check_units(&mut self, units: &gds::GdsUnits) -> Result<()> {
        let gdsunit = units.db_unit();

        if let Some(expected_units) = &self.opts.units {
            if (gdsunit - expected_units.db_unit()).abs() / expected_units.db_unit() > 1e-3 {
                return Err(GdsImportError);
            }
        }
        Ok(())
    }
    /// Imports and adds a cell if not already defined
    fn import_and_add(&mut self, strukt: &gds::GdsStruct) -> Result<CellId> {
        let name = &strukt.name;
        // Check whether we're already defined, and bail if so
        if self.lib.try_cell_id_named(name).is_some() {
            tracing::event!(
                Level::ERROR,
                cell_name = %name,
                "duplicate cell name: `{}`",
                name
            );
            return Err(GdsImportError);
        }

        let mut cell = Cell::new(name);
        self.import_gds_struct(strukt, &mut cell)?;
        let id = self.lib.add_cell(cell);
        Ok(id)
    }
    /// Imports a GDS Cell ([gds::GdsStruct]) into a [Cell]
    fn import_gds_struct(
        &mut self,
        strukt: &gds::GdsStruct,
        cell: &mut Cell<GdsLayer>,
    ) -> Result<()> {
        let span = span!(Level::INFO, "cell", name=%cell.name());
        let _guard = span.enter();

        for elem in &strukt.elems {
            use gds::GdsElement::*;
            match elem {
                GdsBoundary(ref x) => cell.add_element(self.import_boundary(x)?),
                GdsPath(ref x) => cell.add_element(self.import_path(x)?),
                GdsBox(ref x) => cell.add_element(self.import_box(x)?),
                GdsArrayRef(ref x) => {
                    let elems = self.import_instance_array(x)?;
                    for elem in elems {
                        cell.add_instance(elem);
                    }
                }
                GdsStructRef(ref x) => {
                    cell.add_instance(self.import_instance(x)?);
                }
                GdsTextElem(ref x) => {
                    cell.add_element(self.import_text_elem(x)?);
                }
                // GDSII "Node" elements are fairly rare, and are not supported.
                // (Maybe some day we'll even learn what they are.)
                GdsNode(ref elem) => {
                    tracing::warn!(?elem, "ignoring unsupported GDS Node element");
                }
            };
        }
        Ok(())
    }
    /// Imports a [gds::GdsBoundary] into a [Shape]
    fn import_boundary(&mut self, x: &gds::GdsBoundary) -> Result<Shape<GdsLayer>> {
        let span = span!(Level::INFO, "boundary", value=?x);
        let _guard = span.enter();

        let mut pts: Vec<Point> = self.import_point_vec(&x.xy)?;
        if pts.is_empty() {
            tracing::event!(
                Level::ERROR,
                "cannot import empty GDS boundary: empty polygons are not permitted"
            );
            return Err(GdsImportError);
        }
        if pts[0] != *pts.last().unwrap() {
            tracing::event!(Level::ERROR, first_pt=?pts[0], last_pt=?pts.last().unwrap(), "invalid GDS boundary: last point of polygon must be the same as the first point, or else the polygon is not closed");
            return Err(GdsImportError);
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

        let layer = self.import_element_layer(x)?;
        let shape = Shape::new(layer, inner);
        Ok(shape)
    }
    /// Imports a [gds::GdsBox] into a [Shape]
    fn import_box(&mut self, gds_box: &gds::GdsBox) -> Result<Shape<GdsLayer>> {
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

        let layer = self.import_element_layer(gds_box)?;
        let shape = Shape::new(layer, inner);
        Ok(shape)
    }
    /// Import a [gds::GdsPath] into an [Element]
    fn import_path(&mut self, x: &gds::GdsPath) -> Result<Shape<GdsLayer>> {
        let span = span!(Level::INFO, "path");
        let _guard = span.enter();

        let pts = self.import_point_vec(&x.xy)?;
        let width = if let Some(w) = x.width {
            w as i64
        } else {
            tracing::event!(Level::ERROR, "GDS path width must be specified");
            return Err(GdsImportError);
        };

        let layer = self.import_element_layer(x)?;
        let (begin_extn, end_extn) = match x.path_type {
            Some(0) => (0, 0),
            Some(2) => (width / 2, width / 2),
            Some(4) => (
                x.begin_extn.unwrap_or_default() as i64,
                x.end_extn.unwrap_or_default() as i64,
            ),
            None => (0, 0),
            _ => {
                tracing::event!(
                    Level::ERROR,
                    "Only flush and square path ends are supported"
                );
                return Err(GdsImportError);
            }
        };

        if pts.iter().all(|pt| pt.x == pts[0].x) {
            Ok(Shape::new(
                layer,
                Rect::from_spans(
                    Span::from_center_span(pts[0].x, width),
                    Span::new(
                        pts.iter().map(|pt| pt.y).min().unwrap(),
                        pts.iter().map(|pt| pt.y).max().unwrap(),
                    )
                    .union(Span::from_point(pts[0].y).expand_all(begin_extn))
                    .union(Span::from_point(pts[pts.len() - 1].y).expand_all(end_extn)),
                ),
            ))
        } else if pts.iter().all(|pt| pt.y == pts[0].y) {
            Ok(Shape::new(
                layer,
                Rect::from_spans(
                    Span::new(
                        pts.iter().map(|pt| pt.x).min().unwrap(),
                        pts.iter().map(|pt| pt.x).max().unwrap(),
                    )
                    .union(Span::from_point(pts[0].x).expand_all(begin_extn))
                    .union(Span::from_point(pts[pts.len() - 1].x).expand_all(end_extn)),
                    Span::from_center_span(pts[0].y, width),
                ),
            ))
        } else {
            tracing::event!(Level::ERROR, "2D GDS paths not supported");
            Err(GdsImportError)
        }
    }
    /// Import a [gds::GdsTextElem] cell/struct-instance into an [TextElement].
    fn import_text_elem(&mut self, sref: &gds::GdsTextElem) -> Result<Text<GdsLayer>> {
        let string = ArcStr::from(sref.string.to_lowercase());
        let span = span!(Level::INFO, "text element", text = %string);
        let _guard = span.enter();

        // Convert its location
        let loc = self.import_point(&sref.xy)?;
        let layer = self.import_element_layer(sref)?;
        Ok(Text::with_transformation(
            layer,
            string,
            Transformation::from_offset(loc),
        ))
    }
    /// Import a [gds::GdsStructRef] cell/struct-instance into an [Instance]
    fn import_instance(&mut self, sref: &gds::GdsStructRef) -> Result<Instance> {
        let span = span!(Level::INFO, "instance", name = %sref.name, loc = %sref.xy);
        let _guard = span.enter();

        // Look up the cell-key, which must be imported by now
        let cell = self.lib.try_cell_id_named(&sref.name).ok_or_else(|| {
            tracing::event!(Level::ERROR, cell_name=%sref.name, "cell not found: `{}`", sref.name);
            GdsImportError
        })?;
        // Convert its location
        let loc = self.import_point(&sref.xy)?;
        Ok(Instance::with_transformation(
            cell,
            sref.name.clone(),
            Transformation::from_offset_and_orientation(
                loc,
                sref.strans
                    .as_ref()
                    .map(|value| self.import_orientation(value))
                    .transpose()?
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
    fn import_instance_array(&mut self, aref: &gds::GdsArrayRef) -> Result<Vec<Instance>> {
        let span = span!(Level::INFO, "instance array");
        let _guard = span.enter();

        // Look up the cell, which must be imported by now
        let cell = self.lib.try_cell_id_named(&aref.name).ok_or_else(|| {
            tracing::event!(Level::ERROR, cell_name=%aref.name, "cell not found: `{}`", aref.name);
            GdsImportError
        })?;

        // Convert its three (x,y) coordinates
        let p0 = self.import_point(&aref.xy[0])?;
        let p1 = self.import_point(&aref.xy[1])?;
        let p2 = self.import_point(&aref.xy[2])?;
        // Check for (thus far) unsupported non-rectangular arrays
        if p0.y != p1.y || p0.x != p2.x {
            tracing::event!(Level::ERROR, p0=?p0, p1=?p1, p2=?p2, "unsupported non-rectangular GDS instance array");
            return Err(GdsImportError);
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
                insts.push(Instance::with_transformation(
                    cell,
                    arcstr::format!("{}_{}_{}", aref.name, ix, iy),
                    Transformation::from_offset_and_orientation(Point::new(x, y), orientation),
                ));
            }
        }
        Ok(insts)
    }
    /// Imports a [`Point`].
    fn import_point(&self, pt: &gds::GdsPoint) -> Result<Point> {
        let x = pt.x.into();
        let y = pt.y.into();
        Ok(Point::new(x, y))
    }
    /// Imports a vector of [`Point`]s.
    fn import_point_vec(&mut self, pts: &[gds::GdsPoint]) -> Result<Vec<Point>> {
        pts.iter()
            .map(|p| self.import_point(p))
            .collect::<Result<Vec<_>>>()
    }
    /// Imports an orientation.
    fn import_orientation(&mut self, strans: &gds::GdsStrans) -> Result<Orientation> {
        let span = span!(Level::INFO, "orientation", value=?strans);
        let _guard = span.enter();

        if strans.abs_mag || strans.abs_angle {
            tracing::event!(
                Level::ERROR,
                "absolute magnitude/absolute angle are unsupported"
            );
            return Err(GdsImportError);
        }
        if strans.mag.is_some() {
            tracing::event!(Level::ERROR, "orientation magnitude unsupported");
            return Err(GdsImportError);
        }

        let rotation = Rotation::try_from(strans.angle.unwrap_or_default()).map_err(|_| {
            tracing::event!(Level::ERROR, "rotations must be in 90 degree increments");
            GdsImportError
        })?;
        let orientation = Orientation::from_reflect_and_angle(strans.reflected, rotation);
        Ok(orientation)
    }
    /// Gets the [`LayerSpec`] for a GDS element implementing its [`gds::HasLayer`] trait.
    /// Layers are created if they do not already exist,
    /// although this may eventually be a per-importer setting.
    fn import_element_layer(&mut self, elem: &impl gds::HasLayer) -> Result<GdsLayer> {
        let spec = elem.layerspec();
        let span = span!(Level::INFO, "layer", spec=?spec);
        let _guard = span.enter();
        let first = u16::try_from(spec.layer).map_err(|_| {
            tracing::event!(Level::ERROR, layer=%spec.layer, "failed to convert layer number to u16");
            GdsImportError
        })?;
        let second = u16::try_from(spec.xtype).map_err(|_| {
            tracing::event!(Level::ERROR, layer=%spec.layer, "failed to convert layer xtype number to u16");
            GdsImportError
        })?;
        Ok(GdsLayer(first, second))
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
