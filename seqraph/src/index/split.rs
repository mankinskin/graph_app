use std::num::NonZeroUsize;

use crate::*;
use super::*;

type HashSet<T> = DeterministicHashSet<T>;
type HashMap<K, V> = DeterministicHashMap<K, V>;

#[derive(Debug, Clone)]
pub struct Splitter<T: Tokenize, D: IndexDirection, Side: IndexSide<D>> {
    indexer: Indexer<T, D>,
    _ty: std::marker::PhantomData<(D, Side)>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D>> Splitter<T, D, Side> {
    pub fn new(indexer: Indexer<T, D>) -> Self {
        Self {
            indexer,
            _ty: Default::default()
        }
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D> + 'a> Traversable<'a, 'g, T> for Splitter<T, D, Side> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'g self) -> Self::Guard {
        self.indexer.graph()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D> + 'a> TraversableMut<'a, 'g, T> for Splitter<T, D, Side> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    fn graph_mut(&'g mut self) -> Self::GuardMut {
        self.indexer.graph_mut()
    }
}
//pub(crate) trait IndexSplit<'a: 'g, 'g, T: Tokenize, D: IndexDirection, Side: IndexSide<D>>: Indexing<'a, 'g, T, D> {
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D>> Splitter<T, D, Side> {
    pub(crate) fn contexter(&self) -> Contexter<T, D, Side> {
        Contexter::new(self.indexer.clone())
    }
    pub(crate) fn pather(&self) -> Pather<T, D, Side> {
        Pather::new(self.indexer.clone())
    }
    //pub fn single_entry_split(
    //    &'a mut self,
    //    entry: ChildLocation,
    //    path: impl ContextPath,
    //) -> Option<IndexSplitResult> {
    //    self.pather().index_primary_entry_path::<InnerSide, _>(
    //        entry,
    //        path,
    //    )
    //}
    pub fn single_path_split(
        &'a mut self,
        path: impl ContextPath,
    ) -> Option<IndexSplitResult> {
        self.pather().index_primary_path::<InnerSide, _>(
            path,
        )
    }
    pub fn single_offset_split(
        &'a mut self,
        parent: Child,
        offset: NonZeroUsize,
    ) -> IndexSplitResult {
        self.pather().single_offset_split::<InnerSide>(
            parent,
            offset,
        )
    }
    // split parent at token offset from direction start
    //fn path_segment_split(
    //    &'a mut self,
    //    prev: Option<IndexSplitResult>,
    //    seg: ChildLocation,
    //) -> Option<IndexSplitResult> {
    //    //let mut graph = self.graph_mut();
    //    if let Some(mut prev) = prev {
    //        // index lower context
    //        let (split_context, split_location) = self.contexter()
    //            .try_context_entry_path(
    //                prev.location,
    //                prev.path.clone(),
    //                prev.inner,
    //            );
    //        let inner_context_range = Side::inner_context_range(seg.sub_index);
    //        let inner_context = self.graph().expect_child_pattern_range(
    //            seg,
    //            inner_context_range,
    //        ).to_vec();
    //        if inner_context.is_empty() {
    //            Some(IndexSplitResult {
    //                location: seg,
    //                path: vec![
    //                    split_location
    //                ],
    //                inner: prev.inner,
    //            })
    //        } else {
    //            let inner_pat = Side::concat_inner_and_inner_context(
    //                prev.inner,
    //                inner_context.borrow(),
    //            );
    //            let offset = Side::inner_width_to_offset(
    //                &seg.parent,
    //                pattern::pattern_width(inner_pat),
    //            ).unwrap();
    //            // split other patterns at offset
    //            let other_patterns = self.graph().expect_child_patterns(seg.parent)
    //                .into_iter()
    //                .filter(|(id, _)| **id != seg.pattern_id)
    //                .map(|(id, p)| (*id, p.clone()))
    //                .collect::<HashMap<_, _>>();
    //            let _s = format!("{:#?}", other_patterns);
    //            let mut splits =
    //                self.child_pattern_offset_splits(
    //                    seg.parent,
    //                    other_patterns,
    //                    offset,
    //                ).expect_err("Other pattern with split at same offset!");

    //            prev.location = split_location;
    //            splits.push((seg, self.graph().expect_pattern_at(seg), prev, split_context));

    //            Some(self.unperfect_splits(
    //                seg.parent,
    //                splits,
    //            ))
    //        }
    //    } else {
    //        self.entry_perfect_split(
    //            seg,
    //        ).map(|(_, r)| r)
    //    }
    //}
}
//impl<
//    'a: 'g,
//    'g,
//    T: Tokenize,
//    D: IndexDirection,
//    Trav: Indexing<'a, 'g, T, D>,
//    S: IndexSide<D>,
//> IndexSplit<'a, 'g, T, D, S> for Trav {}