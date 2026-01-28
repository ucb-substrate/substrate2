//! Converting GDS libraries.

use std::collections::HashMap;
use std::hash::Hash;

use crate::GdsLayer;
use arcstr::ArcStr;
use geometry::prelude::Contains;
use layir::{Cell, Direction, Element, Instance, LibraryBuilder, Port};
use thiserror::Error;

/// A layer type that can be constructed from a [`GdsLayer`].
pub trait FromGds: Sized {
    /// Converts the given GDS layer to this layer type.
    ///
    /// Returns [`None`] if the GDS layer has no valid mapping to the new layer type.
    fn from_gds(layer: GdsLayer) -> Option<Self>;
    /// Converts the given GDS layer to this layer type if the GDS layer is a pin-type layer.
    ///
    /// Should return [`None`] if the GDS layer has no valid mapping to the new layer type,
    /// or if the given GDS layer is not a pin layer.
    fn from_gds_pin(layer: GdsLayer) -> Option<Self>;
    /// Converts the given GDS layer to this layer type if the GDS layer is a label-type layer.
    ///
    /// Should return [`None`] if the GDS layer has no valid mapping to the new layer type,
    /// or if the given GDS layer is not a label layer.
    fn from_gds_label(layer: GdsLayer) -> Option<Self>;
}

#[derive(Error, Debug)]
pub enum FromGdsError {
    #[error("no layer mapping for layer {layer} in cell `{cell}`")]
    NoLayerMapping { cell: ArcStr, layer: GdsLayer },
    #[error("pin with multiple labels: `{label1}`, `{label2}` in cell `{cell}`")]
    PinWithMultipleLabels {
        cell: ArcStr,
        label1: ArcStr,
        label2: ArcStr,
    },
    #[error("pin with no label in cell `{cell}`")]
    PinWithNoLabel { cell: ArcStr },
    #[error("error building LayIR library")]
    BuildError(#[from] layir::BuildError),
}

/// Convert a GDS layout library to a sky130 layout library.
pub fn from_gds<L: FromGds + Hash + Eq + Clone>(
    lib: &layir::Library<GdsLayer>,
) -> Result<layir::Library<L>, FromGdsError> {
    let mut olib = LibraryBuilder::<L>::new();
    let cells = lib.topological_order();
    for cell in cells {
        let cell = lib.cell(cell);
        let mut ocell = Cell::new(cell.name());
        for elt in cell.elements() {
            let layer = *elt.layer();
            let l = L::from_gds(layer).ok_or_else(|| FromGdsError::NoLayerMapping {
                cell: cell.name().clone(),
                layer,
            })?;
            ocell.add_element(elt.with_layer(l));
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
            let port = oport.map_layer(|layer| L::from_gds(*layer).unwrap());
            ocell.add_port(name, port);
        }
        let mut pin_correspondences: HashMap<L, (Vec<_>, Vec<_>)> = HashMap::new();
        for elt in cell.elements() {
            match elt {
                Element::Shape(s) => {
                    if let Some(layer) = L::from_gds_pin(*s.layer()) {
                        let entry = pin_correspondences.entry(layer.clone()).or_default();
                        entry.0.push(s.with_layer(layer));
                    }
                }
                Element::Text(t) => {
                    if let Some(layer) =
                        L::from_gds_pin(*t.layer()).or_else(|| L::from_gds_label(*t.layer()))
                    {
                        let entry = pin_correspondences.entry(layer.clone()).or_default();
                        entry.1.push(t.with_layer(layer));
                    }
                }
            }
        }
        for (_, (shapes, texts)) in pin_correspondences {
            for shape in shapes {
                let mut name: Option<ArcStr> = None;
                let mut pin_texts = Vec::new();
                for text in texts.iter() {
                    if shape
                        .shape()
                        .contains(&text.transformation().offset_point())
                        .intersects()
                    {
                        // Identify pin shapes with multiple labels.
                        if let Some(ref name) = name
                            && name != text.text()
                        {
                            return Err(FromGdsError::PinWithMultipleLabels {
                                cell: cell.name().clone(),
                                label1: name.clone(),
                                label2: text.text().clone(),
                            });
                        }
                        name = Some(text.text().clone());
                        pin_texts.push(text.clone());
                    }
                }

                // If name is None, no label was found for this shape.
                // We ignore shapes with missing labels.
                // In the future, we can perform connectivity analysis
                // to identify other pins to which this shape is connected,
                // and then add this shape to the appropriate pin.
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
                }
            }
        }
        olib.add_cell(ocell);
    }

    Ok(olib.build()?)
}
