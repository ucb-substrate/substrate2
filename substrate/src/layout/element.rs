//! Basic layout elements.
//!
//! Substrate layouts consist of cells, instances, geometric shapes, and text annotations.

use std::{collections::HashMap, sync::Arc};

use arcstr::ArcStr;
use geometry::{
    prelude::{Bbox, Point},
    rect::Rect,
    transform::{
        HasTransformedView, Transform, TransformMut, Transformation, Transformed, TranslateMut,
    },
    union::BoundingUnion,
};
use indexmap::{map::Entry, IndexMap};
use serde::{Deserialize, Serialize};

use crate::io::layout::PortGeometry;
use crate::layout::bbox::LayerBbox;
use crate::{
    error::{Error, Result},
    io::NameBuf,
    pdk::{layers::LayerId, Pdk},
};

use super::{Draw, DrawReceiver, ExportsLayoutData, Instance};

/// A context-wide unique identifier for a cell.
#[derive(
    Default, Debug, Copy, Clone, Serialize, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct CellId(u64);

impl CellId {
    pub(crate) fn increment(&mut self) {
        *self = CellId(self.0 + 1)
    }
}

/// A mapping from names to ports.
pub type NamedPorts = IndexMap<NameBuf, PortGeometry>;

/// A raw layout cell.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct RawCell {
    pub(crate) id: CellId,
    pub(crate) name: ArcStr,
    pub(crate) elements: Vec<Element>,
    pub(crate) blockages: Vec<Shape>,
    ports: NamedPorts,
    port_names: HashMap<String, NameBuf>,
}

impl RawCell {
    pub(crate) fn new(id: CellId, name: impl Into<ArcStr>) -> Self {
        Self {
            id,
            name: name.into(),
            elements: Vec::new(),
            blockages: Vec::new(),
            ports: IndexMap::new(),
            port_names: HashMap::new(),
        }
    }

    pub(crate) fn with_ports(self, ports: NamedPorts) -> Self {
        let port_names = ports.keys().map(|k| (k.to_string(), k.clone())).collect();
        Self {
            ports,
            port_names,
            ..self
        }
    }

    #[doc(hidden)]
    pub fn port_map(&self) -> &NamedPorts {
        &self.ports
    }

    pub(crate) fn add_element(&mut self, elem: impl Into<Element>) {
        self.elements.push(elem.into());
    }

    #[allow(dead_code)]
    pub(crate) fn add_blockage(&mut self, shape: impl Into<Shape>) {
        self.blockages.push(shape.into());
    }

    #[allow(dead_code)]
    pub(crate) fn add_elements(&mut self, elems: impl IntoIterator<Item = impl Into<Element>>) {
        self.elements.extend(elems.into_iter().map(|x| x.into()));
    }

    #[allow(dead_code)]
    pub(crate) fn add_blockages(&mut self, shapes: impl IntoIterator<Item = impl Into<Shape>>) {
        self.blockages.extend(shapes.into_iter().map(|x| x.into()));
    }

    /// Merges a port into this cell.
    ///
    /// Primarily for use in GDS import.
    pub(crate) fn merge_port(&mut self, name: impl Into<NameBuf>, port: impl Into<PortGeometry>) {
        let name = name.into();
        match self.ports.entry(name.clone()) {
            Entry::Occupied(mut o) => {
                o.get_mut().merge(port.into());
            }
            Entry::Vacant(v) => {
                v.insert(port.into());
            }
        }
        self.port_names.insert(name.to_string(), name);
    }

    /// The ID of this cell.
    pub fn id(&self) -> CellId {
        self.id
    }

    /// Returns an iterator over the elements of this cell.
    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.elements.iter()
    }

    /// Returns an iterator over the ports of this cell, as `(name, geometry)` pairs.
    pub fn ports(&self) -> impl Iterator<Item = (&NameBuf, &PortGeometry)> {
        self.ports.iter()
    }

    /// Returns a reference to the port with the given name, if it exists.
    pub fn port_named(&self, name: &str) -> Option<&PortGeometry> {
        let name_buf = self.port_names.get(name)?;
        self.ports.get(name_buf)
    }
}

impl Bbox for RawCell {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.elements.bbox()
    }
}

impl LayerBbox for RawCell {
    fn layer_bbox(&self, layer: LayerId) -> Option<Rect> {
        self.elements.layer_bbox(layer)
    }
}

impl HasTransformedView for RawCell {
    type TransformedView = RawCell;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView {
        RawCell {
            id: self.id,
            name: self.name.clone(),
            elements: self.elements.transformed_view(trans),
            blockages: self.blockages.transformed_view(trans),
            ports: self
                .ports
                .iter()
                .map(|(k, v)| (k.clone(), v.transformed_view(trans)))
                .collect(),
            port_names: self.port_names.clone(),
        }
    }
}

/// A raw layout instance.
///
/// Consists of a pointer to an underlying cell and its instantiated transformation.
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct RawInstance {
    pub(crate) cell: Arc<RawCell>,
    pub(crate) trans: Transformation,
}

impl RawInstance {
    /// Create a new raw instance of the given cell.
    pub fn new(cell: impl Into<Arc<RawCell>>, trans: Transformation) -> Self {
        Self {
            cell: cell.into(),
            trans,
        }
    }

    /// Returns a reference to the child cell.
    ///
    /// The returned object provides coordinates in the parent cell's coordinate system.
    /// If you want coordinates in the child cell's coordinate system,
    /// consider using [`RawInstance::raw_cell`] instead.
    #[inline]
    pub fn cell(&self) -> Transformed<RawCell> {
        self.cell.transformed_view(self.trans)
    }

    /// Returns a raw reference to the child cell.
    ///
    /// The returned cell does not store any information related
    /// to this instance's transformation.
    /// Consider using [`RawInstance::cell`] instead.
    #[inline]
    pub fn raw_cell(&self) -> &RawCell {
        &self.cell
    }
}

impl Bbox for RawInstance {
    fn bbox(&self) -> Option<Rect> {
        self.cell.bbox().map(|rect| rect.transform(self.trans))
    }
}

impl LayerBbox for RawInstance {
    fn layer_bbox(&self, layer: LayerId) -> Option<Rect> {
        self.cell
            .layer_bbox(layer)
            .map(|rect| rect.transform(self.trans))
    }
}

impl<T: ExportsLayoutData> TryFrom<Instance<T>> for RawInstance {
    type Error = Error;

    fn try_from(value: Instance<T>) -> Result<Self> {
        Ok(Self {
            cell: value.try_cell()?.raw,
            trans: value.trans,
        })
    }
}

impl<PDK: Pdk> Draw<PDK> for RawInstance {
    fn draw(self, recv: &mut DrawReceiver<PDK>) -> Result<()> {
        recv.draw_element(self);
        Ok(())
    }
}

impl TranslateMut for RawInstance {
    fn translate_mut(&mut self, p: Point) {
        self.transform_mut(Transformation::from_offset(p));
    }
}

impl TransformMut for RawInstance {
    fn transform_mut(&mut self, trans: Transformation) {
        self.trans = Transformation::cascade(trans, self.trans);
    }
}

impl HasTransformedView for RawInstance {
    type TransformedView = RawInstance;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView {
        self.clone().transform(trans)
    }
}

/// A primitive layout shape consisting of a layer and a geometric shape.
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub struct Shape {
    layer: LayerId,
    shape: geometry::shape::Shape,
}

impl Shape {
    /// Creates a new layout shape.
    pub fn new(layer: impl AsRef<LayerId>, shape: impl Into<geometry::shape::Shape>) -> Self {
        Self {
            layer: *layer.as_ref(),
            shape: shape.into(),
        }
    }

    /// Returns the layer that this shape is on.
    pub fn layer(&self) -> LayerId {
        self.layer
    }

    /// Returns the geometric shape of this layout shape.
    pub fn shape(&self) -> &geometry::shape::Shape {
        &self.shape
    }
}

impl Bbox for Shape {
    fn bbox(&self) -> Option<Rect> {
        self.shape.bbox()
    }
}

impl LayerBbox for Shape {
    fn layer_bbox(&self, layer: LayerId) -> Option<Rect> {
        if self.layer == layer {
            self.bbox()
        } else {
            None
        }
    }
}

impl<T: Bbox> BoundingUnion<T> for Shape {
    type Output = Rect;

    fn bounding_union(&self, other: &T) -> Self::Output {
        self.bbox().unwrap().bounding_union(&other.bbox())
    }
}

impl HasTransformedView for Shape {
    type TransformedView = Shape;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView {
        Shape {
            layer: self.layer,
            shape: self.shape.transformed_view(trans),
        }
    }
}

impl TranslateMut for Shape {
    fn translate_mut(&mut self, p: Point) {
        self.shape.translate_mut(p)
    }
}

impl TransformMut for Shape {
    fn transform_mut(&mut self, trans: Transformation) {
        self.shape.transform_mut(trans)
    }
}

impl<PDK: Pdk> Draw<PDK> for Shape {
    fn draw(self, recv: &mut DrawReceiver<PDK>) -> Result<()> {
        recv.draw_element(self);
        Ok(())
    }
}

/// A primitive text annotation consisting of a layer, string, and location.
#[derive(Default, Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub struct Text {
    layer: LayerId,
    text: ArcStr,
    pub(crate) trans: Transformation,
}

impl Text {
    /// Creates a new layout text annotation.
    pub fn new(layer: impl AsRef<LayerId>, text: impl Into<ArcStr>, trans: Transformation) -> Self {
        Self {
            layer: *layer.as_ref(),
            text: text.into(),
            trans,
        }
    }

    /// Gets the layer that this annotation is on.
    pub fn layer(&self) -> LayerId {
        self.layer
    }

    /// Gets the text of this annotation.
    pub fn text(&self) -> &ArcStr {
        &self.text
    }
}

impl HasTransformedView for Text {
    type TransformedView = Text;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView {
        self.clone().transform(trans)
    }
}

impl TranslateMut for Text {
    fn translate_mut(&mut self, p: Point) {
        self.transform_mut(Transformation::from_offset(p))
    }
}

impl TransformMut for Text {
    fn transform_mut(&mut self, trans: Transformation) {
        self.trans = Transformation::cascade(trans, self.trans);
    }
}

impl<PDK: Pdk> Draw<PDK> for Text {
    fn draw(self, recv: &mut DrawReceiver<PDK>) -> Result<()> {
        recv.draw_element(self);
        Ok(())
    }
}

/// A primitive layout element.
#[derive(Debug, Clone, PartialEq)]
pub enum Element {
    /// A raw layout instance.
    Instance(RawInstance),
    /// A primitive layout shape.
    Shape(Shape),
    /// A primitive text annotation.
    Text(Text),
}

/// A pointer to a primitive layout element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElementRef<'a> {
    /// A raw layout instance.
    Instance(&'a RawInstance),
    /// A primitive layout shape.
    Shape(&'a Shape),
    /// A primitive text annotation.
    Text(&'a Text),
}

impl Element {
    /// Converts from `&Element` to `ElementRef`.
    ///
    /// Produces a new `ElementRef` containing a reference into
    /// the original element, but leaves the original in place.
    pub fn as_ref(&self) -> ElementRef<'_> {
        match self {
            Self::Instance(x) => ElementRef::Instance(x),
            Self::Shape(x) => ElementRef::Shape(x),
            Self::Text(x) => ElementRef::Text(x),
        }
    }

    /// If this is an `Instance` variant, returns the contained instance.
    /// Otherwise, returns [`None`].
    pub fn instance(self) -> Option<RawInstance> {
        match self {
            Self::Instance(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Shape` variant, returns the contained shape.
    /// Otherwise, returns [`None`].
    pub fn shape(self) -> Option<Shape> {
        match self {
            Self::Shape(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Text` variant, returns the contained text.
    /// Otherwise, returns [`None`].
    pub fn text(self) -> Option<Text> {
        match self {
            Self::Text(x) => Some(x),
            _ => None,
        }
    }
}

impl<'a> ElementRef<'a> {
    /// If this is an `Instance` variant, returns the contained instance.
    /// Otherwise, returns [`None`].
    pub fn instance(self) -> Option<&'a RawInstance> {
        match self {
            Self::Instance(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Shape` variant, returns the contained shape.
    /// Otherwise, returns [`None`].
    pub fn shape(self) -> Option<&'a Shape> {
        match self {
            Self::Shape(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Text` variant, returns the contained text.
    /// Otherwise, returns [`None`].
    pub fn text(self) -> Option<&'a Text> {
        match self {
            Self::Text(x) => Some(x),
            _ => None,
        }
    }
}

impl Bbox for Element {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        match self {
            Element::Instance(inst) => inst.bbox(),
            Element::Shape(shape) => shape.bbox(),
            Element::Text(_) => None,
        }
    }
}

impl LayerBbox for Element {
    fn layer_bbox(&self, layer: LayerId) -> Option<geometry::rect::Rect> {
        match self {
            Element::Instance(inst) => inst.layer_bbox(layer),
            Element::Shape(shape) => shape.layer_bbox(layer),
            Element::Text(_) => None,
        }
    }
}

impl From<RawInstance> for Element {
    fn from(value: RawInstance) -> Self {
        Self::Instance(value)
    }
}

impl From<Shape> for Element {
    fn from(value: Shape) -> Self {
        Self::Shape(value)
    }
}

impl From<Text> for Element {
    fn from(value: Text) -> Self {
        Self::Text(value)
    }
}

impl HasTransformedView for Element {
    type TransformedView = Element;

    fn transformed_view(&self, trans: Transformation) -> Self::TransformedView {
        self.clone().transform(trans)
    }
}

impl TranslateMut for Element {
    fn translate_mut(&mut self, p: Point) {
        match self {
            Element::Instance(inst) => inst.translate_mut(p),
            Element::Shape(shape) => shape.translate_mut(p),
            Element::Text(text) => text.translate_mut(p),
        }
    }
}

impl TransformMut for Element {
    fn transform_mut(&mut self, trans: Transformation) {
        match self {
            Element::Instance(inst) => inst.transform_mut(trans),
            Element::Shape(shape) => shape.transform_mut(trans),
            Element::Text(text) => text.transform_mut(trans),
        }
    }
}

impl<PDK: Pdk> Draw<PDK> for Element {
    fn draw(self, cell: &mut DrawReceiver<PDK>) -> Result<()> {
        cell.draw_element(self);
        Ok(())
    }
}

impl Bbox for ElementRef<'_> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        match self {
            ElementRef::Instance(inst) => inst.bbox(),
            ElementRef::Shape(shape) => shape.bbox(),
            ElementRef::Text(_) => None,
        }
    }
}

impl LayerBbox for ElementRef<'_> {
    fn layer_bbox(&self, layer: LayerId) -> Option<Rect> {
        match self {
            ElementRef::Instance(inst) => inst.layer_bbox(layer),
            ElementRef::Shape(shape) => shape.layer_bbox(layer),
            ElementRef::Text(_) => None,
        }
    }
}

impl<'a> From<&'a RawInstance> for ElementRef<'a> {
    fn from(value: &'a RawInstance) -> Self {
        Self::Instance(value)
    }
}

impl<'a> From<&'a Shape> for ElementRef<'a> {
    fn from(value: &'a Shape) -> Self {
        Self::Shape(value)
    }
}

impl<'a> From<&'a Text> for ElementRef<'a> {
    fn from(value: &'a Text) -> Self {
        Self::Text(value)
    }
}
