//! Built-in implementations of IO traits.

use std::fmt::Display;
use std::ops::IndexMut;
use std::{ops::DerefMut, slice::SliceIndex};

use crate::layout::error::LayoutError;

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

/// Blocks with no ports can declare their `Io` as `()`.
impl Io for () {}

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

impl Undirected for () {}

impl SchematicType for () {
    type Data = ();
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
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

impl LayoutType for () {
    type Data = ();
    type Builder = ();

    fn builder(&self) {}
}

impl LayoutDataBuilder<()> for () {
    fn build(self) -> Result<()> {
        Ok(())
    }
}

impl Flatten<LayoutPort> for () {
    fn flatten<E>(&self, _output: &mut E)
    where
        E: Extend<LayoutPort>,
    {
    }
}

impl FlatLen for Signal {
    fn len(&self) -> usize {
        1
    }
}

impl SchematicType for Signal {
    type Data = Node;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        if let [id, rest @ ..] = ids {
            (*id, rest)
        } else {
            unreachable!();
        }
    }
}

impl LayoutType for Signal {
    type Data = LayoutPort;
    type Builder = LayoutPortBuilder;

    fn builder(&self) -> Self::Builder {
        LayoutPortBuilder::default()
    }
}

impl HasNameTree for Signal {
    fn names(&self) -> Option<Vec<NameTree>> {
        Some(vec![])
    }
}

impl Undirected for Signal {}

impl FlatLen for ShapePort {
    fn len(&self) -> usize {
        1
    }
}

impl LayoutType for ShapePort {
    type Data = Shape;
    type Builder = OptionBuilder<Shape>;

    fn builder(&self) -> Self::Builder {
        Default::default()
    }
}

impl HasNameTree for ShapePort {
    fn names(&self) -> Option<Vec<NameTree>> {
        Some(vec![])
    }
}

impl Undirected for ShapePort {}

impl CustomLayoutType<Signal> for ShapePort {
    fn from_layout_type(_other: &Signal) -> Self {
        ShapePort
    }
}

impl FlatLen for Node {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<Node> for Node {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        output.extend(std::iter::once(*self));
    }
}

impl Undirected for Node {}

impl FlatLen for Shape {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<LayoutPort> for Shape {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<LayoutPort>,
    {
        output.extend(std::iter::once(LayoutPort {
            primary: self.clone(),
            unnamed_shapes: Vec::new(),
            named_shapes: HashMap::new(),
        }));
    }
}

impl Undirected for Shape {}

impl<T: LayoutData> LayoutDataBuilder<T> for OptionBuilder<T> {
    fn build(self) -> Result<T> {
        self.build()
    }
}

impl<T: Undirected> Undirected for OptionBuilder<T> {}

impl<T: Undirected> Undirected for Option<T> {}

impl FlatLen for LayoutPort {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<LayoutPort> for LayoutPort {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<LayoutPort>,
    {
        output.extend(std::iter::once(self.clone()));
    }
}

impl<'a> From<TransformedLayoutPort<'a>> for LayoutPort {
    fn from(value: TransformedLayoutPort<'a>) -> Self {
        Self {
            primary: value.primary,
            unnamed_shapes: value.unnamed_shapes.to_vec(),
            named_shapes: value
                .named_shapes
                .to_hash_map()
                .into_iter()
                .map(|(name, shape)| (name.clone(), shape))
                .collect(),
        }
    }
}

impl HasTransformedView for LayoutPort {
    type TransformedView<'a> = TransformedLayoutPort<'a>;

    fn transformed_view(
        &self,
        trans: geometry::transform::Transformation,
    ) -> Self::TransformedView<'_> {
        Self::TransformedView {
            primary: self.primary.transformed_view(trans),
            unnamed_shapes: self.unnamed_shapes.transformed_view(trans),
            named_shapes: self.named_shapes.transformed_view(trans),
        }
    }
}

impl Undirected for LayoutPort {}

impl FlatLen for LayoutPortBuilder {
    fn len(&self) -> usize {
        1
    }
}

impl LayoutDataBuilder<LayoutPort> for LayoutPortBuilder {
    fn build(self) -> Result<LayoutPort> {
        Ok(LayoutPort {
            primary: self.primary.ok_or_else(|| {
                tracing::event!(
                    Level::ERROR,
                    "primary shape in port geometry was not specified"
                );
                LayoutError::IoDefinition
            })?,
            unnamed_shapes: self.unnamed_shapes,
            named_shapes: self.named_shapes,
        })
    }
}

impl Undirected for LayoutPortBuilder {}

impl<T: Undirected> AsRef<T> for Input<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Undirected> Deref for Input<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Undirected> DerefMut for Input<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<U: Undirected, T: From<U> + Undirected> From<U> for Input<T> {
    fn from(value: U) -> Self {
        Input(value.into())
    }
}

impl<T: Undirected> Borrow<T> for Input<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> SchematicType for Input<T>
where
    T: Undirected + SchematicType,
    T::Data: Undirected,
{
    type Data = Input<T::Data>;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (Input(data), ids)
    }
}

impl<T> LayoutType for Input<T>
where
    T: Undirected + LayoutType,
    T::Data: Undirected,
    T::Builder: Undirected,
{
    type Data = T::Data;
    type Builder = T::Builder;

    fn builder(&self) -> Self::Builder {
        self.0.builder()
    }
}

impl<T, U: CustomLayoutType<T>> CustomLayoutType<Input<T>> for U
where
    T: Undirected + LayoutType,
    T::Data: Undirected,
    T::Builder: Undirected,
{
    fn from_layout_type(other: &Input<T>) -> Self {
        <U as CustomLayoutType<T>>::from_layout_type(&other.0)
    }
}

impl<T: Undirected + HasNameTree> HasNameTree for Input<T> {
    fn names(&self) -> Option<Vec<NameTree>> {
        self.0.names()
    }
}

impl<T: Undirected + FlatLen> FlatLen for Input<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Undirected + FlatLen> Flatten<Direction> for Input<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        output.extend(std::iter::repeat(Direction::Input).take(self.0.len()))
    }
}
impl<T: Undirected + Flatten<Node>> Flatten<Node> for Input<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        self.0.flatten(output);
    }
}

impl<T> SchematicType for Output<T>
where
    T: Undirected + SchematicType,
    T::Data: Undirected,
{
    type Data = Output<T::Data>;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (Output(data), ids)
    }
}

impl<T> LayoutType for Output<T>
where
    T: Undirected + LayoutType,
    T::Data: Undirected,
    T::Builder: Undirected,
{
    type Data = T::Data;
    type Builder = T::Builder;

    fn builder(&self) -> Self::Builder {
        self.0.builder()
    }
}

impl<T, U: CustomLayoutType<T>> CustomLayoutType<Output<T>> for U
where
    T: Undirected + LayoutType,
    T::Data: Undirected,
    T::Builder: Undirected,
{
    fn from_layout_type(other: &Output<T>) -> Self {
        <U as CustomLayoutType<T>>::from_layout_type(&other.0)
    }
}

impl<T: Undirected + HasNameTree> HasNameTree for Output<T> {
    fn names(&self) -> Option<Vec<NameTree>> {
        self.0.names()
    }
}

impl<T: Undirected + FlatLen> FlatLen for Output<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Undirected + FlatLen> Flatten<Direction> for Output<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        output.extend(std::iter::repeat(Direction::Output).take(self.0.len()))
    }
}
impl<T: Undirected + Flatten<Node>> Flatten<Node> for Output<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        self.0.flatten(output);
    }
}

impl<T: Undirected> AsRef<T> for Output<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T: Undirected> Deref for Output<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Undirected> DerefMut for Output<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<U: Undirected, T: From<U> + Undirected> From<U> for Output<T> {
    fn from(value: U) -> Self {
        Output(value.into())
    }
}

impl<T: Undirected> Borrow<T> for Output<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T> SchematicType for InOut<T>
where
    T: Undirected + SchematicType,
    T::Data: Undirected,
{
    type Data = InOut<T::Data>;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (InOut(data), ids)
    }
}

impl<T> LayoutType for InOut<T>
where
    T: Undirected + LayoutType,
    T::Data: Undirected,
    T::Builder: Undirected,
{
    type Data = T::Data;
    type Builder = T::Builder;

    fn builder(&self) -> Self::Builder {
        self.0.builder()
    }
}

impl<T, U: CustomLayoutType<T>> CustomLayoutType<InOut<T>> for U
where
    T: Undirected + LayoutType,
    T::Data: Undirected,
    T::Builder: Undirected,
{
    fn from_layout_type(other: &InOut<T>) -> Self {
        <U as CustomLayoutType<T>>::from_layout_type(&other.0)
    }
}

impl<T: Undirected + HasNameTree> HasNameTree for InOut<T> {
    fn names(&self) -> Option<Vec<NameTree>> {
        self.0.names()
    }
}

impl<T: Undirected + FlatLen> FlatLen for InOut<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T: Undirected + FlatLen> Flatten<Direction> for InOut<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Direction>,
    {
        output.extend(std::iter::repeat(Direction::Input).take(self.0.len()))
    }
}
impl<T: Undirected + Flatten<Node>> Flatten<Node> for InOut<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        self.0.flatten(output);
    }
}
impl<T: Undirected> AsRef<T> for InOut<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}
impl<T: Undirected> Deref for InOut<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Undirected> DerefMut for InOut<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<U: Undirected, T: From<U> + Undirected> From<U> for InOut<T> {
    fn from(value: U) -> Self {
        InOut(value.into())
    }
}

impl<T: Undirected> Borrow<T> for InOut<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

impl<T: FlatLen> FlatLen for Array<T> {
    fn len(&self) -> usize {
        self.ty.len() * self.len
    }
}

impl<T: SchematicType> SchematicType for Array<T> {
    type Data = ArrayData<T::Data>;

    fn instantiate<'n>(&self, mut ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
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

impl<T: LayoutType> LayoutType for Array<T> {
    type Data = ArrayData<T::Data>;
    type Builder = ArrayData<T::Builder>;

    fn builder(&self) -> Self::Builder {
        Self::Builder {
            elems: (0..self.len).map(|_| self.ty.builder()).collect(),
            ty_len: self.ty.len(),
        }
    }
}

impl<T: LayoutType, U: LayoutType + CustomLayoutType<T>> CustomLayoutType<Array<T>> for Array<U> {
    fn from_layout_type(other: &Array<T>) -> Self {
        Self {
            ty: <U as CustomLayoutType<T>>::from_layout_type(&other.ty),
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
                    fragment: NameFragment::Idx(i),
                    children: inner.clone(),
                })
                .collect(),
        )
    }
}

// TODO: Maybe do lazy transformation here.
impl<T: HasTransformedView> HasTransformedView for ArrayData<T> {
    type TransformedView<'a>
    = ArrayData<Transformed<'a, T>> where T: 'a;

    fn transformed_view(
        &self,
        trans: geometry::transform::Transformation,
    ) -> Self::TransformedView<'_> {
        Self::TransformedView {
            elems: self
                .elems
                .iter()
                .map(|elem: &T| elem.transformed_view(trans))
                .collect(),
            ty_len: self.ty_len,
        }
    }
}

impl<T: LayoutData, B: LayoutDataBuilder<T>> LayoutDataBuilder<ArrayData<T>> for ArrayData<B> {
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

impl<T: Undirected> Undirected for Array<T> {}

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

impl<T: Flatten<LayoutPort>> Flatten<LayoutPort> for ArrayData<T> {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<LayoutPort>,
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

impl<T: Undirected> Undirected for ArrayData<T> {}

impl<T> Connect<T> for T {}
impl<T: Undirected> Connect<T> for Input<T> {}
impl<T: Undirected> Connect<T> for Output<T> {}
impl<T: Undirected> Connect<T> for InOut<T> {}
impl<T: Undirected> Connect<Input<T>> for T {}
impl<T: Undirected> Connect<Output<T>> for T {}
impl<T: Undirected> Connect<InOut<T>> for T {}

// For analog circuits, we don't check directionality of connections.
impl<T: Undirected> Connect<Input<T>> for Output<T> {}
impl<T: Undirected> Connect<Input<T>> for InOut<T> {}
impl<T: Undirected> Connect<Output<T>> for Input<T> {}
impl<T: Undirected> Connect<Output<T>> for InOut<T> {}
impl<T: Undirected> Connect<InOut<T>> for Input<T> {}
impl<T: Undirected> Connect<InOut<T>> for Output<T> {}

impl From<ArcStr> for NameFragment {
    fn from(value: ArcStr) -> Self {
        Self::Str(value)
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
        if let Some(fragment) = self.fragments.get(0) {
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
            fragment: fragment.into(),
            children,
        }
    }

    /// Flattens the node name tree, returning a list of [`NameBuf`]s.
    pub fn flatten(&self) -> Vec<NameBuf> {
        self.flatten_inner(NameBuf::new())
    }

    fn flatten_inner(&self, mut parent: NameBuf) -> Vec<NameBuf> {
        parent.fragments.push(self.fragment.clone());
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
}

impl Port {
    #[inline]
    pub(crate) fn new(node: Node, direction: Direction) -> Self {
        Self { node, direction }
    }

    #[inline]
    #[allow(dead_code)]
    pub(crate) fn direction(&self) -> Direction {
        self.direction
    }

    #[inline]
    pub(crate) fn node(&self) -> Node {
        self.node
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
}
