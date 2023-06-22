use std::{any::Any, sync::Arc};

use geometry::{
    prelude::Point,
    transform::{Transform, Transformation},
};

pub trait HasLayout {
    type Data;
}

pub struct Cell<T: HasLayout> {
    block: T,
    data: T::Data,
}

pub struct Instance<T: HasLayout> {
    cell: Arc<Cell<T>>,
    transform: Transformation,
}

impl<T: HasLayout> Instance<T> {
    fn enter<I: Transform>(&self, f: impl FnOnce(&Cell<T>) -> I) -> I {
        let mut ret = f(&self.cell);
        ret.transform(self.transform);
        ret
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use geometry::{
        prelude::{NamedOrientation, Point},
        rect::Rect,
        transform::{Transform, Transformation},
    };

    use super::{Cell, HasLayout, Instance};

    pub struct Block1;
    pub struct Block1Data {
        rect: Rect,
    }
    impl HasLayout for Block1 {
        type Data = Block1Data;
    }

    pub struct Block2;
    pub struct Block2Data {
        inner: Instance<Block1>,
    }
    impl HasLayout for Block2 {
        type Data = Block2Data;
    }

    pub struct Block3;
    pub struct Block3Data {
        inner: Instance<Block2>,
    }
    impl HasLayout for Block3 {
        type Data = Block3Data;
    }

    #[test]
    fn instance_enter_works() {
        let t1 =
            Transformation::from_offset_and_orientation(Point::new(20, 50), NamedOrientation::R0);
        let t2 = Transformation::from_offset_and_orientation(
            Point::new(-100, -200),
            NamedOrientation::R90,
        );
        let t3 = Transformation::from_offset_and_orientation(
            Point::new(-50, -150),
            NamedOrientation::R270,
        );
        let instance1 = Instance {
            cell: Arc::new(Cell {
                block: Block1,
                data: Block1Data {
                    rect: Rect::from_sides(0, 0, 100, 200),
                },
            }),
            transform: t1,
        };
        let instance2 = Instance {
            cell: Arc::new(Cell {
                block: Block2,
                data: Block2Data { inner: instance1 },
            }),
            transform: t2,
        };
        let instance3 = Instance {
            cell: Arc::new(Cell {
                block: Block3,
                data: Block3Data { inner: instance2 },
            }),
            transform: t3,
        };

        let trect =
            instance3.enter(|c| c.data.inner.enter(|c| c.data.inner.enter(|c| c.data.rect)));
        let mut rect = Rect::from_sides(0, 0, 100, 200);
        rect.transform(t1);
        rect.transform(t2);
        rect.transform(t3);
        assert_eq!(trect, rect);
    }
}
