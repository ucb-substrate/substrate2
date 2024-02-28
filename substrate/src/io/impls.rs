//! Built-in implementations of IO traits.

use crate::io::layout::{
    BundleBuilder, CustomHardwareType, HierarchicalBuildFrom, PortGeometryBuilder,
};
use crate::io::schematic::{Connect, HasTerminalView};
use std::fmt::Display;
use std::ops::IndexMut;
use std::{ops::DerefMut, slice::SliceIndex};

use crate::schematic::HasNestedView;

use super::*;

impl<T> FlatLen for &T
where
    T: FlatLen,
{
    fn len(&self) -> usize {
        (*self).len()
    }
}

impl<T> Flatten<Node> for &T
where
    T: Flatten<Node>,
{
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        (*self).flatten(output)
    }
}

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

impl schematic::HardwareType for () {
    type Bundle = ();
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Bundle, &'n [Node]) {
        ((), ids)
    }
}

impl HasNameTree for () {
    fn names(&self) -> Option<Vec<NameTree>> {
        None
    }
}

impl Flatten<Node> for () {
    fn flatten<E>(&self, _output: &mut E)
    where
        E: Extend<Node>,
    {
    }
}

impl layout::HardwareType for () {
    type Bundle = ();
    type Builder = ();

    fn builder(&self) {}
}

impl BundleBuilder<()> for () {
    fn build(self) -> Result<()> {
        Ok(())
    }
}

impl Flatten<PortGeometry> for () {
    fn flatten<E>(&self, _output: &mut E)
    where
        E: Extend<PortGeometry>,
    {
    }
}

impl FlatLen for Signal {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<Direction> for Signal {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        output.extend(std::iter::once(Direction::InOut));
    }
}

impl schematic::HardwareType for Signal {
    type Bundle = Node;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Bundle, &'n [Node]) {
        if let [id, rest @ ..] = ids {
            (*id, rest)
        } else {
            unreachable!();
        }
    }
}

impl layout::HardwareType for Signal {
    type Bundle = PortGeometry;
    type Builder = PortGeometryBuilder;

    fn builder(&self) -> Self::Builder {
        PortGeometryBuilder::default()
    }
}

impl HasNameTree for Signal {
    fn names(&self) -> Option<Vec<NameTree>> {
        Some(vec![])
    }
}

// FIXME: macro-ify START

impl<T> AsRef<T> for Input<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Input<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Input<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Input<T> {
    fn from(value: T) -> Self {
        Input(value)
    }
}

impl<T> Borrow<T> for Input<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> schematic::HardwareType for Input<T>
where
    T: schematic::HardwareType,
{
    type Bundle = T::Bundle;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Bundle, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (data, ids)
    }
}

impl<T> layout::HardwareType for Input<T>
where
    T: layout::HardwareType,
{
    type Bundle = T::Bundle;
    type Builder = T::Builder;

    fn builder(&self) -> Self::Builder {
        self.0.builder()
    }
}

impl<T, U: CustomHardwareType<T>> CustomHardwareType<Input<T>> for U
where
    T: layout::HardwareType,
{
    fn from_layout_type(other: &Input<T>) -> Self {
        <U as CustomHardwareType<T>>::from_layout_type(&other.0)
    }
}

impl<T: HasNameTree> HasNameTree for Input<T> {
    fn names(&self) -> Option<Vec<NameTree>> {
        self.0.names()
    }
}

impl<T: FlatLen> FlatLen for Input<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: FlatLen> Flatten<Direction> for Input<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        output.extend(std::iter::repeat(Direction::Input).take(self.0.len()))
    }
}
impl<T: Flatten<Node>> Flatten<Node> for Input<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        self.0.flatten(output);
    }
}

impl<T: HasNestedView> HasNestedView for Input<T> {
    type NestedView = T::NestedView;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        self.0.nested_view(parent)
    }
}

impl<T: HasTerminalView> HasTerminalView for Input<T> {
    type TerminalView = T::TerminalView;

    fn terminal_view(
        cell: CellId,
        cell_io: &Self,
        instance: InstanceId,
        instance_io: &Self,
    ) -> Self::TerminalView {
        HasTerminalView::terminal_view(cell, &cell_io.0, instance, &instance_io.0)
    }
}

impl<T> schematic::HardwareType for Output<T>
where
    T: schematic::HardwareType,
{
    type Bundle = T::Bundle;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Bundle, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (data, ids)
    }
}

impl<T> layout::HardwareType for Output<T>
where
    T: layout::HardwareType,
{
    type Bundle = T::Bundle;
    type Builder = T::Builder;

    fn builder(&self) -> Self::Builder {
        self.0.builder()
    }
}

impl<T, U: CustomHardwareType<T>> CustomHardwareType<Output<T>> for U
where
    T: layout::HardwareType,
{
    fn from_layout_type(other: &Output<T>) -> Self {
        <U as CustomHardwareType<T>>::from_layout_type(&other.0)
    }
}

impl<T: HasNameTree> HasNameTree for Output<T> {
    fn names(&self) -> Option<Vec<NameTree>> {
        self.0.names()
    }
}

impl<T: FlatLen> FlatLen for Output<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: FlatLen> Flatten<Direction> for Output<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        output.extend(std::iter::repeat(Direction::Output).take(self.0.len()))
    }
}

impl<T: Flatten<Node>> Flatten<Node> for Output<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        self.0.flatten(output);
    }
}

impl<T: HasNestedView> HasNestedView for Output<T> {
    type NestedView = T::NestedView;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        self.0.nested_view(parent)
    }
}

impl<T: HasTerminalView> HasTerminalView for Output<T> {
    type TerminalView = T::TerminalView;

    fn terminal_view(
        cell: CellId,
        cell_io: &Self,
        instance: InstanceId,
        instance_io: &Self,
    ) -> Self::TerminalView {
        HasTerminalView::terminal_view(cell, &cell_io.0, instance, &instance_io.0)
    }
}

impl<T> AsRef<T> for Output<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Deref for Output<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Output<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Output<T> {
    fn from(value: T) -> Self {
        Output(value)
    }
}

impl<T> Borrow<T> for Output<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> schematic::HardwareType for InOut<T>
where
    T: schematic::HardwareType,
{
    type Bundle = T::Bundle;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Bundle, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (data, ids)
    }
}

impl<T> layout::HardwareType for InOut<T>
where
    T: layout::HardwareType,
{
    type Bundle = T::Bundle;
    type Builder = T::Builder;

    fn builder(&self) -> Self::Builder {
        self.0.builder()
    }
}

impl<T, U: CustomHardwareType<T>> CustomHardwareType<InOut<T>> for U
where
    T: layout::HardwareType,
{
    fn from_layout_type(other: &InOut<T>) -> Self {
        <U as CustomHardwareType<T>>::from_layout_type(&other.0)
    }
}

impl<T: HasNameTree> HasNameTree for InOut<T> {
    fn names(&self) -> Option<Vec<NameTree>> {
        self.0.names()
    }
}

impl<T: FlatLen> FlatLen for InOut<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T: FlatLen> Flatten<Direction> for InOut<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        output.extend(std::iter::repeat(Direction::InOut).take(self.0.len()))
    }
}
impl<T: Flatten<Node>> Flatten<Node> for InOut<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        self.0.flatten(output);
    }
}

impl<T: HasNestedView> HasNestedView for InOut<T> {
    type NestedView = T::NestedView;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        self.0.nested_view(parent)
    }
}

impl<T: HasTerminalView> HasTerminalView for InOut<T> {
    type TerminalView = T::TerminalView;

    fn terminal_view(
        cell: CellId,
        cell_io: &Self,
        instance: InstanceId,
        instance_io: &Self,
    ) -> Self::TerminalView {
        HasTerminalView::terminal_view(cell, &cell_io.0, instance, &instance_io.0)
    }
}

impl<T> AsRef<T> for InOut<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T> Deref for InOut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for InOut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for InOut<T> {
    fn from(value: T) -> Self {
        InOut(value)
    }
}

impl<T> Borrow<T> for InOut<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> schematic::HardwareType for Flipped<T>
where
    T: schematic::HardwareType,
{
    type Bundle = T::Bundle;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Bundle, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (data, ids)
    }
}

impl<T> layout::HardwareType for Flipped<T>
where
    T: layout::HardwareType,
{
    type Bundle = T::Bundle;
    type Builder = T::Builder;

    fn builder(&self) -> Self::Builder {
        self.0.builder()
    }
}

impl<T, U: CustomHardwareType<T>> CustomHardwareType<Flipped<T>> for U
where
    T: layout::HardwareType,
{
    fn from_layout_type(other: &Flipped<T>) -> Self {
        <U as CustomHardwareType<T>>::from_layout_type(&other.0)
    }
}

impl<T: HasNameTree> HasNameTree for Flipped<T> {
    fn names(&self) -> Option<Vec<NameTree>> {
        self.0.names()
    }
}

impl<T: FlatLen> FlatLen for Flipped<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T: Flatten<Direction>> Flatten<Direction> for Flipped<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        let inner = self.0.flatten_vec();
        output.extend(inner.into_iter().map(|d| d.flip()))
    }
}
impl<T: Flatten<Node>> Flatten<Node> for Flipped<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        self.0.flatten(output);
    }
}

impl<T: HasNestedView> HasNestedView for Flipped<T> {
    type NestedView = T::NestedView;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        self.0.nested_view(parent)
    }
}

impl<T> AsRef<T> for Flipped<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T> Deref for Flipped<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Flipped<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<T> for Flipped<T> {
    fn from(value: T) -> Self {
        Flipped(value)
    }
}

impl<T> Borrow<T> for Flipped<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

// FIXME: macro-ify END

impl<T: FlatLen> FlatLen for Array<T> {
    fn len(&self) -> usize {
        self.ty.len() * self.len
    }
}

impl<T: schematic::HardwareType> schematic::HardwareType for Array<T> {
    type Bundle = ArrayData<T::Bundle>;

    fn instantiate<'n>(&self, mut ids: &'n [Node]) -> (Self::Bundle, &'n [Node]) {
        let elems = (0..self.len)
            .scan(&mut ids, |ids, _| {
                let (elem, new_ids) = self.ty.instantiate(ids);
                **ids = new_ids;
                Some(elem)
            })
            .collect();
        (
            ArrayData {
                elems,
                ty_len: self.ty.len(),
            },
            ids,
        )
    }
}

impl<T: layout::HardwareType> layout::HardwareType for Array<T> {
    type Bundle = ArrayData<T::Bundle>;
    type Builder = ArrayData<T::Builder>;

    fn builder(&self) -> Self::Builder {
        Self::Builder {
            elems: (0..self.len).map(|_| self.ty.builder()).collect(),
            ty_len: self.ty.len(),
        }
    }
}

impl<T: layout::HardwareType, U: CustomHardwareType<T>> CustomHardwareType<Array<T>> for Array<U> {
    fn from_layout_type(other: &Array<T>) -> Self {
        Self {
            ty: <U as CustomHardwareType<T>>::from_layout_type(&other.ty),
            len: other.len,
        }
    }
}

impl<T: HasNameTree> HasNameTree for Array<T> {
    fn names(&self) -> Option<Vec<NameTree>> {
        if self.len == 0 {
            return None;
        }
        let inner = self.ty.names()?;
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

impl<T, S> HierarchicalBuildFrom<S> for ArrayData<T>
where
    T: HierarchicalBuildFrom<S>,
{
    fn build_from(&mut self, path: &mut NameBuf, source: &S) {
        for (i, elem) in self.elems.iter_mut().enumerate() {
            path.push(i);
            HierarchicalBuildFrom::<S>::build_from(elem, path, source);
            path.pop();
        }
    }
}

// TODO: Maybe do lazy transformation here.
impl<T: HasTransformedView> HasTransformedView for ArrayData<T> {
    type TransformedView = ArrayData<Transformed<T>>;

    fn transformed_view(
        &self,
        trans: geometry::transform::Transformation,
    ) -> Self::TransformedView {
        Self::TransformedView {
            elems: self
                .elems
                .iter()
                .map(|elem| elem.transformed_view(trans))
                .collect(),
            ty_len: self.ty_len,
        }
    }
}

impl<T: layout::IsBundle, B: BundleBuilder<T>> BundleBuilder<ArrayData<T>> for ArrayData<B> {
    fn build(self) -> Result<ArrayData<T>> {
        let mut elems = Vec::new();
        for e in self.elems {
            elems.push(e.build()?);
        }
        Ok(ArrayData {
            elems,
            ty_len: self.ty_len,
        })
    }
}

impl<T: Flatten<Direction>> Flatten<Direction> for Array<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        for _ in 0..self.len {
            self.ty.flatten(output);
        }
    }
}

impl<T: FlatLen> FlatLen for ArrayData<T> {
    fn len(&self) -> usize {
        self.elems.len() * self.ty_len
    }
}

impl<T: Flatten<Node>> Flatten<Node> for ArrayData<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        self.elems.iter().for_each(|e| e.flatten(output));
    }
}

impl<T: HasNestedView> HasNestedView for ArrayData<T> {
    type NestedView = ArrayData<T::NestedView>;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView {
        ArrayData {
            elems: self
                .elems
                .iter()
                .map(|elem| elem.nested_view(parent))
                .collect(),
            ty_len: self.ty_len,
        }
    }
}

impl<T: HasTerminalView> HasTerminalView for ArrayData<T> {
    type TerminalView = ArrayData<T::TerminalView>;

    fn terminal_view(
        cell: CellId,
        cell_io: &Self,
        instance: InstanceId,
        instance_io: &Self,
    ) -> Self::TerminalView {
        ArrayData {
            elems: cell_io
                .elems
                .iter()
                .zip(instance_io.elems.iter())
                .map(|(cell_elem, instance_elem)| {
                    HasTerminalView::terminal_view(cell, cell_elem, instance, instance_elem)
                })
                .collect(),
            ty_len: cell_io.ty_len,
        }
    }
}

impl<T: Flatten<PortGeometry>> Flatten<PortGeometry> for ArrayData<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<PortGeometry>,
    {
        self.elems.iter().for_each(|e| e.flatten(output));
    }
}

impl<T, I> Index<I> for ArrayData<T>
where
    I: SliceIndex<[T]>,
{
    type Output = <I as SliceIndex<[T]>>::Output;
    #[inline]
    fn index(&self, index: I) -> &Self::Output {
        Index::index(&self.elems, index)
    }
}

impl<T, I> IndexMut<I> for ArrayData<T>
where
    I: SliceIndex<[T]>,
{
    #[inline]
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        IndexMut::index_mut(&mut self.elems, index)
    }
}

impl<T> Connect<T> for T {}
impl<T> Connect<&T> for T {}
impl<T> Connect<T> for &T {}

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

impl<T> ArrayData<T> {
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
    use crate::io::*;

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
