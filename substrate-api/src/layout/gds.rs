//! Utilities for GDS conversion.
//!
//! Converts between Substrate's layout data-model and [`gds`] structures.

use std::{collections::HashSet, sync::Arc};

use arcstr::ArcStr;
use geometry::{
    prelude::{Corner, Orientation, Point},
    rect::Rect,
};
use tracing::{span, Level};

use crate::pdk::layers::{GdsLayerSpec, LayerContext, LayerId};

use super::{
    element::{Element, RawCell, RawInstance, Shape, Text},
    error::GdsExportResult,
};

/// An exporter for GDS files.
///
/// Takes a [`RawCell`] and converts it to a [`gds::GdsLibrary`].
pub struct GdsExporter<'a> {
    cell: Arc<RawCell>,
    layers: &'a LayerContext,
    cell_names: HashSet<ArcStr>,
    gds: gds::GdsLibrary,
}

impl<'a> GdsExporter<'a> {
    /// Creates a new GDS exporter.
    ///
    /// Requires the cell to be exported and a [`LayerContext`] for mapping Substrate layers to GDS
    /// layers.
    pub fn new(cell: Arc<RawCell>, layers: &'a LayerContext) -> Self {
        Self {
            cell,
            layers,
            cell_names: Default::default(),
            gds: gds::GdsLibrary::new("TOP"),
        }
    }

    /// Exports the contents of `self` as a [`gds::GdsLibrary`].
    pub fn export(mut self) -> GdsExportResult<gds::GdsLibrary> {
        self.cell.clone().export(&mut self)?;
        Ok(self.gds)
    }

    fn assign_name(&mut self, cell: &RawCell) -> ArcStr {
        let name = &cell.name;
        let name = if self.cell_names.contains(name) {
            let mut i = 1;
            loop {
                let new_name = arcstr::format!("{}_{}", name, i);
                if !self.cell_names.contains(&new_name) {
                    break new_name;
                }
                i += 1;
            }
        } else {
            name.clone()
        };

        self.cell_names.insert(name.clone());
        name
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

        for element in self.elements.iter() {
            if let Some(elem) = element.export(exporter)? {
                cell.elems.push(elem);
            }
        }

        exporter.gds.structs.push(cell.clone());

        Ok(cell)
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

        let cell = self.cell.export(exporter)?;

        Ok(gds::GdsStructRef {
            name: cell.name,
            xy: self.loc.export(exporter)?,
            strans: Some(self.orientation.export(exporter)?),
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
                xy: self.loc().export(exporter)?,
                strans: Some(self.orientation().export(exporter)?),
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
