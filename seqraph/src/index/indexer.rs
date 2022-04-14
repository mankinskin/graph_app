use std::{sync::{
    RwLockReadGuard,
    RwLockWriteGuard,
}, ops::{ControlFlow, RangeBounds, RangeTo, Range}, borrow::Borrow};

use itertools::Itertools;

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
trait IndexingTraversalPolicy<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a>:
    DirectedTraversalPolicy<'a, 'g, T, D, Trav=Indexer<T, D>, Folder=Indexer<T, D>>
{}
impl<'a: 'g, 'g, T: Tokenize, D: IndexDirection> IndexingTraversalPolicy<'a, 'g, T, D> for Indexing<'a, T, D> {}

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
         } = Indexable::<_, D, IndexBack>::index_entry_split(&mut ltrav, start.entry(), start.width());
        let start = StartPath::First { entry, child: post, width: start.width() };
        Self::parent_nodes(trav, query, Some(start))
    }
}
fn index_end_path<T: Tokenize, D: IndexDirection>(
    trav: &Indexer<T, D>,
    end: EndPath,
) -> IndexSplitResult {
    let mut ltrav = trav.clone();
    let EndPath {
        mut path,
        entry,
        width,
    } = end;
    while let Some(entry) = path.pop() {
        Indexable::<_, D, IndexFront>::index_entry_split(
            &mut ltrav,
            entry,
            width,
        );
    }
    Indexable::<_, D, IndexFront>::index_entry_split(
        &mut ltrav,
        entry,
        width,
    )
}
fn index_range_path<T: Tokenize, D: IndexDirection>(
    trav: &Indexer<T, D>,
    path: GraphRangePath,
) -> Child {
    let mut ltrav = trav.clone();
    let ChildLocation {
        parent,
        pattern_id,
        sub_index: entry,
    } = path.start.entry();
    let exit = path.exit;
    let (start, end) = path.into_paths();
    let pattern = start.pattern(&ltrav);

    return parent;
    unreachable!();
    //match (
    //    start.is_perfect(),
    //    start.is_at_pattern_border(pattern), 
    //    end.is_perfect(),
    //    end.is_at_pattern_border(pattern)
    //) {
    //    //   start         end
    //    // perf comp    perf   comp
    //    (true, true, true, true) => panic!("GraphRangePath references complete index!"),
    //    (true, false, false, true) => {
    //        // index inner
    //        // index context
    //        // index target range
    //        // replace contexts and inner in old pattern
    //        // create new pattern

    //        let IndexSplitResult {
    //            inner: back,
    //            context,
    //            ..
    //        } = IndexBack::index_entry_split(&mut ltrav, start.entry(), start.width());
    //        let inner = &pattern[entry+1..exit];
    //        let inner = match inner.len() {
    //            0 => panic!("GraphPath references range in child index"),
    //            _ => trav.graph_mut().insert_pattern(inner),
    //        };
    //        let IndexSplitResult {
    //            inner: front,
    //            context: front_context,
    //            ..
    //        } = index_end_path(&mut ltrav, end);
    //        let inner = [Some(back), inner, Some(front)].into_iter().flatten().collect_vec();
    //        let inner = trav.graph_mut().index_pattern(inner);
    //        trav.graph_mut().expect_vertex_data_mut(parent).add_pattern_with_update([
    //            back_context.unwrap_child(), inner, front_context.unwrap_child()
    //    },
    //    (false, false, false, false) => {
    //        // index inner
    //        // index context
    //        // index target range
    //        // replace contexts and inner in old pattern
    //        // create new pattern
    //    },
    //    // both perfect
    //    (true, _, true, _) => {
    //        // replace range
    //        //pattern[entry..exit].into_pattern()
    //    },
    //    // one perfect
    //    (false, _, true, _) => {
    //        // create index for inner and context
    //        // replace in old pattern
    //        // create index for target range and its context
    //        // create new index from old pattern half and new indexed range
    //        // replace index in old pattern
    //    },
    //    (true, _, false, _) => {
    //    }
    //    (false, _, false, _) => {
    //    },
    //}
    //let start_half = if start.is_perfect() {
    //} else {
    //    let pre_context = D::back_context(&pattern, entry);
    //    let pre_context = match pre_context.len() {
    //        0 => None,
    //        1 => Some(pre_context.into_iter().next().unwrap()),
    //        _ => Some(ltrav.graph_mut().insert_pattern(pre_context)),
    //    };
    //    PathRootHalf::Unperfect(pre_context, context, back)
    //};
    //let front = if end.is_perfect() {
    //    match start_half {
    //        PathRootHalf::Perfect(back) => {
    //            let mut graph = ltrav.graph_mut();
    //            let inner = graph.insert_pattern([back.as_slice(), &[pattern[exit]]].concat());
    //            graph.replace_in_pattern(start.entry(), entry..=exit, inner);
    //            inner
    //        },
    //        PathRootHalf::Unperfect(_context, back) => {
    //            start.pattern(ltrav)[exit]
    //        }
    //    }
    //} else {
    //    match start_half {
    //        PathRootHalf::Perfect(back) => {
    //        },
    //        PathRootHalf::Unperfect(back_context, back) => {
    //            let inner = if D::index_next(&pattern[..], entry).unwrap() == exit {
    //                None
    //            } else {
    //                let mut graph = ltrav.graph_mut();
    //                let inner = graph.insert_pattern(&pattern[entry+1..exit]);
    //                graph.replace_in_pattern(start.entry(), entry+1..exit, inner);
    //                Some(inner)
    //            };
    //            let post_context = D::front_context(&pattern, exit);
    //            let post_context = match post_context.len() {
    //                0 => None,
    //                1 => Some(post_context.into_iter().next().unwrap()),
    //                _ => Some(ltrav.graph_mut().insert_pattern(post_context)),
    //            };
    //        }
    //    }
    //    front
    //};
    unimplemented!();
}
fn index_found<T: Tokenize, D: IndexDirection>(
    trav: &Indexer<T, D>,
    found: FoundPath,
) -> Child {
    match found {
        FoundPath::Range(path) => index_range_path(trav, path),
        FoundPath::Complete(c) => c
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a> TraversalFolder<'a, 'g, T, D> for Indexer<T, D> {
    type Trav = Self;
    type Break = Child;
    type Continue = Option<Child>;
    fn fold_found(
        trav: &Self::Trav,
        acc: Self::Continue,
        node: TraversalNode,
    ) -> ControlFlow<Self::Break, Self::Continue> {
        match node {
            TraversalNode::End(Some(found)) => {
                ControlFlow::Break(index_found(trav, found.found))
            },
            TraversalNode::Mismatch(path) => {
                let found = path.reduce_mismatch::<_, _, D>(trav);
                ControlFlow::Break(index_found(trav, found))
            },
            _ => ControlFlow::Continue(acc)
        }
    }
}
trait Indexable<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Side: IndexSide<T, D> + 'a>: TraversableMut<'a, 'g, T> {
    /// todo: a little bit dirty because width always points to a perfect split
    /// if the graph path segment it comes from is a leaf node
    fn index_entry_split(&'a mut self, entry: ChildLocation, width: usize) -> IndexSplitResult {
        let offset = Side::width_offset(&entry.parent, width);
        self.index_offset_split(entry.parent, offset)
    }
    fn index_perfect_split(&'a mut self, entry: ChildLocation) -> IndexSplitResult {
        let mut graph = self.graph_mut();
        let pattern = graph.expect_pattern_at(&entry);
        Indexable::<_, _, Side>::pattern_index_perfect_split(&mut *graph, pattern, entry)
    }
    fn pattern_index_perfect_split(&'a mut self, pattern: Pattern, mut entry: ChildLocation) -> IndexSplitResult {
        let pos = entry.sub_index;
        let (back, front) = D::directed_pattern_split(&pattern[..], pos);
        let (inner, context) = Side::back_front_order(back, front);
        let inner = if inner.len() == 1 {
            inner.into_iter().next().unwrap()
        } else {
            let mut graph = self.graph_mut();
            let inner = graph.index_pattern(inner);
            let range = Side::inner_range(pos);
            let new_pos = range.start();
            graph.replace_in_pattern(&entry, range, [inner]);
            entry.sub_index = new_pos;
            inner
        };
        let context = ContextHalf::try_new(context).expect("GraphRangePath references border of index!");
        IndexSplitResult {
            location: entry,
            context: vec![entry],
            inner,
        }
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
            let context = Indexable::<_, _, Side>::index_context_path_segment(&mut *graph, location);
            if let Some(mut acc) = &mut acc {
                let (back, front) = Side::context_inner_order(&context, &acc);
                acc = graph.index_pattern([back[0], front[0]]);
            } else {
                acc = Some(context);
            }
        }
        let context = Indexable::<_, _, Side>::index_context_path_segment(&mut *graph, entry);
        if let Some(acc) = acc {
            let (back, front) = Side::context_inner_order(&context, &acc);
            graph.index_pattern([back[0], front[0]])
        } else {
            context
        }
    }
    fn index_unperfect_splits(&'a mut self, parent: Child, positions: Vec<(Pattern, usize, usize)>) -> IndexSplitResult {
        let mut graph = self.graph_mut();
        let (backs, fronts) = positions.into_iter()
            .map(|(pattern, pos, offset)| {
                let IndexSplitResult {
                    inner,
                    context,
                    location,
                } = Indexable::<_, _, Side>::index_offset_split(&mut *graph, *pattern.get(pos).unwrap(), offset);
                let context = Indexable::<_, _, Side>::index_context_path(&mut *graph, location, context);
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
        let (inner, context) = Side::back_front_order(back, front);
        let location = ChildLocation::new(parent, pid, 1);
        IndexSplitResult {
            location,
            context: vec![],
            inner,
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
                    acc.push((pattern.into_pattern(), index, offset));
                    ControlFlow::Continue(acc)
                }
            });
        match perfect {
            ControlFlow::Break((pattern, pid, pos)) =>
                Indexable::< _, _, Side>::pattern_index_perfect_split(&mut *graph, pattern, ChildLocation::new(parent, pid, pos)),
            ControlFlow::Continue(positions) =>
                Indexable::< _, _, Side>::index_unperfect_splits(&mut *graph, parent, positions),
        }
    }
}
impl<'a: 'g, 'g, T: Tokenize + 'a, D: IndexDirection + 'a, Trav: TraversableMut<'a, 'g, T>, S: IndexSide<T, D> + 'a> Indexable<'a, 'g, T, D, S> for Trav {
}
trait IndexSide<T: Tokenize, D: IndexDirection> {
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
}
struct IndexBack;
impl<T: Tokenize, D: IndexDirection> IndexSide<T, D> for IndexBack {
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
}
struct IndexFront;
impl<T: Tokenize, D: IndexDirection> IndexSide<T, D> for IndexFront {
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