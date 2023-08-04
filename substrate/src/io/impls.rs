//! Built-in implementations of IO traits.

use std::fmt::Display;
use std::ops::IndexMut;
use std::{ops::DerefMut, slice::SliceIndex};

use crate::layout::error::LayoutError;
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
    type Data = PortGeometry;
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

impl FlatLen for ShapePort {
    fn len(&self) -> usize {
        1
    }
}

impl LayoutType for ShapePort {
    type Data = IoShape;
    type Builder = OptionBuilder<IoShape>;

    fn builder(&self) -> Self::Builder {
        Default::default()
    }
}

impl HasNameTree for ShapePort {
    fn names(&self) -> Option<Vec<NameTree>> {
        Some(vec![])
    }
}

impl CustomLayoutType<Signal> for ShapePort {
    fn from_layout_type(_other: &Signal) -> Self {
        ShapePort
    }
}

impl FlatLen for LayoutPort {
    fn len(&self) -> usize {
        1
    }
}

impl LayoutType for LayoutPort {
    type Data = PortGeometry;
    type Builder = PortGeometryBuilder;

    fn builder(&self) -> Self::Builder {
        Default::default()
    }
}

impl HasNameTree for LayoutPort {
    fn names(&self) -> Option<Vec<NameTree>> {
        Some(vec![])
    }
}

impl CustomLayoutType<Signal> for LayoutPort {
    fn from_layout_type(_other: &Signal) -> Self {
        LayoutPort
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

impl HasNestedView for Node {
    type NestedView<'a> = NestedNode;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        NestedNode {
            node: *self,
            path: parent.clone(),
        }
    }
}

impl HasNestedView for NestedNode {
    type NestedView<'a> = NestedNode;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        NestedNode {
            node: self.node,
            path: self.path.prepend(parent),
        }
    }
}

impl FlatLen for NestedNode {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<Node> for NestedNode {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<Node>,
    {
        output.extend(std::iter::once(self.node));
    }
}

impl FlatLen for IoShape {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<PortGeometry> for IoShape {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<PortGeometry>,
    {
        output.extend(std::iter::once(PortGeometry {
            primary: self.clone(),
            unnamed_shapes: Vec::new(),
            named_shapes: HashMap::new(),
        }));
    }
}

impl HierarchicalBuildFrom<NamedPorts> for OptionBuilder<IoShape> {
    fn build_from(&mut self, path: &mut NameBuf, source: &NamedPorts) {
        self.set(source.get(path).unwrap().primary.clone());
    }
}

impl<T: LayoutData> LayoutDataBuilder<T> for OptionBuilder<T> {
    fn build(self) -> Result<T> {
        self.build()
    }
}

impl FlatLen for PortGeometry {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<PortGeometry> for PortGeometry {
    fn flatten<E>(&self, output: &mut E)
    where
        E: Extend<PortGeometry>,
    {
        output.extend(std::iter::once(self.clone()));
    }
}

impl<'a> From<TransformedPortGeometry<'a>> for PortGeometry {
    fn from(value: TransformedPortGeometry<'a>) -> Self {
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

impl HasTransformedView for PortGeometry {
    type TransformedView<'a> = TransformedPortGeometry<'a>;

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

impl FlatLen for PortGeometryBuilder {
    fn len(&self) -> usize {
        1
    }
}

impl LayoutDataBuilder<PortGeometry> for PortGeometryBuilder {
    fn build(self) -> Result<PortGeometry> {
        Ok(PortGeometry {
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

impl HierarchicalBuildFrom<NamedPorts> for PortGeometryBuilder {
    fn build_from(&mut self, path: &mut NameBuf, source: &NamedPorts) {
        let source = source.get(path).unwrap();
        self.primary = Some(source.primary.clone());
        self.unnamed_shapes.clone_from(&source.unnamed_shapes);
        self.named_shapes.clone_from(&source.named_shapes);
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

impl<T> SchematicType for Input<T>
where
    T: SchematicType,
{
    type Data = T::Data;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (data, ids)
    }
}

impl<T> LayoutType for Input<T>
where
    T: LayoutType,
{
    type Data = T::Data;
    type Builder = T::Builder;

    fn builder(&self) -> Self::Builder {
        self.0.builder()
    }
}

impl<T, U: CustomLayoutType<T>> CustomLayoutType<Input<T>> for U
where
    T: LayoutType,
{
    fn from_layout_type(other: &Input<T>) -> Self {
        <U as CustomLayoutType<T>>::from_layout_type(&other.0)
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
    type NestedView<'a> = T::NestedView<'a> where T: 'a;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        self.0.nested_view(parent)
    }
}

impl<T> SchematicType for Output<T>
where
    T: SchematicType,
{
    type Data = T::Data;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (data, ids)
    }
}

impl<T> LayoutType for Output<T>
where
    T: LayoutType,
{
    type Data = T::Data;
    type Builder = T::Builder;

    fn builder(&self) -> Self::Builder {
        self.0.builder()
    }
}

impl<T, U: CustomLayoutType<T>> CustomLayoutType<Output<T>> for U
where
    T: LayoutType,
{
    fn from_layout_type(other: &Output<T>) -> Self {
        <U as CustomLayoutType<T>>::from_layout_type(&other.0)
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
    type NestedView<'a> = T::NestedView<'a> where T: 'a;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        self.0.nested_view(parent)
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

impl<T> SchematicType for InOut<T>
where
    T: SchematicType,
{
    type Data = T::Data;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (data, ids)
    }
}

impl<T> LayoutType for InOut<T>
where
    T: LayoutType,
{
    type Data = T::Data;
    type Builder = T::Builder;

    fn builder(&self) -> Self::Builder {
        self.0.builder()
    }
}

impl<T, U: CustomLayoutType<T>> CustomLayoutType<InOut<T>> for U
where
    T: LayoutType,
{
    fn from_layout_type(other: &InOut<T>) -> Self {
        <U as CustomLayoutType<T>>::from_layout_type(&other.0)
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

impl<T: SchematicData + HasNestedView> HasNestedView for InOut<T> {
    type NestedView<'a> = T::NestedView<'a> where T: 'a;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
        self.0.nested_view(parent)
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

impl<T> SchematicType for Flipped<T>
where
    T: SchematicType,
{
    type Data = T::Data;
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        let (data, ids) = self.0.instantiate(ids);
        (data, ids)
    }
}

impl<T> LayoutType for Flipped<T>
where
    T: LayoutType,
{
    type Data = T::Data;
    type Builder = T::Builder;

    fn builder(&self) -> Self::Builder {
        self.0.builder()
    }
}

impl<T, U: CustomLayoutType<T>> CustomLayoutType<Flipped<T>> for U
where
    T: LayoutType,
{
    fn from_layout_type(other: &Flipped<T>) -> Self {
        <U as CustomLayoutType<T>>::from_layout_type(&other.0)
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
    type NestedView<'a> = T::NestedView<'a> where T: 'a;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
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

impl<T: Io> Io for Array<T> {}

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
    type NestedView<'a> = ArrayData<T::NestedView<'a>> where T: 'a;

    fn nested_view(&self, parent: &InstancePath) -> Self::NestedView<'_> {
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
