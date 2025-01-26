use hypercontext_api::graph::vertex::{child::Child, location::child::ChildLocation, pattern::IntoPattern};

use super::IndexSide;

pub trait RelativeSide<S: IndexSide<<BaseGraphKind as GraphKind>::Direction>>:
Sync + Send + Unpin
{
    type Opposite: RelativeSide<S>;
    type Range: PatternRangeIndex + StartInclusive;
    fn is_context_side() -> bool;
    fn is_inner_side() -> bool {
        !Self::is_context_side()
    }
    fn exclusive_primary_index(index: usize) -> Option<usize>;
    fn exclusive_primary_location(location: ChildLocation) -> Option<ChildLocation> {
        Some(ChildLocation {
            sub_index: Self::exclusive_primary_index(location.sub_index)?,
            ..location
        })
    }
    fn exclusive_secondary_index(index: usize) -> Option<usize>;
    fn exclusive_secondary_location(location: ChildLocation) -> Option<ChildLocation> {
        Some(ChildLocation {
            sub_index: Self::exclusive_secondary_index(location.sub_index)?,
            ..location
        })
    }
    fn primary_range(index: usize) -> Self::Range;
    fn primary_indexed_pos(index: usize) -> usize;
    fn secondary_range(index: usize) -> <Self::Opposite as RelativeSide<S>>::Range {
        <Self::Opposite as RelativeSide<S>>::primary_range(index)
    }
    fn split_primary(
        pattern: &'_ impl IntoPattern,
        pos: usize,
    ) -> &'_ [Child] {
        &pattern.borrow()[Self::primary_range(pos)]
    }
    fn split_secondary(
        pattern: &'_ impl IntoPattern,
        pos: usize,
    ) -> &'_ [Child] {
        &pattern.borrow()[Self::secondary_range(pos)]
    }
    fn outer_inner_order(
        outer: Child,
        inner: Child,
    ) -> (Child, Child);
    fn index_inner_and_context(
        indexer: &mut Indexer,
        inner: Child,
        context: Child,
    ) -> Child;
    fn has_secondary_exclusive(
        pattern: &'_ impl IntoPattern,
        pos: usize,
    ) -> bool {
        Self::is_inner_side() && !Self::split_secondary(pattern, pos).is_empty()
            || Self::is_context_side() && Self::split_secondary(pattern, pos).len() > 1
    }
}

pub struct ContextSide;

impl<S: IndexSide> RelativeSide<S> for ContextSide {
    type Opposite = InnerSide;
    type Range = <S as IndexSide>::ContextRange;
    fn is_context_side() -> bool {
        true
    }
    fn exclusive_primary_index(index: usize) -> Option<usize> {
        Some(index)
    }
    fn exclusive_primary_location(location: ChildLocation) -> Option<ChildLocation> {
        Some(location)
    }
    fn exclusive_secondary_index(index: usize) -> Option<usize> {
        <S as IndexSide<_>>::next_inner_index(index)
    }
    fn primary_range(index: usize) -> Self::Range {
        S::context_range(index)
    }
    fn primary_indexed_pos(index: usize) -> usize {
        S::inner_pos_after_context_indexed(index)
    }
    fn index_inner_and_context(
        indexer: &mut Indexer,
        inner: Child,
        context: Child,
    ) -> Child {
        let (back, front) = <Self as RelativeSide<S>>::outer_inner_order(context, inner);
        if let Ok((c, _)) = indexer.index_pattern([back, front]) {
            c
        } else {
            indexer.graph_mut().insert_pattern([back, front])
        }
        //indexer.graph_mut().insert_pattern([back, front])
    }
    fn outer_inner_order(
        outer: Child,
        inner: Child,
    ) -> (Child, Child) {
        let (back, front) = S::context_inner_order(&outer, &inner);
        (back[0], front[0])
    }
}

pub struct InnerSide;

impl<S: IndexSide> RelativeSide<S> for InnerSide {
    type Opposite = ContextSide;
    type Range = <S as IndexSide>::InnerRange;
    fn is_context_side() -> bool {
        false
    }
    fn exclusive_primary_index(index: usize) -> Option<usize> {
        S::next_inner_index(index)
    }
    fn exclusive_secondary_index(index: usize) -> Option<usize> {
        Some(index)
    }
    fn exclusive_secondary_location(location: ChildLocation) -> Option<ChildLocation> {
        Some(location)
    }
    fn primary_range(index: usize) -> Self::Range {
        S::inner_range(index)
    }
    fn primary_indexed_pos(index: usize) -> usize {
        index
    }
    fn index_inner_and_context(
        indexer: &mut Indexer,
        inner: Child,
        context: Child,
    ) -> Child {
        let (back, front) = <Self as RelativeSide<S>>::outer_inner_order(context, inner);
        //indexer.graph_mut().insert_pattern([back, front])
        match indexer.index_pattern([back, front]) {
            Ok((c, _)) => c,
            _ => indexer.graph_mut().insert_pattern([back, front]),
        }
    }
    fn outer_inner_order(
        outer: Child,
        inner: Child,
    ) -> (Child, Child) {
        let (back, front) = S::context_inner_order(&inner, &outer);
        (back[0], front[0])
    }
}
