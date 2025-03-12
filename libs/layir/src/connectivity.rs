use crate::{Cell, Shape};

trait Connectivity: Sized + PartialEq {
    fn connected_layers(&self) -> Vec<Self>;

    fn connected(&self, other: &Self) -> bool {
        self.connected_layers().contains(other)
    }

    fn connected_shapes<'a, 'b>(
        cell: &'a Cell<Self>,
        start: &'b Shape<Self>,
    ) -> Vec<&'a Shape<Self>> {
        // unimplemented!()
        let mut ret : Vec<&'a Shape<Self>>;

        for shape in cell.elements() {
            if start.connected(shape) {
                ret.push(shape);
            }
        }
        ret
    }
}

#[cfg(test)]
mod tests {
    use geometry::rect::Rect;

    use super::*;

    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    enum Layer {
        Met1,
        Via1,
        Met2,
    }

    impl Connectivity for Layer {
        fn connected_layers(&self) -> Vec<Self> {
            match self {
                Self::Met1 => vec![Self::Met1, Self::Via1],
                Self::Via1 => vec![Self::Met1, Self::Via1, Self::Met2],
                Self::Met2 => vec![Self::Via1, Self::Met2],
            }
        }
    }

    #[test]
    fn test_connectivity() {
        let mut cell = Cell::new("test");
        let m1_shape = Shape::new(Layer::Met1, Rect::from_sides(0, 0, 100, 100));
        cell.add_element(m1_shape.clone());
        assert_eq!(Layer::connected_shapes(&cell, &m1_shape), vec![&m1_shape]);
    }
}