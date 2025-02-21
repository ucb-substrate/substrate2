use std::cell::Cell;

use crate::Shape;

trait Connectivity: Sized + PartialEq {
    fn connected_layers(&self) -> Vec<Self>;

    fn connected(&self, other: &Self) -> bool {
        self.connected_layers().contains(other)
    }

    fn connected_shapes(cell: Cell<Self>, start: Shape<Self>) -> Vec<Shape<Self>> {
        unimplemented!()
    }
}
