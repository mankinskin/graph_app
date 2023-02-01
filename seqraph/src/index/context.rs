use crate::*;

trait ContextLocation {
    fn index<
        'a: 'g,
        'g,
        Side: IndexSide<<Trav::Kind as GraphKind>::Direction>,
        Trav: TraversableMut,
    >(self, trav: Trav) -> (Child, ChildLocation);
}
pub trait ContextPath:
    IntoIterator<
        Item=<Self as ContextPath>::Item,
        IntoIter=<Self as ContextPath>::IntoIter,
    >
    + Debug
    + Send
    + Sync
    + Unpin
{
    type Item: Borrow<ChildLocation> + Debug + Send + Sync + Unpin;
    type IntoIter: DoubleEndedIterator<Item=<Self as ContextPath>::Item> + Debug + Send + Sync + Unpin + ExactSizeIterator;
}
impl<
    Item: Borrow<ChildLocation> + Debug + Send + Sync + Unpin,
    IntoIter: DoubleEndedIterator<Item=Item> + Debug + Send + Sync + Unpin + ExactSizeIterator,
    T: IntoIterator<Item=Item, IntoIter=IntoIter> + Debug + Send + Sync + Unpin
> ContextPath for T {
    type Item = Item;
    type IntoIter = IntoIter;
}

#[derive(Debug, Clone)]
pub struct Contexter<G: GraphKind, Side: IndexSide<G::Direction>> {
    pub(crate) indexer: Indexer<G>,
    _ty: std::marker::PhantomData<Side>,
}
impl<G: GraphKind, Side: IndexSide<G::Direction>> Contexter<G, Side> {
    pub fn new(indexer: Indexer<G>) -> Self {
        Self {
            indexer,
            _ty: Default::default()
        }
    }
}

//pub trait IndexContext<T: Tokenize, D: IndexDirection, Side: IndexSide<D>>: Indexing<T, D> {
impl<G: GraphKind, Side: IndexSide<G::Direction>> Contexter<G, Side> {
    /// replaces context in pattern at location with child and returns it with new location
    pub fn pather(&self) -> Pather<G, Side> {
        Pather::new(self.indexer.clone())
    }
    #[instrument(skip(self, path))]
    pub fn try_context_path(
        &mut self,
        path: impl ContextPath,
    ) -> Option<(Child, ChildLocation)> {
        let path = path.into_iter();
        self.pather().index_primary_path::<ContextSide, _>(
            path,
        ).map(|split|
            (split.inner, split.location)
        )
    }
}