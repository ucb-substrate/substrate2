use std::{collections::HashMap, fmt::Debug};

use arcstr::ArcStr;
use geometry::{
    align::AlignRect,
    bbox::Bbox,
    point::Point,
    prelude::AlignMode,
    rect::Rect,
    transform::{Translate, TranslateOwned},
};

#[derive(Default)]
pub struct Context {
    instances: Vec<RawInstance>,
}

impl Context {
    pub fn new() -> Self {
        Self::default()
    }
}

pub struct RawInstance {
    pub block: HashMap<ArcStr, ArcStr>,
    pub bbox: Rect,
    pub loc: Point,
}

#[derive(Debug, Clone)]
pub struct Instance<T> {
    block: T,
    bbox: Rect,
    loc: Point,
}

impl<T: Block> Instance<T> {
    pub fn new(block: T, bbox: Rect) -> Self {
        Self {
            block,
            bbox,
            loc: Point::zero(),
        }
    }
}

pub trait Block: Debug + Clone {
    fn into_hashmap(self) -> HashMap<ArcStr, ArcStr>;
}

impl<T: Block> From<Instance<T>> for RawInstance {
    fn from(value: Instance<T>) -> Self {
        Self {
            block: value.block.into_hashmap(),
            bbox: value.bbox,
            loc: value.loc,
        }
    }
}

impl<T> Translate for Instance<T> {
    fn translate(&mut self, p: Point) {
        self.loc.translate(p);
    }
}

impl<T> Bbox for Instance<T> {
    fn bbox(&self) -> Option<Rect> {
        Some(self.bbox.translate_owned(self.loc))
    }
}

#[derive(Clone)]
pub struct Group<T> {
    inner: T,
}

impl<T: Translate> Translate for Group<T> {
    fn translate(&mut self, p: Point) {
        self.inner.translate(p);
    }
}

impl<T> Group<T> {
    pub fn new(inner: T) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl<T: Bbox> Bbox for Group<T> {
    fn bbox(&self) -> Option<Rect> {
        self.inner.bbox()
    }
}

impl<T: Draw> Draw for Group<T> {
    fn draw(self, ctx: &mut Context) {
        self.inner.draw(ctx);
    }
}

impl<T: DrawRef> DrawRef for Group<T> {
    fn draw_ref(&self, ctx: &mut Context) {
        self.inner.draw_ref(ctx);
    }
}

pub trait Draw {
    fn draw(self, ctx: &mut Context);
}

macro_rules! draw_tuple_impls {
    ( $( $name:ident )+ ) => {
        #[allow(non_snake_case)]
        impl<$($name: Draw),+> Draw for ($($name,)+)
        {
            fn draw(self, ctx: &mut Context) {
                let ($( $name, )+) = self;
                $($name.draw(ctx);)+
            }
        }
    };
}

draw_tuple_impls! { A }
draw_tuple_impls! { A B }
draw_tuple_impls! { A B C }
draw_tuple_impls! { A B C D }
draw_tuple_impls! { A B C D E }
draw_tuple_impls! { A B C D E F }
draw_tuple_impls! { A B C D E F G }
draw_tuple_impls! { A B C D E F G H }
draw_tuple_impls! { A B C D E F G H I }
draw_tuple_impls! { A B C D E F G H I J }
draw_tuple_impls! { A B C D E F G H I J K }
draw_tuple_impls! { A B C D E F G H I J K L }

pub trait DrawRef {
    fn draw_ref(&self, ctx: &mut Context);
}

macro_rules! draw_ref_tuple_impls {
    ( $( $name:ident )+ ) => {
        #[allow(non_snake_case)]
        impl<$($name: DrawRef),+> DrawRef for ($($name,)+)
        {
            fn draw_ref(&self, ctx: &mut Context) {
                let ($( $name, )+) = self;
                $($name.draw_ref(ctx);)+
            }
        }
    };
}

draw_ref_tuple_impls! { A }
draw_ref_tuple_impls! { A B }
draw_ref_tuple_impls! { A B C }
draw_ref_tuple_impls! { A B C D }
draw_ref_tuple_impls! { A B C D E }
draw_ref_tuple_impls! { A B C D E F }
draw_ref_tuple_impls! { A B C D E F G }
draw_ref_tuple_impls! { A B C D E F G H }
draw_ref_tuple_impls! { A B C D E F G H I }
draw_ref_tuple_impls! { A B C D E F G H I J }
draw_ref_tuple_impls! { A B C D E F G H I J K }
draw_ref_tuple_impls! { A B C D E F G H I J K L }

impl<T: Block> Draw for Instance<T> {
    fn draw(self, ctx: &mut Context) {
        self.draw_ref(ctx);
    }
}

impl<T: Block> DrawRef for Instance<T> {
    fn draw_ref(&self, ctx: &mut Context) {
        ctx.instances.push(self.clone().into());
    }
}

pub trait Tileable: Draw + AlignRect {}
impl<T: Draw + AlignRect> Tileable for T {}
pub trait RefTileable: DrawRef + AlignRect {}
impl<T: DrawRef + AlignRect> RefTileable for T {}

#[derive(Debug, Clone)]
pub struct Tile<T> {
    inner: T,
    rect: Rect,
}

impl<T: Bbox> Tile<T> {
    pub fn from_bbox(inner: T) -> Self {
        let rect = inner.bbox().unwrap();
        Self { inner, rect }
    }
}

impl<T: Translate> Translate for Tile<T> {
    fn translate(&mut self, p: Point) {
        self.inner.translate(p);
        self.rect.translate(p);
    }
}

pub enum RawTileKind<'a> {
    Ref(&'a mut dyn RefTileable),
    RefNoDraw(&'a mut dyn AlignRect),
}

pub struct RawTile<'a> {
    tile: RawTileKind<'a>,
    rect: Rect,
}

impl<'a> Translate for RawTileKind<'a> {
    fn translate(&mut self, p: Point) {
        match self {
            RawTileKind::Ref(tile) => tile.translate(p),
            RawTileKind::RefNoDraw(tile) => tile.translate(p),
        };
    }
}

impl<'a> Translate for RawTile<'a> {
    fn translate(&mut self, p: Point) {
        self.tile.translate(p);
        self.rect.translate(p);
    }
}

impl<'a> RawTile<'a> {
    pub fn from_bbox<T: Bbox + RefTileable + 'a>(inner: &'a mut T) -> Self {
        let rect = inner.bbox().unwrap();
        Self {
            tile: RawTileKind::Ref(inner),
            rect,
        }
    }
    pub fn from_bbox_no_draw<T: Bbox + AlignRect + 'a>(inner: &'a mut T) -> Self {
        let rect = inner.bbox().unwrap();
        Self {
            tile: RawTileKind::RefNoDraw(inner),
            rect,
        }
    }
}

#[derive(Default)]
pub struct ArrayTiler<'a> {
    tiles: Vec<RawTile<'a>>,
}

impl<'a> ArrayTiler<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, tile: RawTile<'a>) {
        self.tiles.push(tile);
    }

    pub fn apply(&mut self) {
        for i in 1..self.tiles.len() {
            let srect = self.tiles[i].rect;
            let orect = self.tiles[i - 1].rect;
            self.tiles[i].align(AlignMode::ToTheRight, srect, orect, 0);
            self.tiles[i].align(AlignMode::Top, srect, orect, 0);
        }
    }
}

impl<'a> Draw for ArrayTiler<'a> {
    fn draw(mut self, ctx: &mut Context) {
        self.apply();
        for tile in self.tiles.into_iter() {
            match tile.tile {
                RawTileKind::Ref(tile) => tile.draw_ref(ctx),
                RawTileKind::RefNoDraw(_tile) => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use arcstr::ArcStr;
    use geometry::{
        prelude::{AlignBbox, AlignMode, Bbox, Point},
        rect::Rect,
        transform::Translate,
    };

    use crate::tiler::{ArrayTiler, Context, Draw, Group, RawTile};

    use super::{Block, Instance};

    #[derive(Debug, Clone)]
    pub struct TestBlock {
        data: usize,
    }

    impl TestBlock {
        fn new(data: usize) -> Self {
            TestBlock { data }
        }
    }

    impl Block for TestBlock {
        fn into_hashmap(self) -> HashMap<ArcStr, ArcStr> {
            HashMap::from_iter([(arcstr::literal!("data"), arcstr::format!("{}", self.data))])
        }
    }

    #[test]
    fn test_tiler_api() {
        let mut ctx = Context::new();
        let mut instance1 = Instance::new(TestBlock::new(5), Rect::from_sides(0, 0, 100, 200));
        instance1.translate(Point::new(5, 0));
        assert_eq!(instance1.loc, Point::new(5, 0));

        let mut instance2 = instance1.clone();
        instance2.align_bbox(AlignMode::ToTheRight, &instance1, 20);
        assert_eq!(
            instance2.bbox().unwrap(),
            Rect::from_sides(125, 0, 225, 200)
        );

        assert_eq!(instance1.block.data, instance2.block.data);

        let group = Group::new((instance1, instance2));

        let mut groups = vec![group];
        for i in 1..5 {
            let mut group = groups[i - 1].clone();
            group.align_bbox(AlignMode::Beneath, &groups[i - 1], 0);
            groups.push(group);
        }

        let (instance1, instance2) = &groups[4].inner;
        assert_eq!(instance1.block.data, 5);
        assert_eq!(instance2.block.data, 5);
        assert_eq!(instance1.loc, Point::new(5, -800));
        assert_eq!(instance2.loc, Point::new(125, -800));

        let mut tiler = ArrayTiler::new();
        for group in groups.iter_mut() {
            tiler.push(RawTile::from_bbox(group));
        }
        let mut instance3 = Instance::new(TestBlock::new(6), Rect::from_sides(0, 0, 200, 100));
        tiler.push(RawTile::from_bbox_no_draw(&mut instance3));
        tiler.apply();
        tiler.draw(&mut ctx);

        assert_eq!(ctx.instances.len(), 10);
        for (i, instance) in ctx.instances.iter().enumerate() {
            assert_eq!(
                instance.block,
                HashMap::from_iter([(arcstr::literal!("data"), arcstr::literal!("5"))])
            );
            assert_eq!(instance.bbox, Rect::from_sides(0, 0, 100, 200));
            assert_eq!(
                instance.loc,
                Point::new(5 + 220 * (i as i64 / 2) + 120 * (i as i64 % 2), 0)
            );
        }

        let (instance1, instance2) = &groups[4].inner;
        assert_eq!(instance1.block.data, 5);
        assert_eq!(instance2.block.data, 5);
        assert_eq!(instance3.block.data, 6);
        assert_eq!(instance1.loc, Point::new(885, 0));
        assert_eq!(instance2.loc, Point::new(1005, 0));
        assert_eq!(instance3.loc, Point::new(1105, 100));
    }
}
