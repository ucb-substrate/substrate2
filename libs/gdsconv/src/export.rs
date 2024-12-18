use arcstr::ArcStr;
use gds::{
    GdsBoundary, GdsElement, GdsLibrary, GdsPoint, GdsStrans, GdsStruct, GdsStructRef, GdsTextElem,
    GdsUnits,
};
use geometry::{
    corner::Corner,
    point::Point,
    prelude::{Orientation, Polygon},
    rect::Rect,
};
use layir::{Cell, Element, Instance, Library, Shape, Text};

use crate::GdsLayer;

pub struct GdsExportOpts {
    /// Name of the GDS library.
    pub name: ArcStr,
    pub units: Option<GdsUnits>,
}

pub fn export_gds(lib: Library<GdsLayer>, opts: GdsExportOpts) -> GdsLibrary {
    let exporter = GdsExporter { opts, lib: &lib };
    exporter.export()
}

struct GdsExporter<'a> {
    opts: GdsExportOpts,
    lib: &'a Library<GdsLayer>,
}

impl GdsExporter<'_> {
    fn export(mut self) -> GdsLibrary {
        let mut gds = if let Some(units) = self.opts.units.clone() {
            GdsLibrary::with_units(self.opts.name.clone(), units)
        } else {
            GdsLibrary::new(self.opts.name.clone())
        };
        for id in self.lib.topological_order() {
            let cell = self.lib.cell(id);
            let strukt = self.export_cell(cell);
            gds.structs.push(strukt);
        }
        gds
    }

    fn export_cell(&mut self, cell: &Cell<GdsLayer>) -> GdsStruct {
        let mut gcell = GdsStruct::new(cell.name().clone());
        for (_, port) in cell.ports() {
            for elt in port.elements() {
                gcell.elems.push(export_element(elt));
            }
        }
        for elt in cell.elements() {
            gcell.elems.push(export_element(elt));
        }
        for (_, inst) in cell.instances() {
            gcell.elems.push(export_instance(self.lib, inst));
        }
        gcell
    }
}

fn export_instance(lib: &Library<GdsLayer>, inst: &Instance) -> GdsElement {
    let cell = lib.cell(inst.child());
    GdsStructRef {
        name: cell.name().clone(),
        xy: export_point(inst.transformation().offset_point()),
        strans: Some(export_orientation(inst.transformation().orientation())),
        ..Default::default()
    }
    .into()
}

fn export_element(elt: &Element<GdsLayer>) -> GdsElement {
    match elt {
        Element::Shape(shape) => export_shape(shape),
        Element::Text(text) => export_text(text),
    }
}

fn export_shape(shape: &Shape<GdsLayer>) -> GdsElement {
    match shape.shape() {
        geometry::shape::Shape::Rect(rect) => GdsBoundary {
            layer: shape.layer().0 as i16,
            datatype: shape.layer().1 as i16,
            xy: export_rect(rect),
            ..Default::default()
        }
        .into(),
        geometry::shape::Shape::Polygon(poly) => GdsBoundary {
            layer: shape.layer().0 as i16,
            datatype: shape.layer().1 as i16,
            xy: export_polygon(poly),
            ..Default::default()
        }
        .into(),
    }
}

fn export_point(p: Point) -> GdsPoint {
    let x = p.x.try_into().unwrap();
    let y = p.y.try_into().unwrap();
    GdsPoint::new(x, y)
}

fn export_polygon(poly: &Polygon) -> Vec<GdsPoint> {
    let mut points: Vec<gds::GdsPoint> = poly
        .points()
        .iter()
        .copied()
        .map(export_point)
        .collect::<Vec<_>>();
    let point0 = export_point(poly.points()[0]);

    points.push(point0);
    points
}

fn export_rect(rect: &Rect) -> Vec<GdsPoint> {
    let bl = export_point(rect.corner(Corner::LowerLeft));
    let br = export_point(rect.corner(Corner::LowerRight));
    let ur = export_point(rect.corner(Corner::UpperRight));
    let ul = export_point(rect.corner(Corner::UpperLeft));
    vec![bl.clone(), br, ur, ul, bl]
}

fn export_text(text: &Text<GdsLayer>) -> GdsElement {
    GdsTextElem {
        string: text.text().clone(),
        layer: text.layer().0 as i16,
        texttype: text.layer().1 as i16,
        xy: export_point(text.transformation().offset_point()),
        strans: Some(export_orientation(text.transformation().orientation())),
        ..Default::default()
    }
    .into()
}

fn export_orientation(orientation: Orientation) -> GdsStrans {
    GdsStrans {
        reflected: orientation.reflect_vert(),
        angle: Some(orientation.angle().degrees()),
        ..Default::default()
    }
}
