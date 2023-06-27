//! Built-in implementations of IO traits.

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
    fn flatten(&self) -> Vec<Node> {
        (*self).flatten()
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
    fn flatten(&self) -> Vec<Direction> {
        vec![]
    }
}

impl Undirected for () {}

impl SchematicType for () {
    type Data = ();
    fn instantiate<'n>(&self, ids: &'n [Node]) -> (Self::Data, &'n [Node]) {
        ((), ids)
    }
}

impl Flatten<Node> for () {
    fn flatten(&self) -> Vec<Node> {
        vec![]
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
    fn flatten(&self) -> Vec<PortGeometry> {
        vec![]
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

impl Undirected for Signal {}

impl FlatLen for Node {
    fn len(&self) -> usize {
        1
    }
}

impl Flatten<Node> for Node {
    fn flatten(&self) -> Vec<Node> {
        vec![*self]
    }
}

impl Undirected for Node {}

impl FlatLen for PortGeometry {
    fn len(&self) -> usize {
        1
    }
}
impl Flatten<PortGeometry> for PortGeometry {
    fn flatten(&self) -> Vec<PortGeometry> {
        vec![self.clone()]
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
    type Data = Input<T::Data>;
    type Builder = Input<T::Builder>;

    fn builder(&self) -> Self::Builder {
        Input(self.0.builder())
    }
}

impl<T: Undirected + LayoutData, B: Undirected + LayoutDataBuilder<T>> LayoutDataBuilder<Input<T>>
    for Input<B>
{
    fn build(self) -> Result<Input<T>> {
        Ok(Input(self.0.build()?))
    }
}

impl<T: Undirected + Flatten<PortGeometry>> Flatten<PortGeometry> for Input<T> {
    fn flatten(&self) -> Vec<PortGeometry> {
        self.0.flatten()
    }
}

impl<T: Undirected + FlatLen> FlatLen for Input<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Undirected + FlatLen> Flatten<Direction> for Input<T> {
    fn flatten(&self) -> Vec<Direction> {
        vec![Direction::Input; self.0.len()]
    }
}

impl<T: Undirected + Flatten<Node>> Flatten<Node> for Input<T> {
    fn flatten(&self) -> Vec<Node> {
        self.0.flatten()
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
    type Data = Output<T::Data>;
    type Builder = Output<T::Builder>;

    fn builder(&self) -> Self::Builder {
        Output(self.0.builder())
    }
}

impl<T: Undirected + LayoutData, B: Undirected + LayoutDataBuilder<T>> LayoutDataBuilder<Output<T>>
    for Output<B>
{
    fn build(self) -> Result<Output<T>> {
        Ok(Output(self.0.build()?))
    }
}

impl<T: Undirected + Flatten<PortGeometry>> Flatten<PortGeometry> for Output<T> {
    fn flatten(&self) -> Vec<PortGeometry> {
        self.0.flatten()
    }
}

impl<T: Undirected + FlatLen> FlatLen for Output<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T: Undirected + FlatLen> Flatten<Direction> for Output<T> {
    fn flatten(&self) -> Vec<Direction> {
        vec![Direction::Output; self.0.len()]
    }
}

impl<T: Undirected + Flatten<Node>> Flatten<Node> for Output<T> {
    fn flatten(&self) -> Vec<Node> {
        self.0.flatten()
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
    type Data = InOut<T::Data>;
    type Builder = InOut<T::Builder>;

    fn builder(&self) -> Self::Builder {
        InOut(self.0.builder())
    }
}

impl<T: Undirected + LayoutData, B: Undirected + LayoutDataBuilder<T>> LayoutDataBuilder<InOut<T>>
    for InOut<B>
{
    fn build(self) -> Result<InOut<T>> {
        Ok(InOut(self.0.build()?))
    }
}

impl<T: Undirected + Flatten<PortGeometry>> Flatten<PortGeometry> for InOut<T> {
    fn flatten(&self) -> Vec<PortGeometry> {
        self.0.flatten()
    }
}

impl<T: Undirected + FlatLen> FlatLen for InOut<T> {
    #[inline]
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T: Undirected + FlatLen> Flatten<Direction> for InOut<T> {
    fn flatten(&self) -> Vec<Direction> {
        vec![Direction::InOut; self.0.len()]
    }
}
impl<T: Undirected + Flatten<Node>> Flatten<Node> for InOut<T> {
    fn flatten(&self) -> Vec<Node> {
        self.0.flatten()
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

impl<T: Flatten<Direction>> Flatten<Direction> for Array<T> {
    fn flatten(&self) -> Vec<Direction> {
        let dirs = self.ty.flatten();
        let len = dirs.len();
        dirs.into_iter().cycle().take(len * self.len).collect()
    }
}

impl<T: Undirected> Undirected for Array<T> {}

impl<T: FlatLen> FlatLen for ArrayData<T> {
    fn len(&self) -> usize {
        self.elems.len() * self.ty_len
    }
}

impl<T: Flatten<Node>> Flatten<Node> for ArrayData<T> {
    fn flatten(&self) -> Vec<Node> {
        self.elems.iter().flat_map(|e| e.flatten()).collect()
    }
}

impl<T> Index<usize> for ArrayData<T> {
    type Output = T;
    #[inline]
    fn index(&self, index: usize) -> &Self::Output {
        self.elems.index(index)
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
