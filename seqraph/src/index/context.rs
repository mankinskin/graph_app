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
    fn context_path_segment(
        &'a mut self,
        location: ChildLocation
    ) -> (Child, ChildLocation) {
        let pattern = self.graph().expect_pattern_at(&location);
        let context = Side::split_context(&pattern, location.sub_index);
        if context.len() < 2 {
            if context.is_empty() {
                assert!(!context.is_empty());
            }
            (*context.into_iter().next().unwrap(), location)
        } else {
            let c = self.indexer.index_pattern(context).unwrap().0;
            (c, location.to_child_location(Side::inner_pos_after_context_indexed(location.sub_index)))
        }
    }
    pub fn try_context_path(
        &'a mut self,
        context_path: Vec<ChildLocation>,
        inner: Child,
    ) -> Option<(Child, ChildLocation)> {
        context_path.into_iter().rev().fold(None, |acc, location| {
            let (context, inner_location) = self.context_path_segment(location);
            Some(if let Some((acc_ctx, _)) = acc {
                let (back, front) = Side::context_inner_order(&context, &acc_ctx);
                let context = self.indexer.index_pattern([back[0], front[0]]).unwrap().0;
                let pid = self.graph_mut().add_pattern_with_update(location, Side::concat_inner_and_context(inner, context));
                let (sub_index, _) = Side::back_front_order(0, 1);
                (context, ChildLocation {
                    parent: inner_location.parent,
                    pattern_id: pid,
                    sub_index,
                })
            } else {
                (context, inner_location)
            })
        })
    }
    /// indexes context patterns along a path and accumulates nested contexts
    pub fn context_entry_path(
        &'a mut self,
        entry: ChildLocation,
        context_path: Vec<ChildLocation>,
        inner: Child,
    ) -> (Child, ChildLocation) {
        let context_path = vec![entry].tap_mut(|v| v.extend(context_path));
        self.try_context_path(
            context_path,
            inner,
        ).unwrap()
    }
}