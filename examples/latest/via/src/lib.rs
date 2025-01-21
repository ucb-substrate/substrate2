use layir::Shape;
use substrate::types::codegen::PortGeometryBundle;
use substrate::types::layout::PortGeometry;
use substrate::types::ArrayBundle;
use substrate::{
    block::Block,
    geometry::rect::Rect,
    layout::{schema::Schema, Layout},
    types::{layout::PortGeometryBuilder, Array, InOut, Io, Signal},
};

#[derive(Clone, Debug, Default, Io)]
pub struct ViaIo {
    pub x: InOut<Signal>,
}

#[derive(Clone, Debug, Io)]
pub struct RectsIo {
    pub x: InOut<Array<Signal>>,
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Block)]
#[substrate(io = "ViaIo")]
pub struct Via;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum Layer {
    MetBot,
    Cut,
    MetTop,
}

impl Schema for Layer {
    type Layer = Layer;
}

impl Layout for Via {
    type Data = ();
    type Schema = Layer;
    type Bundle = ViaIoView<PortGeometryBundle<Layer>>;
    fn layout(
        &self,
        cell: &mut substrate::layout::CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        cell.draw(Shape::new(Layer::MetTop, Rect::from_sides(0, 0, 100, 100)))?;
        let cut = Shape::new(Layer::Cut, Rect::from_sides(40, 40, 60, 60));
        cell.draw(cut.clone())?;
        cell.draw(Shape::new(Layer::MetBot, Rect::from_sides(20, 20, 80, 80)))?;
        let mut x = PortGeometryBuilder::default();
        x.push(cut);
        Ok((ViaIoView { x: x.build()? }, ()))
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Rects {
    n_io: usize,
    n_drawn: usize,
}

impl Block for Rects {
    type Io = RectsIo;

    fn io(&self) -> Self::Io {
        RectsIo {
            x: InOut(Array::new(self.n_io, Signal)),
        }
    }
    fn name(&self) -> substrate::arcstr::ArcStr {
        substrate::arcstr::literal!("rects")
    }
}

impl Layout for Rects {
    type Data = ();
    type Schema = Layer;
    type Bundle = RectsIoView<PortGeometryBundle<Layer>>;
    fn layout(
        &self,
        cell: &mut substrate::layout::CellBuilder<Self::Schema>,
    ) -> substrate::error::Result<(Self::Bundle, Self::Data)> {
        let ports = (0..self.n_drawn)
            .map(|i| {
                let i = i as i64;
                let cut = Shape::new(
                    Layer::Cut,
                    Rect::from_sides(40 + 100 * i, 40, 60 + 100 * i, 60),
                );
                cell.draw(cut.clone()).expect("failed to draw geometry");
                PortGeometry::new(cut)
            })
            .collect();
        let x = ArrayBundle::new(Signal, ports);
        Ok((RectsIoView { x }, ()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use layir::Element;
    use substrate::context::Context;

    #[test]
    fn export_via_layout() {
        let ctx = Context::builder().build();
        let lib = ctx.export_layir(Via).expect("failed to export layout");
        assert_eq!(lib.layir.cells().count(), 1);
        let cell = lib.layir.cells().next().unwrap().1;
        assert_eq!(cell.name(), "via");
        assert_eq!(cell.elements().count(), 3);
        assert_eq!(cell.ports().count(), 1);
        let port = cell.ports().next().unwrap();
        let mut iter = port.1.elements();
        let x = iter.next().unwrap();
        assert_eq!(
            *x,
            Element::Shape(Shape::new(Layer::Cut, Rect::from_sides(40, 40, 60, 60)))
        );
    }

    #[test]
    fn export_rects_layout() {
        let ctx = Context::builder().build();
        let lib = ctx
            .export_layir(Rects {
                n_io: 12,
                n_drawn: 12,
            })
            .expect("failed to export layout");
        assert_eq!(lib.layir.cells().count(), 1);
        let cell = lib.layir.cells().next().unwrap().1;
        assert_eq!(cell.name(), "rects");
        assert_eq!(cell.elements().count(), 12);
        assert_eq!(cell.ports().count(), 12);
        let mut ports = cell.ports();
        let port = ports.next().unwrap();
        let mut iter = port.1.elements();
        let x = iter.next().unwrap();
        assert_eq!(
            *x,
            Element::Shape(Shape::new(Layer::Cut, Rect::from_sides(40, 40, 60, 60)))
        );
        let port = ports.next().unwrap();
        let mut iter = port.1.elements();
        let x = iter.next().unwrap();
        assert_eq!(
            *x,
            Element::Shape(Shape::new(Layer::Cut, Rect::from_sides(140, 40, 160, 60)))
        );
    }

    #[test]
    fn export_rects_layout_mismatched_io_length() {
        let ctx = Context::builder().build();
        assert!(ctx
            .export_layir(Rects {
                n_io: 12,
                n_drawn: 16,
            })
            .is_err());
    }
}
