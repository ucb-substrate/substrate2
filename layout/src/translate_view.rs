use geometry::prelude::Point;

pub trait HasLayout {
    type Data: HasTranslateView;
}

pub struct TranslatedCell<'a, T: HasLayout> {
    cell: &'a Cell<T>,
    loc: Point,
}

pub struct Cell<T: HasLayout> {
    block: T,
    data: T::Data,
}

pub trait HasTransformedView {
    type TransformedView;

    fn transformed_view(&self, pt: Point) -> Self::TranslateView;
}

impl<T: HasLayout> HasTranslateView for Cell<T> {
    type TranslateView = Trans;
}
