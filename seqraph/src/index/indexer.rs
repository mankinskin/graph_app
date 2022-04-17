use std::{sync::{
    RwLockReadGuard,
    RwLockWriteGuard,
}, ops::{ControlFlow, Range}, borrow::Borrow};

use crate::{
    *,
    vertex::*,
    index::*,
    Hypergraph,
    HypergraphRef,
    DirectedTraversalPolicy,
    QueryRangePath,
    StartPath,
    TraversableMut,
    TraversalNode,
    TraversalIterator,
    TraversalFolder,
    Bft, EndPath, GraphPath, GraphRangePath, DirectedGraphPath,
};

#[derive(Debug, Clone)]
pub struct Indexer<T: Tokenize, D: IndexDirection> {
    graph: HypergraphRef<T>,
    _ty: std::marker::PhantomData<D>,
}
struct Indexing<'a, T: Tokenize + 'a, D: IndexDirection + 'a> {
    _ty: std::marker::PhantomData<(&'a T, D)>,
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Traversable<'a, 'g, T> for Indexer<T, D> {
    type Guard = RwLockReadGuard<'g, Hypergraph<T>>;
    fn graph(&'a self) -> Self::Guard {
        self.graph.read().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> TraversableMut<'a, 'g, T> for Indexer<T, D> {
    type GuardMut = RwLockWriteGuard<'g, Hypergraph<T>>;
    fn graph_mut(&'a mut self) -> Self::GuardMut {
        self.graph.write().unwrap()
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> DirectedTraversalPolicy<'a, 'g, T, D> for Indexing<'a, T, D> {
    type Trav = Indexer<T, D>;
    type Folder = Indexer<T, D>;
    fn end_op(
        trav: &'a Self::Trav,
        query: QueryRangePath,
        start: StartPath,
    ) -> Vec<TraversalNode> {
        let mut ltrav = trav.clone();
        let IndexSplitResult {
            inner: post,
            location: entry,
            ..
            // should call leaf split and use known info of leaf position
         } = IndexableSide::<_, D, IndexBack>::index_entry_split(&mut ltrav, start.entry(), start.width());
        let start = StartPath::First { entry, child: post, width: start.width() };
        Self::parent_nodes(trav, query, Some(start))
    }
}
trait IndexingTraversalPolicy<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a>:
    DirectedTraversalPolicy<'a, 'g, T, D, Trav=Indexer<T, D>, Folder=Indexer<T, D>>
{ }
impl<'a: 'g, 'g, T: Tokenize, D: IndexDirection> IndexingTraversalPolicy<'a, 'g, T, D> for Indexing<'a, T, D> {}

impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> TraversalFolder<'a, 'g, T, D> for Indexer<T, D> {
    type Trav = Self;
    type Break = Child;
    type Continue = Option<Child>;
    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode,
    ) -> ControlFlow<Self::Break, Self::Continue> {
        let mut trav = trav.clone();
        match node {
            TraversalNode::End(Some(found)) => {
                ControlFlow::Break(Indexable::<_, D>::index_found(&mut trav, found.found))
            },
            TraversalNode::Mismatch(path) => {
                let found = path.reduce_mismatch::<_, _, D>(&trav);
                ControlFlow::Break(Indexable::<_, D>::index_found(&mut trav, found))
            },
            _ => ControlFlow::Continue(acc)
        }
    }
}
trait Indexable<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a>: TraversableMut<'a, 'g, T> {
    fn index_end_path(
        &'a mut self,
        end: EndPath,
    ) -> IndexSplitResult {
        let EndPath {
            mut path,
            entry,
            width,
        } = end;
        let mut graph = self.graph_mut();
        while let Some(entry) = path.pop() {
            IndexableSide::<_, D, IndexFront>::index_entry_split(
                &mut *graph,
                entry,
                width,
            );
        }
        IndexableSide::<_, D, IndexFront>::index_entry_split(
            &mut *graph,
            entry,
            width,
        )
    }
    fn index_range_path(
        &'a mut self,
        path: GraphRangePath,
    ) -> Child {
        let offset = path.width();
        let location@ChildLocation {
            parent,
            sub_index: entry,
            ..
        } = path.start.entry();
        let exit = path.exit;
        let (start, end) = path.into_paths();
        let mut graph = self.graph_mut();
        let pattern = start.pattern(&*graph);
        match (
            start.is_perfect(),
            DirectedGraphPath::<D>::is_at_pattern_border(&start, pattern.borrow()), 
            end.is_perfect(),
            DirectedGraphPath::<D>::is_at_pattern_border(&end, pattern.borrow()), 
        ) {
            //   start         end
            // perf comp    perf   comp
            (true, true, true, true) => panic!("GraphRangePath references complete index!"),
            // perfect back half split
            (true, _, true, true) =>
                IndexableSide::<_, D, IndexBack>::index_perfect_split(&mut *graph, location),
            // perfect front half split
            (true, true, true, _) =>
                IndexableSide::<_, D, IndexFront>::index_perfect_split(&mut *graph, end.entry()),
            // unperfect back half split
            (false, _, true, true) =>
                IndexableSide::<_, D, IndexBack>::index_offset_split(&mut *graph, parent, offset),
            // unperfect front half split
            (true, true, false, _) =>
                IndexableSide::<_, D, IndexFront>::index_offset_split(&mut *graph, parent, offset),
            // perfect/perfect inner split
            (true, _, true, _) =>
                Indexable::<_, D>::pattern_index_perfect_split_range(&mut *graph, pattern, location, entry..=exit),
            // perfect/unperfect inner split
            (true, false, false, _) =>
                IndexableSide::<_, D, IndexFront>::pattern_index_unperfect_split(&mut *graph, pattern, location, offset, <IndexBack as IndexSide<D>>::limited_range(entry, exit)),
            // unperfect/perfect inner split
            (false, _, true, false) =>
                IndexableSide::<_, D, IndexBack>::pattern_index_unperfect_split(&mut *graph, pattern, location, offset, <IndexBack as IndexSide<D>>::limited_range(entry, exit)),
            // unperfect/unperfect inner split
            (false, _, false, _) => {
                unreachable!()
                //let mut graph = self.graph_mut();
                //let child_patterns = graph.expect_children_of(parent).clone();
                //let len = child_patterns.len();
                //let positions = child_patterns.into_iter()
                //    .map(|(pid, pattern)| {
                //        let (index, offset) = D::token_offset_split(pattern.borrow(), offset).unwrap();
                //        (pid, pattern.into_pattern(), index, offset)
                //    })
                //    .collect_vec();
                //let (backs, fronts) = positions.into_iter()
                //    .map(|(_, pattern, pos, offset)| {
                //        let IndexSplitResult {
                //            inner,
                //            context,
                //            location,
                //        } = IndexableSide::<_, _, Side>::index_offset_split(&mut *graph, *pattern.get(pos).unwrap(), offset);
                //        let context = IndexableSide::<_, _, Side>::index_context_path(&mut *graph, location, context);
                //        let (back, front) = Side::context_inner_order(&context, &inner);
                //        (
                //            // todo: order depends on D
                //            [&D::back_context(pattern.borrow(), pos)[..], back].concat(),
                //            [front, &D::front_context(pattern.borrow(), pos)[..]].concat(),
                //        )
                //    }).unzip::<_, _, Vec<_>, Vec<_>>();
                //let (back, front) = (
                //    graph.index_patterns(backs),
                //    graph.index_patterns(fronts),
                //);
                //let pid = graph.add_pattern_with_update(parent, [back, front]);
                //let (inner, _) = Side::back_front_order(back, front);
                //let location = ChildLocation::new(parent, pid, 1);
                //IndexSplitResult {
                //    location,
                //    context: vec![],
                //    inner,
                //}
            },
        }.inner
    }
    fn index_found(
        &'a mut self,
        found: FoundPath,
    ) -> Child {
        match found {
            FoundPath::Range(path) => self.index_range_path(path),
            FoundPath::Complete(c) => c
        }
    }
    fn pattern_index_perfect_split_range(
        &'a mut self,
        pattern: Pattern,
        location: impl IntoPatternLocation,
        range: impl PatternRangeIndex + StartInclusive,
    ) -> IndexSplitResult {
        let inner = &pattern[range.clone()];
        let location = location.into_pattern_location().to_child_location(range.start());
        let inner = if inner.len() == 1 {
            *inner.into_iter().next().unwrap()
        } else {
            let mut graph = self.graph_mut();
            let inner = graph.index_pattern(inner);
            graph.replace_in_pattern(&location, range, [inner]);
            inner
        };
        IndexSplitResult {
            location,
            context: vec![],
            inner,
        }
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Trav: TraversableMut<'a, 'g, T>> Indexable<'a, 'g, T, D> for Trav {}

trait IndexableSide<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<D> + 'a>: Indexable<'a, 'g, T, D> {
    /// todo: a little bit dirty because width always points to a perfect split
    /// if the graph path segment it comes from is a leaf node
    fn index_entry_split(&'a mut self, entry: ChildLocation, width: usize) -> IndexSplitResult {
        let offset = Side::width_offset(&entry.parent, width);
        self.index_offset_split(entry.parent, offset)
    }
    fn index_perfect_split(&'a mut self, entry: ChildLocation) -> IndexSplitResult {
        let mut graph = self.graph_mut();
        let pattern = graph.expect_pattern_at(&entry);
        IndexableSide::<_, _, Side>::pattern_index_perfect_split(&mut *graph, pattern, entry)
    }
    fn pattern_index_perfect_split(&'a mut self, pattern: Pattern, entry: ChildLocation) -> IndexSplitResult {
        Self::pattern_index_perfect_split_range(self, pattern, entry, Side::inner_range(entry.sub_index))
    }
    fn index_context_path_segment(&'a mut self, location: ChildLocation) -> Child {
        let mut graph = self.graph_mut();
        let pattern = graph.expect_pattern_at(&location);
        let (back, front) = D::directed_pattern_split(pattern.borrow(), location.sub_index);
        let (_inner, context) = Side::back_front_order(back, front);
        let context = graph.index_pattern(context);
        graph.replace_in_pattern(location, Side::context_range(location.sub_index), context);
        context
    }
    fn index_context_path(&'a mut self, entry: ChildLocation, mut context_path: Vec<ChildLocation>) -> Child {
        let mut graph = self.graph_mut();
        let mut acc: Option<Child> = None;
        while let Some(location) = context_path.pop() {
            let context = IndexableSide::<_, _, Side>::index_context_path_segment(&mut *graph, location);
            if let Some(acc) = &mut acc {
                let (back, front) = Side::context_inner_order(&context, &acc);
                *acc = graph.index_pattern([back[0], front[0]]);
            } else {
                acc = Some(context);
            }
        }
        let context = IndexableSide::<_, _, Side>::index_context_path_segment(&mut *graph, entry);
        if let Some(acc) = acc {
            let (back, front) = Side::context_inner_order(&context, &acc);
            graph.index_pattern([back[0], front[0]])
        } else {
            context
        }
    }
    fn pattern_index_unperfect_split(&'a mut self, pattern: Pattern, location: impl IntoPatternLocation, offset: usize, range: Range<usize>) -> IndexSplitResult {
            let PatternLocation {
                parent,
                pattern_id: pid,
            } = location.into_pattern_location();
            let pos = Side::range_front(&range);
            let mut graph = self.graph_mut();
            let IndexSplitResult {
                inner,
                context,
                location,
            } = IndexableSide::<_, _, Side>::index_offset_split(&mut *graph, *pattern.get(pos).unwrap(), offset);
            let inner_context = IndexableSide::<_, _, Side>::index_context_path(&mut *graph, location, context);
            let (back, front) = Side::context_inner_order(&inner_context, &inner);
            let old = pattern.get(range.clone()).unwrap();
            let context_range = Side::limited_inner_range(&range);
            let front = graph.index_pattern([front, pattern.get(context_range).unwrap_or(&[])].concat());
            let new = [back[0], front];
            let (inner, ids) = graph.index_patterns_with_ids([&new, &old[..]]);
            let inner_pid = ids[0];
            graph.replace_in_pattern(location, range, inner);
            // todo: pos depends on Direction
            let location = ChildLocation::new(parent, pid, pos);
            IndexSplitResult {
                location,
                context: vec![ChildLocation::new(inner, inner_pid, 1)],
                inner,
            }
    }
    fn index_unperfect_splits(&'a mut self, parent: Child, positions: Vec<(PatternId, Pattern, usize, usize)>) -> IndexSplitResult {
        // todo: fix resulting locations, fix D order
        let mut graph = self.graph_mut();
        if positions.len() == 1 {
            let (pid, pattern, pos, offset) = positions.into_iter().next().unwrap();
            let range = Side::max_range(pattern.borrow(), pos);
            IndexableSide::<_, _, Side>::pattern_index_unperfect_split(&mut *graph, pattern, parent.to_pattern_location(pid), offset, range)
        } else {
            let (backs, fronts) = positions.into_iter()
                .map(|(_, pattern, pos, offset)| {
                    let IndexSplitResult {
                        inner,
                        context,
                        location,
                    } = IndexableSide::<_, _, Side>::index_offset_split(&mut *graph, *pattern.get(pos).unwrap(), offset);
                    let context = IndexableSide::<_, _, Side>::index_context_path(&mut *graph, location, context);
                    let (back, front) = Side::context_inner_order(&context, &inner);
                    (
                        // todo: order depends on D
                        [&D::back_context(pattern.borrow(), pos)[..], back].concat(),
                        [front, &D::front_context(pattern.borrow(), pos)[..]].concat(),
                    )
                }).unzip::<_, _, Vec<_>, Vec<_>>();
            let (back, front) = (
                graph.index_patterns(backs),
                graph.index_patterns(fronts),
            );
            let pid = graph.add_pattern_with_update(parent, [back, front]);
            let (inner, _) = Side::back_front_order(back, front);
            let location = ChildLocation::new(parent, pid, 1);
            IndexSplitResult {
                location,
                context: vec![],
                inner,
            }
        }
    }
    fn index_offset_split(&'a mut self, parent: Child, offset: usize) -> IndexSplitResult {
        let mut graph = self.graph_mut();
        let child_patterns = graph.expect_children_of(parent).clone();
        let len = child_patterns.len();
        let perfect = child_patterns.into_iter()
            .try_fold(Vec::with_capacity(len), |mut acc, (pid, pattern)| {
                let (index, offset) = D::token_offset_split(pattern.borrow(), offset).unwrap();
                if offset == 0 {
                    ControlFlow::Break((pattern.into_pattern(), pid, index))
                } else {
                    acc.push((pid, pattern.into_pattern(), index, offset));
                    ControlFlow::Continue(acc)
                }
            });
        match perfect {
            ControlFlow::Break((pattern, pid, pos)) =>
                IndexableSide::< _, _, Side>::pattern_index_perfect_split(&mut *graph, pattern, ChildLocation::new(parent, pid, pos)),
            ControlFlow::Continue(positions) =>
                IndexableSide::< _, _, Side>::index_unperfect_splits(&mut *graph, parent, positions),
        }
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Trav: Indexable<'a, 'g, T, D>, S: IndexSide<D> + 'a> IndexableSide<'a, 'g, T, D, S> for Trav {}

trait IndexSide<D: IndexDirection> {
    type Path: DirectedGraphPath<D>;
    type InnerRange: PatternRangeIndex + StartInclusive;
    type ContextRange: PatternRangeIndex + StartInclusive;
    fn width_offset(parent: &Child, width: usize) -> usize;
    /// returns inner, context
    fn back_front_order<A>(back: A, front: A) -> (A, A);
    /// returns back, front
    fn context_inner_order<
        'a,
        C: AsRef<[Child]> + 'a,
        I: AsRef<[Child]> + 'a
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]);
    fn inner_range(pos: usize) -> Self::InnerRange;
    fn context_range(pos: usize) -> Self::ContextRange;
    fn limited_range(start: usize, end: usize) -> Range<usize>;
    fn range_front(range: &Range<usize>) -> usize;
    fn limited_inner_range(range: &Range<usize>) -> Range<usize>;
    fn max_range(pattern: impl IntoPattern, pos: usize) -> Range<usize>;
}
struct IndexBack;
impl<D: IndexDirection> IndexSide<D> for IndexBack {
    type Path = StartPath;
    type InnerRange = RangeFrom<usize>;
    type ContextRange = Range<usize>;
    fn inner_range(pos: usize) -> Self::InnerRange {
        pos..
    }
    fn context_range(pos: usize) -> Self::ContextRange {
        0..pos
    }
    fn context_inner_order<
        'a,
        C: AsRef<[Child]> + 'a,
        I: AsRef<[Child]> + 'a
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]) {
        (context.as_ref(), inner.as_ref())
    }
    fn back_front_order<A>(back: A, front: A) -> (A, A) {
        (front, back)
    }
    fn width_offset(parent: &Child, width: usize) -> usize {
        // todo: changes with index direction
        parent.width() - width
    }
    fn limited_range(start: usize, end: usize) -> Range<usize> {
        end..start
    }
    fn limited_inner_range(range: &Range<usize>) -> Range<usize> {
        D::index_next(range.start()).unwrap()..range.end()
    }
    fn range_front(range: &Range<usize>) -> usize {
        range.end()
    }
    fn max_range(_pattern: impl IntoPattern, pos: usize) -> Range<usize> {
        0..pos
    }
}
struct IndexFront;
impl<D: IndexDirection> IndexSide<D> for IndexFront {
    type Path = EndPath;
    type InnerRange = RangeInclusive<usize>;
    type ContextRange = RangeFrom<usize>;
    fn inner_range(pos: usize) -> Self::InnerRange {
        0..=pos
    }
    fn context_range(pos: usize) -> Self::ContextRange {
        D::index_next(pos).unwrap()..
    }
    fn context_inner_order<
        'a,
        C: AsRef<[Child]> + 'a,
        I: AsRef<[Child]> + 'a
    >(context: &'a C, inner: &'a I) -> (&'a [Child], &'a [Child]) {
        (inner.as_ref(), context.as_ref())
    }
    fn back_front_order<A>(back: A, front: A) -> (A, A) {
        (back, front)
    }
    fn width_offset(_parent: &Child, width: usize) -> usize {
        width
    }
    fn limited_range(start: usize, end: usize) -> Range<usize> {
        start..end
    }
    fn range_front(range: &Range<usize>) -> usize {
        range.start()
    }
    fn limited_inner_range(range: &Range<usize>) -> Range<usize> {
        range.start()..D::index_prev(range.end()).unwrap()
    }
    fn max_range(pattern: impl IntoPattern, pos: usize) -> Range<usize> {
        pos..pattern.borrow().len()
    }
}

impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> Indexer<T, D> {
    pub fn new(graph: HypergraphRef<T>) -> Self {
        Self {
            graph,
            _ty: Default::default(),
        }
    }
    pub(crate) fn index_prefix(
        &mut self,
        pattern: impl IntoPattern,
    ) -> Result<Child, NoMatch> {
        self.indexing::<Bft<_, _, _, _>, Indexing<T, D>, _>(pattern)
    }
    //pub(crate) fn index_prefix_at(
    //    &mut self,
    //    location: &ChildLocation,
    //) -> Result<Child, NoMatch> {
    //    self.graph_mut().index_range_in(location.parent, location.pattern_id, 0..location.sub_index + 1)
    //}
    //pub(crate) fn index_postfix_at(
    //    &mut self,
    //    location: &ChildLocation,
    //) -> Result<Child, NoMatch> {
    //    self.graph_mut().index_range_in(location.parent, location.pattern_id, location.sub_index..)
    //}
    //pub(crate) fn index_pre_context_at(
    //    &mut self,
    //    location: &ChildLocation,
    //) -> Result<Child, NoMatch> {
    //    self.graph_mut().index_range_in(location.parent, location.pattern_id, 0..location.sub_index)
    //}
    //pub(crate) fn index_post_context_at(
    //    &mut self,
    //    location: &ChildLocation,
    //) -> Result<Child, NoMatch> {
    //    self.graph_mut().index_range_in(location.parent, location.pattern_id, location.sub_index + 1..)
    //}
    //pub(crate) fn index_split(
    //    &mut self,
    //    path: ChildPath,
    //) -> IndexedChild {
    //    path.into_iter().fold(None, |acc, location| {
    //        let context = self.index_pre_context_at(&location).ok();
    //        Some(if let Some(IndexedChild {
    //                context: prev_context,
    //                inner: prev_inner,
    //                ..
    //            }) = acc {
    //            let context = context.and_then(|context|
    //                prev_context.map(|prev_context|
    //                    self.graph_mut().insert_pattern([context, prev_context].as_slice())
    //                )
    //                .or(Some(context))
    //            )
    //            .or(prev_context);
    //            let inner = self.index_post_context_at(&location).expect("Invalid child location!");
    //            IndexedChild {
    //                context,
    //                inner: self.graph_mut().insert_pattern([prev_inner, inner]),
    //                location,
    //            }
    //        } else {
    //            IndexedChild {
    //                context,
    //                inner: self.index_postfix_at(&location).expect("Invalid child location!"),
    //                location,
    //            }
    //        })
    //    })
    //    .unwrap()
    //}
    /// creates an IndexingNode::Parent for each parent of root, extending its start path
    fn indexing<
        Ti: TraversalIterator<'a, 'g, T, Self, D, S>,
        S: IndexingTraversalPolicy<'a, 'g, T, D>,
        Q: IntoPattern,
    >(
        &'a mut self,
        query: Q,
    ) -> Result<Child, NoMatch> {
        let query = query.into_pattern();
        let query_path = QueryRangePath::new_directed::<D, _>(query.borrow())?;

        match Ti::new(self, TraversalNode::Query(query_path))
            .try_fold(None, |acc, (_, node)|
                S::Folder::fold_found(self, acc, node)
            )
        {
            ControlFlow::Continue(None) => Err(NoMatch::NotFound(query)),
            ControlFlow::Continue(Some(found)) => Ok(found),
            ControlFlow::Break(found) => Ok(found)
        }
    }
}