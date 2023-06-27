use std::{any::Any, sync::Arc};

use geometry::{
    prelude::Point,
    transform::{Transform, TransformMut, Transformation},
};

use crate::translate_view::{HasTransformedView, Transformed};

pub trait HasLayout {
    type Data: HasTransformedView;
}

pub struct Cell<T: HasLayout> {
    block: T,
    data: T::Data,
}
pub struct TranslatedCell<'a, T: HasLayout> {
    block: &'a T,
    data: Transformed<'a, T::Data>,
}

impl<T: HasLayout + Any> HasTransformedView for Cell<T> {
    type TransformedView<'a> = TranslatedCell<'a, T>;

    fn transformed_view<'a>(&'a self, trans: Transformation) -> Self::TransformedView<'a> {
        TranslatedCell {
            block: &self.block,
            data: self.data.transformed_view(trans),
        }
    }
}

pub struct Instance<T: HasLayout> {
    cell: Arc<Cell<T>>,
    transform: Transformation,
}

impl<T: HasLayout + Any> HasTransformedView for Instance<T> {
    type TransformedView<'a> = Instance<T>;

    fn transformed_view<'a>(&'a self, trans: Transformation) -> Self::TransformedView<'a> {
        Instance {
            cell: self.cell.clone(),
            transform: Transformation::cascade(trans, self.transform),
        }
    }
}

impl<T: HasLayout + Any> Instance<T> {
    fn enter<I: Transform>(&self, f: impl FnOnce(&Cell<T>) -> I) -> I {
        let mut ret = f(&self.cell);
        ret.transform(self.transform)
    }

    fn cell(&self) -> TranslatedCell<'_, T> {
        self.cell.as_ref().transformed_view(self.transform)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use geometry::{
        prelude::{NamedOrientation, Point},
        rect::Rect,
        transform::{Transform, TransformMut, Transformation},
    };

    use crate::translate_view::{HasTransformedView, Transformed};

    use super::{Cell, HasLayout, Instance};

    pub struct Block1;
    pub struct Block1Data {
        rect: Rect,
        rects: Vec<Rect>,
    }

    pub struct TransformedBlock1Data<'a> {
        rect: Transformed<'a, Rect>,
        rects: Transformed<'a, Vec<Rect>>,
    }
    impl HasLayout for Block1 {
        type Data = Block1Data;
    }

    impl HasTransformedView for Block1Data {
        type TransformedView<'a> = TransformedBlock1Data<'a>;

        fn transformed_view<'a>(&'a self, trans: Transformation) -> Self::TransformedView<'a> {
            TransformedBlock1Data {
                rect: self.rect.transformed_view(trans),
                rects: self.rects.transformed_view(trans),
            }
        }
    }

    pub struct Block2;
    pub struct Block2Data {
        inner: Instance<Block1>,
    }
    impl HasLayout for Block2 {
        type Data = Block2Data;
    }

    impl HasTransformedView for Block2Data {
        type TransformedView<'a> = Block2Data;

        fn transformed_view<'a>(&'a self, trans: Transformation) -> Self::TransformedView<'a> {
            Block2Data {
                inner: self.inner.transformed_view(trans),
            }
        }
    }

    pub struct Block3;
    pub struct Block3Data {
        inner: Instance<Block2>,
    }
    impl HasLayout for Block3 {
        type Data = Block3Data;
    }

    impl HasTransformedView for Block3Data {
        type TransformedView<'a> = Block3Data;

        fn transformed_view<'a>(&'a self, trans: Transformation) -> Self::TransformedView<'a> {
            Block3Data {
                inner: self.inner.transformed_view(trans),
            }
        }
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
                    rects: vec![
                        Rect::from_sides(0, 0, 10, 20),
                        Rect::from_sides(0, 0, 30, 20),
                        Rect::from_sides(0, 0, 10, 50),
                    ],
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
        rect.transform_mut(t1);
        rect.transform_mut(t2);
        rect.transform_mut(t3);
        assert_eq!(trect, rect);

        assert_eq!(
            instance3
                .cell()
                .data
                .inner
                .cell()
                .data
                .inner
                .cell()
                .data
                .rects
                .get(0)
                .unwrap(),
            Rect::from_sides(0, 0, 10, 20)
                .transform(t1)
                .transform(t2)
                .transform(t3)
        );
    }
}
