use crate::*;
use super::*;

trait ContextLocation {
    fn index<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
        Side: IndexSide<D>,
        Trav: TraversableMut<T>,
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
pub struct Contexter<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> {
    indexer: Indexer<T, D>,
    _ty: std::marker::PhantomData<(D, Side)>,
}
impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Contexter<T, D, Side> {
    pub fn new(indexer: Indexer<T, D>) -> Self {
        Self {
            indexer,
            _ty: Default::default()
        }
    }
}

impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Traversable<T> for Contexter<T, D, Side> {
    type Guard<'g> = RwLockReadGuard<'g, Hypergraph<T>> where Side: 'g;
    fn graph<'g>(&'g self) -> Self::Guard<'g> {
        self.indexer.graph()
    }
}

impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> TraversableMut<T> for Contexter<T, D, Side> {
    type GuardMut<'g> = RwLockWriteGuard<'g, Hypergraph<T>> where Side: 'g;
    fn graph_mut<'g>(&'g mut self) -> Self::GuardMut<'g> {
        self.indexer.graph_mut()
    }
}
//pub trait IndexContext<T: Tokenize, D: IndexDirection, Side: IndexSide<D>>: Indexing<T, D> {
impl<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> Contexter<T, D, Side> {
    /// replaces context in pattern at location with child and returns it with new location
    pub fn pather(&self) -> Pather<T, D, Side> {
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