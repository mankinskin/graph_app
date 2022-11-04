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
{
    type Item: Borrow<ChildLocation>;
    type IntoIter: DoubleEndedIterator<Item=<Self as ContextPath>::Item>;
}
impl<
    Item: Borrow<ChildLocation>,
    IntoIter: DoubleEndedIterator<Item=Item>,
    T: IntoIterator<Item=Item, IntoIter=IntoIter>
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
    pub(crate) fn splitter(&self) -> Splitter<T, D, Side> {
        Splitter::new(self.indexer.clone())
    }
    //fn local_context(
    //    &'a mut self,
    //    location: impl Borrow<ChildLocation>,
    //) -> Option<(Child, ChildLocation)> {
    //    self.splitter().entry_perfect_split::<ContextSide, _>(location)
    //        .map(|split|
    //            (split.inner, split.location)
    //        )
    //    //let location = location.borrow();
    //    //let pattern = self.graph().expect_pattern_at(location);
    //    //let context = Side::split_context(&pattern, location.sub_index);
    //    //if context.is_empty() {
    //    //    None
    //    //} else {
    //    //    Some(if context.len() < 2 {
    //    //        (*context.into_iter().next().unwrap(), *location)
    //    //    } else {
    //    //        let mut graph = self.graph_mut();
    //    //        let c = graph.insert_pattern(context);
    //    //        let range = Side::context_range(location.sub_index);
    //    //        graph.replace_in_pattern(location, range, c);
    //    //        (c, location.to_child_location(Side::inner_pos_after_context_indexed(location.sub_index)))
    //    //    })
    //    //}
    //}
    //pub fn try_context_path(
    //    &'a mut self,
    //    path: impl ContextPath,
    //    inner: Child,
    //) -> Option<(Child, ChildLocation)> {
    //    Side::bottom_up_path_iter(path).fold(None, |prev, location| {
    //        let location = location.borrow();
    //        let local = self.local_context(location);
    //        if let Some((prev_ctx, prev_loc)) = prev {
    //            if let Some((context, inner_location)) = local {

    //                // join prev and current context
    //                let (back, front) = Side::context_inner_order(&context, &prev_ctx);
    //                let context = self.indexer.index_pattern([back[0], front[0]]).unwrap().0;
    //                let pid = self.graph_mut().add_pattern_with_update(location, Side::concat_inner_and_context(inner, context));

    //                let (sub_index, _) = Side::back_front_order(0, 1);
    //                Some((context, ChildLocation {
    //                    parent: inner_location.parent,
    //                    pattern_id: pid,
    //                    sub_index,
    //                }))
    //            } else {

    //            }
    //        } else {
    //            local
    //        }
    //    })
    //}
    pub(crate) fn pather(&self) -> Pather<T, D, Side> {
        Pather::new(self.indexer.clone())
    }
    pub fn try_context_path(
        &'a mut self,
        path: impl ContextPath,
    ) -> Option<(Child, ChildLocation)> {
        let mut path = path.into_iter();
        self.pather().index_primary_path::<ContextSide, _>(
            path,
        ).map(|split|
            (split.inner, split.location)
        )
    }
}