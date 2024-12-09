//! Built-in implementations of IO traits.

use layout::{HasLayoutBundleKind, LayoutBundle, LayoutBundleBuilder, LayoutBundleKind};
use schematic::{Node, SchematicBundleKind, Terminal};

use geometry::point::Point;
use geometry::transform::{TransformRef, TranslateRef};

use crate::layout::schema::Schema;
use crate::types::layout::{HierarchicalBuildFrom, PortGeometry, PortGeometryBuilder};
use std::fmt::Display;
use std::ops::IndexMut;
use std::{ops::DerefMut, slice::SliceIndex};

use crate::schematic::HasNestedView;

use super::*;

impl FlatLen for () {
    fn len(&self) -> usize {
        0
    }
}

impl Flatten<Direction> for () {
    fn flatten<E>(&self, _output: &mut E)
    where
        E: Extend<Direction>,
    {
    }
}

impl Flatten<Node> for () {
    fn flatten<E>(&self, _output: &mut E)
    where
        E: Extend<Node>,
    {
    }
}

impl Flatten<Terminal> for () {
    fn flatten<E>(&self, _output: &mut E)
    where
        E: Extend<Terminal>,
    {
    }
}

impl HasNameTree for () {
    fn names(&self) -> Option<Vec<NameTree>> {
        None
    }
}

impl HasBundleKind for () {
    type BundleKind = ();

    fn kind(&self) -> Self::BundleKind {}
}

impl SchematicBundleKind for () {
    type NodeBundle = ();
    type TerminalBundle = ();
    fn instantiate_nodes<'n>(
        &self,
        ids: &'n [Node],
    ) -> (<Self as SchematicBundleKind>::NodeBundle, &'n [Node]) {
        ((), ids)
    }
    fn instantiate_terminals<'n>(
        &self,
        ids: &'n [Terminal],
    ) -> (
        <Self as SchematicBundleKind>::TerminalBundle,
        &'n [Terminal],
    ) {
        ((), ids)
    }
    fn terminal_view(
        _cell: CellId,
        _cell_io: &<Self as SchematicBundleKind>::NodeBundle,
        _instance: InstanceId,
        _instance_io: &<Self as SchematicBundleKind>::NodeBundle,
    ) -> <Self as SchematicBundleKind>::TerminalBundle {
    }
}

impl<S: Schema> HasLayoutBundleKind<S> for () {
    type BundleKind = ();

    fn builder(
        &self,
    ) -> <<Self as HasLayoutBundleKind<S>>::BundleKind as LayoutBundleKind<S>>::Builder {
    }
}

impl<S: Schema> LayoutBundleKind<S> for () {
    type Bundle = ();
    type Builder = ();
    fn builder(&self) -> Self::Builder {}
}

impl<S: Schema> LayoutBundleBuilder<S, ()> for () {
    fn build(self) -> Result<()> {
        Ok(())
    }
}

impl<L> Flatten<PortGeometry<L>> for () {
    fn flatten<E>(&self, _output: &mut E)
    where
        E: Extend<PortGeometry<L>>,
    {
    }
}

impl FlatLen for Signal {
    fn len(&self) -> usize {
        1
    }
}

impl<S: Schema> LayoutBundleKind<S> for Signal {
    type Bundle = PortGeometry<S::Layer>;
    type Builder = PortGeometryBuilder<S::Layer>;

    fn builder(&self) -> Self::Builder {
        PortGeometryBuilder::default()
    }
}

impl HasNameTree for Signal {
    fn names(&self) -> Option<Vec<NameTree>> {
        Some(vec![])
    }
}

impl HasBundleKind for Signal {
    type BundleKind = Signal;

    fn kind(&self) -> Self::BundleKind {
        Signal
    }
}

impl SchematicBundleKind for Signal {
    type NodeBundle = Node;
    type TerminalBundle = Terminal;
    fn instantiate_nodes<'n>(
        &self,
        ids: &'n [Node],
    ) -> (<Self as SchematicBundleKind>::NodeBundle, &'n [Node]) {
        if let [id, rest @ ..] = ids {
            (*id, rest)
        } else {
            unreachable!();
        }
    }
    fn instantiate_terminals<'n>(
        &self,
        ids: &'n [Terminal],
    ) -> (
        <Self as SchematicBundleKind>::TerminalBundle,
        &'n [Terminal],
    ) {
        if let [id, rest @ ..] = ids {
            (*id, rest)
        } else {
            unreachable!();
        }
    }
    fn terminal_view(
        cell: CellId,
        cell_io: &<Self as SchematicBundleKind>::NodeBundle,
        instance: InstanceId,
        instance_io: &<Self as SchematicBundleKind>::NodeBundle,
    ) -> <Self as SchematicBundleKind>::TerminalBundle {
        Terminal {
            cell_id: cell,
            cell_node: *cell_io,
            instance_id: instance,
            instance_node: *instance_io,
        }
    }
}

impl<S: Schema> HasLayoutBundleKind<S> for Signal {
    type BundleKind = Signal;

    fn builder(
        &self,
    ) -> <<Self as HasLayoutBundleKind<S>>::BundleKind as LayoutBundleKind<S>>::Builder {
        PortGeometryBuilder::default()
    }
}

macro_rules! impl_direction {
    ($dir:ident, $flatten_dir_bound:path, $flatten_dir_body:item) => {
        impl<T> AsRef<T> for $dir<T> {
            fn as_ref(&self) -> &T {
                &self.0
            }
        }

        impl<T> Deref for $dir<T> {
            type Target = T;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl<T> DerefMut for $dir<T> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl<T> From<T> for $dir<T> {
            fn from(value: T) -> Self {
                $dir(value)
            }
        }

        impl<T> Borrow<T> for $dir<T> {
            fn borrow(&self) -> &T {
                &self.0
            }
        }

        impl<T: FlatLen> FlatLen for $dir<T> {
            #[inline]
            fn len(&self) -> usize {
                self.0.len()
            }
        }

        impl<T: $flatten_dir_bound> Flatten<Direction> for $dir<T> {
            $flatten_dir_body
        }

        impl<T: HasBundleKind> HasBundleKind for $dir<T> {
            type BundleKind = T::BundleKind;

            fn kind(&self) -> Self::BundleKind {
                self.0.kind()
            }
        }

        impl<T: HasNameTree> HasNameTree for $dir<T> {
            fn names(&self) -> Option<Vec<NameTree>> {
                self.0.names()
            }
        }

        impl<S: Schema, T> HasLayoutBundleKind<S> for $dir<T>
        where
            T: HasLayoutBundleKind<S>,
        {
            type BundleKind = <T as HasLayoutBundleKind<S>>::BundleKind;

            fn builder(&self) -> <<Self as HasLayoutBundleKind<S>>::BundleKind as LayoutBundleKind<S>>::Builder {
                self.0.builder()
            }
        }


        impl<S: Schema, T> LayoutBundleKind<S> for $dir<T>
        where
            T: LayoutBundleKind<S>,
        {
            type Bundle = T::Bundle;
            type Builder = T::Builder;

            fn builder(&self) -> Self::Builder {
                self.0.builder()
            }
        }

        // impl<T, U: CustomBundleKind<T>> CustomBundleKind<$dir<T>> for U
        // where
        //     T: LayoutBundleKind,
        // {
        //     fn from_layout_type(other: &$dir<T>) -> Self {
        //         <U as CustomBundleKind<T>>::from_layout_type(&other.0)
        //     }
        // }
    };
}

impl_direction!(
    Input,
    FlatLen,
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        output.extend(std::iter::repeat(Direction::Input).take(self.0.len()))
    }
);
impl_direction!(
    Output,
    FlatLen,
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        output.extend(std::iter::repeat(Direction::Input).take(self.0.len()))
    }
);
impl_direction!(
    InOut,
    FlatLen,
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        output.extend(std::iter::repeat(Direction::Input).take(self.0.len()))
    }
);
impl_direction!(
    Flipped,
    Flatten<Direction>,
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        let inner = self.0.flatten_vec();
        output.extend(inner.into_iter().map(|d| d.flip()))
    }
);

impl<T: FlatLen> FlatLen for Array<T> {
    fn len(&self) -> usize {
        self.kind.len() * self.len
    }
}

impl<S: Schema, T: LayoutBundleKind<S>> LayoutBundleKind<S> for Array<T> {
    type Bundle = ArrayBundle<T::Bundle>;
    type Builder = ArrayBundle<T::Builder>;

    fn builder(&self) -> Self::Builder {
        ArrayBundle {
            elems: (0..self.len).map(|_| self.kind.builder()).collect(),
            kind: self.kind,
        }
    }
}

// impl<S: Schema, T: LayoutBundleKind<S>, U: CustomBundleKind<S, T>> CustomBundleKind<S, Array<T>>
//     for Array<U>
// {
//     fn from_layout_type(other: &Array<T>) -> Self {
//         Self {
//             ty: <U as CustomBundleKind<S, T>>::from_layout_type(&other.ty),
//             len: other.len,
//         }
//     }
// }
impl<T: Flatten<Direction>> Flatten<Direction> for Array<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        for _ in 0..self.len {
            self.kind.flatten(output);
        }
    }
}

impl<T: HasNameTree> HasNameTree for Array<T> {
    fn names(&self) -> Option<Vec<NameTree>> {
        if self.len == 0 {
            return None;
        }
        let inner = self.kind.names()?;
        Some(
            (0..self.len)
                .map(|i| NameTree {
                    fragment: Some(NameFragment::Idx(i)),
                    children: inner.clone(),
                })
                .collect(),
        )
    }
}

impl<T: SchematicBundleKind> SchematicBundleKind for Array<T> {
    type NodeBundle = ArrayBundle<T::NodeBundle>;
    type TerminalBundle = ArrayBundle<T::TerminalBundle>;
    fn instantiate_nodes<'n>(
        &self,
        mut ids: &'n [Node],
    ) -> (<Self as SchematicBundleKind>::NodeBundle, &'n [Node]) {
        let elems = (0..self.len)
            .scan(&mut ids, |ids, _| {
                let (elem, new_ids) = self.kind.instantiate_nodes(ids);
                **ids = new_ids;
                Some(elem)
            })
            .collect();
        (
            ArrayBundle {
                elems,
                kind: self.kind.clone(),
            },
            ids,
        )
    }
    fn instantiate_terminals<'n>(
        &self,
        mut ids: &'n [Terminal],
    ) -> (
        <Self as SchematicBundleKind>::TerminalBundle,
        &'n [Terminal],
    ) {
        let elems = (0..self.len)
            .scan(&mut ids, |ids, _| {
                let (elem, new_ids) = self.kind.instantiate_terminals(ids);
                **ids = new_ids;
                Some(elem)
            })
            .collect();
        (
            ArrayBundle {
                elems,
                kind: self.kind.clone(),
            },
            ids,
        )
    }
    fn terminal_view(
        cell: CellId,
        cell_io: &<Self as SchematicBundleKind>::NodeBundle,
        instance: InstanceId,
        instance_io: &<Self as SchematicBundleKind>::NodeBundle,
    ) -> <Self as SchematicBundleKind>::TerminalBundle {
        ArrayBundle {
            elems: cell_io
                .elems
                .iter()
                .zip(instance_io.elems.iter())
                .map(|(cell_elem, instance_elem)| {
                    <T as SchematicBundleKind>::terminal_view(
                        cell,
                        cell_elem,
                        instance,
                        instance_elem,
                    )
                })
                .collect(),
            kind: cell_io.kind.clone(),
        }
    }
}

impl<T: HasBundleKind> HasBundleKind for Array<T> {
    type BundleKind = Array<<T as HasBundleKind>::BundleKind>;

    fn kind(&self) -> Self::BundleKind {
        Array::new(self.len, self.kind.kind())
    }
}

impl<S: Schema, T: HasLayoutBundleKind<S>> HasLayoutBundleKind<S> for Array<T> {
    type BundleKind = Array<<T as HasLayoutBundleKind<S>>::BundleKind>;

    fn builder(
        &self,
    ) -> <<Self as HasLayoutBundleKind<S>>::BundleKind as LayoutBundleKind<S>>::Builder {
        ArrayBundle {
            elems: (0..self.len).map(|_| self.kind.builder()).collect(),
            kind: self.kind.builder().kind(),
        }
    }
}

// impl<T: LayoutBundleKind, U: CustomBundleKind<T>> CustomBundleKind<Array<T>> for Array<U> {
//     fn from_layout_type(other: &Array<T>) -> Self {
//         Self {
//             kind: <U as CustomBundleKind<T>>::from_layout_type(&other.kind),
//             len: other.len,
//         }
//     }
// }

impl<T: HasBundleKind> HasBundleKind for ArrayBundle<T> {
    type BundleKind = Array<<T as HasBundleKind>::BundleKind>;

    fn kind(&self) -> Self::BundleKind {
        Array::new(self.elems.len(), self.kind.clone())
    }
}

impl<T: HasBundleKind + FlatLen> FlatLen for ArrayBundle<T> {
    fn len(&self) -> usize {
        self.elems.len() * self.kind.flat_names(None).len()
    }
}

impl<S, T: HasBundleKind + Flatten<S>> Flatten<S> for ArrayBundle<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<S>,
    {
        self.elems.iter().for_each(|e| e.flatten(output));
    }
}

impl<
        T: HasBundleKind
            + HasNestedView<NestedView: HasBundleKind<BundleKind = <T as HasBundleKind>::BundleKind>>,
    > HasNestedView for ArrayBundle<T>
{
    type NestedView = ArrayBundle<<T as HasNestedView>::NestedView>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        ArrayBundle {
            elems: self
                .elems
                .iter()
                .map(|elem| elem.nested_view(parent))
                .collect(),
            kind: self.kind.clone(),
        }
    }
}

impl<T, S> HierarchicalBuildFrom<S> for ArrayBundle<T>
where
    T: HasBundleKind + HierarchicalBuildFrom<S>,
{
    fn build_from(&mut self, path: &mut NameBuf, source: &S) {
        for (i, elem) in self.elems.iter_mut().enumerate() {
            path.push(i);
            HierarchicalBuildFrom::<S>::build_from(elem, path, source);
            path.pop();
        }
    }
}

impl<T: HasBundleKind + TranslateRef> TranslateRef for ArrayBundle<T> {
    fn translate_ref(&self, p: Point) -> Self {
        Self {
            elems: self
                .elems
                .iter()
                .map(|elem| elem.translate_ref(p))
                .collect(),
            kind: self.kind.clone(),
        }
    }
}

impl<T: HasBundleKind + TransformRef> TransformRef for ArrayBundle<T> {
    fn transform_ref(&self, trans: geometry::prelude::Transformation) -> Self {
        Self {
            elems: self
                .elems
                .iter()
                .map(|elem| elem.transform_ref(trans))
                .collect(),
            kind: self.kind.clone(),
        }
    }
}

impl<S: Schema, T: LayoutBundle<S>, B: LayoutBundle<S> + LayoutBundleBuilder<S, T>>
    LayoutBundleBuilder<S, ArrayBundle<T>> for ArrayBundle<B>
{
    fn build(self) -> Result<ArrayBundle<T>> {
        let mut elems = Vec::new();
        for e in self.elems {
            elems.push(e.build()?);
        }
        Ok(ArrayBundle {
            elems,
            kind: self.kind.clone(),
        })
    }
}

impl<T: HasBundleKind, I> Index<I> for ArrayBundle<T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;
    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.elems, index)
    }
}

impl<T: HasBundleKind, I> IndexMut<I> for ArrayBundle<T>
where
    I: SliceIndex<[T]>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.elems, index)
    }
}

impl From<ArcStr> for NameFragment {
    fn from(value: ArcStr) -> Self {
        Self::Str(value)
    }
}

impl From<&ArcStr> for NameFragment {
    fn from(value: &ArcStr) -> Self {
        Self::Str(value.clone())
    }
}

impl From<&str> for NameFragment {
    fn from(value: &str) -> Self {
        Self::Str(ArcStr::from(value))
    }
}

impl From<usize> for NameFragment {
    fn from(value: usize) -> Self {
        Self::Idx(value)
    }
}

impl From<ArcStr> for NameBuf {
    fn from(value: ArcStr) -> Self {
        Self {
            fragments: vec![NameFragment::from(value)],
        }
    }
}

impl From<&ArcStr> for NameBuf {
    fn from(value: &ArcStr) -> Self {
        Self {
            fragments: vec![NameFragment::from(value)],
        }
    }
}

impl From<&str> for NameBuf {
    fn from(value: &str) -> Self {
        Self {
            fragments: vec![NameFragment::from(value)],
        }
    }
}

impl From<usize> for NameBuf {
    fn from(value: usize) -> Self {
        Self {
            fragments: vec![NameFragment::from(value)],
        }
    }
}

impl Display for NameFragment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Str(s) => write!(f, "{s}"),
            Self::Idx(i) => write!(f, "{i}"),
        }
    }
}

impl Display for NameBuf {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(fragment) = self.fragments.first() {
            write!(f, "{fragment}")?;
        }
        for fragment in self.fragments.iter().skip(1) {
            write!(f, "_{fragment}")?;
        }
        Ok(())
    }
}

impl NameTree {
    /// Create a new name tree rooted at the given name fragment.
    pub fn new(fragment: impl Into<NameFragment>, children: Vec<NameTree>) -> Self {
        Self {
            fragment: Some(fragment.into()),
            children,
        }
    }

    /// Create a new name tree rooted at the given **optional** name fragment.
    pub fn with_optional_fragment(
        fragment: Option<impl Into<NameFragment>>,
        children: Vec<NameTree>,
    ) -> Self {
        Self {
            fragment: fragment.map(|f| f.into()),
            children,
        }
    }

    /// Create a new name tree rooted at the given **empty** name fragment.
    pub fn with_empty_fragment(children: Vec<NameTree>) -> Self {
        Self {
            fragment: None,
            children,
        }
    }

    /// Flattens the node name tree, returning a list of [`NameBuf`]s.
    pub fn flatten(&self) -> Vec<NameBuf> {
        self.flatten_inner(NameBuf::new())
    }

    fn flatten_inner(&self, mut parent: NameBuf) -> Vec<NameBuf> {
        if let Some(fragment) = self.fragment.clone() {
            parent.fragments.push(fragment);
        }
        if self.children.is_empty() {
            return vec![parent];
        }
        self.children
            .iter()
            .flat_map(|c| c.flatten_inner(parent.clone()))
            .collect()
    }
}

impl FlatLen for NameTree {
    fn len(&self) -> usize {
        // Leaf nodes have a flattened length of 1.
        if self.children.is_empty() {
            return 1;
        }

        self.children.iter().map(|c| c.len()).sum()
    }
}

impl Flatten<NameBuf> for NameTree {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<NameBuf>,
    {
        output.extend(self.flatten());
    }
    fn flatten_vec(&self) -> Vec<NameBuf> {
        self.flatten()
    }
}

impl NameBuf {
    /// Creates a new, empty [`NameBuf`].
    #[inline]
    pub fn new() -> Self {
        Default::default()
    }

    /// Adds a new fragment to the end of this name buffer.
    #[inline]
    pub fn push(&mut self, fragment: impl Into<NameFragment>) {
        self.fragments.push(fragment.into());
    }

    /// Pops and returns the last fragment off of the end of this name buffer.
    ///
    /// If the name buffer is empty, returns [`None`].
    #[inline]
    pub fn pop(&mut self) -> Option<NameFragment> {
        self.fragments.pop()
    }
}

impl<T: HasBundleKind> ArrayBundle<T> {
    /// The number of elements (of type T) in the array.
    ///
    /// Note that this may not be the same as the flattened length of the array.
    /// An array with 10 elements has `num_elems = 10`, but if each element
    /// internally contains 2 items, the flattened length of the array is 20.
    pub fn num_elems(&self) -> usize {
        self.elems.len()
    }
}

#[cfg(test)]
mod tests {
    use crate::types::*;

    #[test]
    fn flatten_name_tree() {
        let tree = NameTree::new(
            "io",
            vec![
                NameTree::new(
                    "pwr",
                    vec![NameTree::new("vdd", vec![]), NameTree::new("vss", vec![])],
                ),
                NameTree::new("out", vec![]),
            ],
        );

        assert_eq!(
            tree.flatten(),
            vec![
                NameBuf {
                    fragments: vec![
                        NameFragment::from("io"),
                        NameFragment::from("pwr"),
                        NameFragment::from("vdd")
                    ]
                },
                NameBuf {
                    fragments: vec![
                        NameFragment::from("io"),
                        NameFragment::from("pwr"),
                        NameFragment::from("vss")
                    ]
                },
                NameBuf {
                    fragments: vec![NameFragment::from("io"), NameFragment::from("out")]
                },
            ]
        );
        assert_eq!(tree.len(), 3);
    }

    #[test]
    fn flatten_name_tree_with_empty_root() {
        let tree = NameTree::with_empty_fragment(vec![
            NameTree::new(
                "pwr",
                vec![NameTree::new("vdd", vec![]), NameTree::new("vss", vec![])],
            ),
            NameTree::new("out", vec![]),
        ]);

        assert_eq!(
            tree.flatten(),
            vec![
                NameBuf {
                    fragments: vec![NameFragment::from("pwr"), NameFragment::from("vdd")]
                },
                NameBuf {
                    fragments: vec![NameFragment::from("pwr"), NameFragment::from("vss")]
                },
                NameBuf {
                    fragments: vec![NameFragment::from("out")]
                },
            ]
        );
        assert_eq!(tree.len(), 3);
    }
}
