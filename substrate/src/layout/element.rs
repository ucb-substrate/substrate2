//! Basic layout elements.
//!
//! Substrate layouts consist of cells, instances, geometric shapes, and text annotations.

use std::{collections::HashMap, sync::Arc};

use arcstr::ArcStr;
use geometry::{
    prelude::{Bbox, Point},
    rect::Rect,
    transform::{
        Transform, TransformMut, TransformRef, Transformation, Translate, TranslateMut,
        TranslateRef,
    },
};
use indexmap::IndexMap;
use layir::{LayerBbox, Shape, Text};
use serde::{Deserialize, Serialize};

use crate::types::layout::PortGeometry;
use crate::{
    error::{Error, Result},
    types::NameBuf,
};

use super::{schema::Schema, Draw, DrawReceiver, Instance, Layout};

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
pub type NamedPorts<L> = IndexMap<NameBuf, PortGeometry<L>>;

/// A raw layout cell.
#[derive(Default, Debug, Clone, PartialEq)]
pub struct RawCell<L> {
    pub(crate) id: CellId,
    pub(crate) name: ArcStr,
    pub(crate) elements: Vec<Element<L>>,
    ports: NamedPorts<L>,
    port_names: HashMap<String, NameBuf>,
}

impl<L> RawCell<L> {
    pub(crate) fn new(id: CellId, name: impl Into<ArcStr>) -> Self {
        Self {
            id,
            name: name.into(),
            elements: Vec::new(),
            ports: IndexMap::new(),
            port_names: HashMap::new(),
        }
    }

    pub(crate) fn with_ports(self, ports: NamedPorts<L>) -> Self {
        let port_names = ports.keys().map(|k| (k.to_string(), k.clone())).collect();
        Self {
            ports,
            port_names,
            ..self
        }
    }

    #[doc(hidden)]
    pub fn port_map(&self) -> &NamedPorts<L> {
        &self.ports
    }

    pub(crate) fn add_element(&mut self, elem: impl Into<Element<L>>) {
        self.elements.push(elem.into());
    }

    #[allow(dead_code)]
    pub(crate) fn add_elements(&mut self, elems: impl IntoIterator<Item = impl Into<Element<L>>>) {
        self.elements.extend(elems.into_iter().map(|x| x.into()));
    }

    /// The ID of this cell.
    pub fn id(&self) -> CellId {
        self.id
    }

    /// Returns an iterator over the elements of this cell.
    pub fn elements(&self) -> impl Iterator<Item = &Element<L>> {
        self.elements.iter()
    }

    /// Returns an iterator over the ports of this cell, as `(name, geometry)` pairs.
    pub fn ports(&self) -> impl Iterator<Item = (&NameBuf, &PortGeometry<L>)> {
        self.ports.iter()
    }

    /// Returns a reference to the port with the given name, if it exists.
    pub fn port_named(&self, name: &str) -> Option<&PortGeometry<L>> {
        let name_buf = self.port_names.get(name)?;
        self.ports.get(name_buf)
    }
}

impl<L> Bbox for RawCell<L> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        self.elements.bbox()
    }
}

impl<L: PartialEq> LayerBbox<L> for RawCell<L> {
    fn layer_bbox(&self, layer: &L) -> Option<Rect> {
        self.elements.layer_bbox(layer)
    }
}

impl<L: Clone> TranslateRef for RawCell<L> {
    fn translate_ref(&self, p: Point) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            elements: self.elements.translate_ref(p),
            ports: self
                .ports
                .iter()
                .map(|(k, v)| (k.clone(), v.translate_ref(p)))
                .collect(),
            port_names: self.port_names.clone(),
        }
    }
}

impl<L: Clone> TransformRef for RawCell<L> {
    fn transform_ref(&self, trans: Transformation) -> Self {
        Self {
            id: self.id,
            name: self.name.clone(),
            elements: self.elements.transform_ref(trans),
            ports: self
                .ports
                .iter()
                .map(|(k, v)| (k.clone(), v.transform_ref(trans)))
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
pub struct RawInstance<L> {
    pub(crate) cell: Arc<RawCell<L>>,
    pub(crate) trans: Transformation,
}

impl<L> RawInstance<L> {
    /// Create a new raw instance of the given cell.
    pub fn new(cell: impl Into<Arc<RawCell<L>>>, trans: Transformation) -> Self {
        Self {
            cell: cell.into(),
            trans,
        }
    }

    /// Returns a raw reference to the child cell.
    ///
    /// The returned cell does not store any information related
    /// to this instance's transformation.
    /// Consider using [`RawInstance::cell`] instead.
    #[inline]
    pub fn raw_cell(&self) -> &RawCell<L> {
        &self.cell
    }
}

impl<L: Clone> RawInstance<L> {
    /// Returns a reference to the child cell.
    ///
    /// The returned object provides coordinates in the parent cell's coordinate system.
    /// If you want coordinates in the child cell's coordinate system,
    /// consider using [`RawInstance::raw_cell`] instead.
    #[inline]
    pub fn cell(&self) -> RawCell<L> {
        self.cell.transform_ref(self.trans)
    }
}

impl<L> Bbox for RawInstance<L> {
    fn bbox(&self) -> Option<Rect> {
        self.cell.bbox().map(|rect| rect.transform(self.trans))
    }
}

impl<L: PartialEq> LayerBbox<L> for RawInstance<L> {
    fn layer_bbox(&self, layer: &L) -> Option<Rect> {
        self.cell
            .layer_bbox(layer)
            .map(|rect| rect.transform(self.trans))
    }
}

impl<T: Layout> TryFrom<Instance<T>> for RawInstance<<T::Schema as Schema>::Layer> {
    type Error = Error;

    fn try_from(value: Instance<T>) -> Result<Self> {
        Ok(Self {
            cell: value.try_cell()?.raw,
            trans: value.trans,
        })
    }
}

impl<S: Schema> Draw<S> for RawInstance<S::Layer> {
    fn draw(self, recv: &mut DrawReceiver<S>) -> Result<()> {
        recv.draw_element(self);
        Ok(())
    }
}

impl<L> TranslateMut for RawInstance<L> {
    fn translate_mut(&mut self, p: Point) {
        self.transform_mut(Transformation::from_offset(p));
    }
}

impl<L> TransformMut for RawInstance<L> {
    fn transform_mut(&mut self, trans: Transformation) {
        self.trans = Transformation::cascade(trans, self.trans);
    }
}

impl<L: Clone> TranslateRef for RawInstance<L> {
    fn translate_ref(&self, p: Point) -> Self {
        self.clone().translate(p)
    }
}

impl<L: Clone> TransformRef for RawInstance<L> {
    fn transform_ref(&self, trans: Transformation) -> Self {
        self.clone().transform(trans)
    }
}

impl<S: Schema> Draw<S> for Shape<S::Layer> {
    fn draw(self, recv: &mut DrawReceiver<S>) -> Result<()> {
        recv.draw_element(self);
        Ok(())
    }
}

impl<S: Schema> Draw<S> for Text<S::Layer> {
    fn draw(self, recv: &mut DrawReceiver<S>) -> Result<()> {
        recv.draw_element(self);
        Ok(())
    }
}

/// A primitive layout element.
#[derive(Debug, Clone, PartialEq)]
pub enum Element<L> {
    /// A raw layout instance.
    Instance(RawInstance<L>),
    /// A primitive layout shape.
    Shape(Shape<L>),
    /// A primitive text annotation.
    Text(Text<L>),
}

/// A pointer to a primitive layout element.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElementRef<'a, L> {
    /// A raw layout instance.
    Instance(&'a RawInstance<L>),
    /// A primitive layout shape.
    Shape(&'a Shape<L>),
    /// A primitive text annotation.
    Text(&'a Text<L>),
}

impl<L> Element<L> {
    /// Converts from `&Element` to `ElementRef`.
    ///
    /// Produces a new `ElementRef` containing a reference into
    /// the original element, but leaves the original in place.
    pub fn as_ref(&self) -> ElementRef<'_, L> {
        match self {
            Self::Instance(x) => ElementRef::Instance(x),
            Self::Shape(x) => ElementRef::Shape(x),
            Self::Text(x) => ElementRef::Text(x),
        }
    }

    /// If this is an `Instance` variant, returns the contained instance.
    /// Otherwise, returns [`None`].
    pub fn instance(self) -> Option<RawInstance<L>> {
        match self {
            Self::Instance(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Shape` variant, returns the contained shape.
    /// Otherwise, returns [`None`].
    pub fn shape(self) -> Option<Shape<L>> {
        match self {
            Self::Shape(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Text` variant, returns the contained text.
    /// Otherwise, returns [`None`].
    pub fn text(self) -> Option<Text<L>> {
        match self {
            Self::Text(x) => Some(x),
            _ => None,
        }
    }
}

impl<L> From<layir::Element<L>> for Element<L> {
    fn from(value: layir::Element<L>) -> Self {
        match value {
            layir::Element::Text(t) => Self::Text(t),
            layir::Element::Shape(s) => Self::Shape(s),
        }
    }
}

impl<'a, L> ElementRef<'a, L> {
    /// If this is an `Instance` variant, returns the contained instance.
    /// Otherwise, returns [`None`].
    pub fn instance(self) -> Option<&'a RawInstance<L>> {
        match self {
            Self::Instance(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Shape` variant, returns the contained shape.
    /// Otherwise, returns [`None`].
    pub fn shape(self) -> Option<&'a Shape<L>> {
        match self {
            Self::Shape(x) => Some(x),
            _ => None,
        }
    }

    /// If this is a `Text` variant, returns the contained text.
    /// Otherwise, returns [`None`].
    pub fn text(self) -> Option<&'a Text<L>> {
        match self {
            Self::Text(x) => Some(x),
            _ => None,
        }
    }
}

impl<L> Bbox for Element<L> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        match self {
            Element::Instance(inst) => inst.bbox(),
            Element::Shape(shape) => shape.bbox(),
            Element::Text(_) => None,
        }
    }
}

impl<L: PartialEq> LayerBbox<L> for Element<L> {
    fn layer_bbox(&self, layer: &L) -> Option<geometry::rect::Rect> {
        match self {
            Element::Instance(inst) => inst.layer_bbox(layer),
            Element::Shape(shape) => shape.layer_bbox(layer),
            Element::Text(_) => None,
        }
    }
}

impl<L> From<RawInstance<L>> for Element<L> {
    fn from(value: RawInstance<L>) -> Self {
        Self::Instance(value)
    }
}

impl<L> From<Shape<L>> for Element<L> {
    fn from(value: Shape<L>) -> Self {
        Self::Shape(value)
    }
}

impl<L> From<Text<L>> for Element<L> {
    fn from(value: Text<L>) -> Self {
        Self::Text(value)
    }
}

impl<L: Clone> TranslateRef for Element<L> {
    fn translate_ref(&self, p: Point) -> Self {
        self.clone().translate(p)
    }
}

impl<L: Clone> TransformRef for Element<L> {
    fn transform_ref(&self, trans: Transformation) -> Self {
        self.clone().transform(trans)
    }
}

impl<L> TranslateMut for Element<L> {
    fn translate_mut(&mut self, p: Point) {
        match self {
            Element::Instance(inst) => inst.translate_mut(p),
            Element::Shape(shape) => shape.translate_mut(p),
            Element::Text(text) => text.translate_mut(p),
        }
    }
}

impl<L> TransformMut for Element<L> {
    fn transform_mut(&mut self, trans: Transformation) {
        match self {
            Element::Instance(inst) => inst.transform_mut(trans),
            Element::Shape(shape) => shape.transform_mut(trans),
            Element::Text(text) => text.transform_mut(trans),
        }
    }
}

impl<S: Schema> Draw<S> for Element<S::Layer> {
    fn draw(self, cell: &mut DrawReceiver<S>) -> Result<()> {
        cell.draw_element(self);
        Ok(())
    }
}

impl<L> Bbox for ElementRef<'_, L> {
    fn bbox(&self) -> Option<geometry::rect::Rect> {
        match self {
            ElementRef::Instance(inst) => inst.bbox(),
            ElementRef::Shape(shape) => shape.bbox(),
            ElementRef::Text(_) => None,
        }
    }
}

impl<L: PartialEq> LayerBbox<L> for ElementRef<'_, L> {
    fn layer_bbox(&self, layer: &L) -> Option<Rect> {
        match self {
            ElementRef::Instance(inst) => inst.layer_bbox(layer),
            ElementRef::Shape(shape) => shape.layer_bbox(layer),
            ElementRef::Text(_) => None,
        }
    }
}

impl<'a, L> From<&'a RawInstance<L>> for ElementRef<'a, L> {
    fn from(value: &'a RawInstance<L>) -> Self {
        Self::Instance(value)
    }
}

impl<'a, L> From<&'a Shape<L>> for ElementRef<'a, L> {
    fn from(value: &'a Shape<L>) -> Self {
        Self::Shape(value)
    }
}

impl<'a, L> From<&'a Text<L>> for ElementRef<'a, L> {
    fn from(value: &'a Text<L>) -> Self {
        Self::Text(value)
    }
}
