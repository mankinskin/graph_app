use crate::*;
use super::*;

trait ContextLocation {
    fn index<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
        Side: IndexSide<D>,
        Trav: TraversableMut<'a, 'g, T>,
    >(self, trav: Trav) -> (Child, ChildLocation);
}
pub trait ContextPath:
    IntoIterator<
        Item=<Self as ContextPath>::Item,
        IntoIter=<Self as ContextPath>::IntoIter,
    >
    + Debug
{
    type Item: Borrow<ChildLocation> + Debug;
    type IntoIter: DoubleEndedIterator<Item=<Self as ContextPath>::Item> + Debug;
}
impl<
    Item: Borrow<ChildLocation> + Debug,
    IntoIter: DoubleEndedIterator<Item=Item> + Debug,
    T: IntoIterator<Item=Item, IntoIter=IntoIter> + Debug
> ContextPath for T {
    type Item = Item;
    type IntoIter = IntoIter;
}

#[derive(Debug, Clone)]
pub struct Contexter<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> {
    indexer: Indexer<T, D>,
    _ty: std::marker::PhantomData<(D, Side)>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D>> Contexter<T, D, Side> {
    pub fn new(indexer: Indexer<T, D>) -> Self {
        Self {
            indexer,
            _ty: Default::default()
        }
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D> + 'a> Traversable<'a, 'g, T> for Contexter<T, D, Side> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.indexer.graph()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D> + 'a> TraversableMut<'a, 'g, T> for Contexter<T, D, Side> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.indexer.graph_mut()
    }
}
//pub(crate) trait IndexContext<'a: 'g, 'g, T: Tokenize, D: IndexDirection, Side: IndexSide<D>>: Indexing<'a, 'g, T, D> {
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D>> Contexter<T, D, Side> {
    /// replaces context in pattern at location with child and returns it with new location
    pub(crate) fn pather(&self) -> Pather<T, D, Side> {
        Pather::new(self.indexer.clone())
    }
    #[instrument(skip(self, path))]
    pub fn try_context_path(
        &'a mut self,
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