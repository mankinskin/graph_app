use crate::{
    vertex::*,
    r#match::*,
    index::*,
    Hypergraph,
};

#[derive(Debug)]
pub struct Indexer<'g, T: Tokenize> {
    graph: &'g mut Hypergraph<T>,
}
impl<'a, T: Tokenize> std::ops::Deref for Indexer<'a, T> {
    type Target = Hypergraph<T>;
    fn deref(&self) -> &Self::Target {
        self.graph
    }
}
impl<'a, T: Tokenize> std::ops::DerefMut for Indexer<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.graph
    }
}
impl<'g, T: Tokenize + 'g> Indexer<'g, T> {
    pub fn new(graph: &'g mut Hypergraph<T>) -> Self {
        Self { graph }
    }
    pub(crate) fn index_found(
        &mut self,
        found_path: FoundPath,
    ) -> (Option<Child>, Child, Option<Child>, Pattern) {
        let FoundPath {
                root,
                start_path,
                end_path,
                remainder,
        } = found_path;
        let left = start_path.map(|start_path| {
            let mut start_path = start_path.into_iter();
            let location = start_path.next().unwrap();
            let inner = self.index_postfix_at(&location).unwrap();
            start_path
                .fold((None, inner, location), |(context, inner, prev_location), location| {
                    let context = context.unwrap_or_else(||
                        self.index_pre_context_at(&prev_location).unwrap()
                    );
                    let prefix = self.index_pre_context_at(&location).unwrap();
                    let context = self.insert_pattern([prefix, context]);
                    let inner = self.index_post_context_at(&location).map(|postfix|
                        self.insert_pattern([inner, postfix])
                    ).unwrap_or(inner);
                    (Some(context), inner, location)
                })
        });
        let right = end_path.map(|end_path| {
            let mut end_path = end_path.into_iter().rev();
            let location = end_path.next().unwrap();
            let inner = self.index_prefix_at(&location).unwrap();
            end_path
                .fold((inner, None, location), |(inner, context, prev_location), location| {
                    let context = context.unwrap_or_else(||
                        self.index_post_context_at(&prev_location).unwrap()
                    );
                    let postfix = self.index_post_context_at(&location).unwrap();
                    let context = self.insert_pattern([context, postfix]);
                    let inner = self.index_pre_context_at(&location).map(|pre|
                        self.insert_pattern([pre, inner])
                    ).unwrap_or(inner);
                    (inner, Some(context), location)
                })
        });
        let (lctx, inner, rctx) = match (left, right) {
            (None, None) => (None, root, None),
            (Some((lcontext, linner, _)), Some((rinner, rcontext, _))) => {
                let inner = self.insert_pattern([linner, rinner].as_slice());
                let root = root.vertex_mut(self);
                match (lcontext, rcontext) {
                    (Some(lctx), Some(rctx)) => {
                        root.add_pattern([lctx, inner, rctx].as_slice());
                    }
                    (Some(lctx), None) => {
                        root.add_pattern([lctx, inner].as_slice());
                    }
                    (None, Some(rctx)) => {
                        root.add_pattern([inner, rctx].as_slice());
                    }
                    (None, None) => unreachable!(),
                };
                (lcontext, inner, rcontext)
            },
            (Some((lcontext, linner, _)), None) => {
                let inner = if let Some(lctx) = lcontext {
                    let root = root.vertex_mut(self);
                    root.add_pattern([lctx, linner].as_slice());
                    linner
                } else {
                    linner
                };
                (lcontext, inner, None)
            }
            (None, Some((rinner, rcontext, _))) => {
                let inner = if let Some(rctx) = rcontext {
                    let root = root.vertex_mut(self);
                    root.add_pattern([rinner, rctx].as_slice());
                    rinner
                } else {
                    rinner
                };
                (None, inner, rcontext)
            }
        };
        (lctx, inner, rctx, remainder.unwrap_or_default())
    }
    /// includes location
    pub(crate) fn index_prefix_at(
        &mut self,
        location: &ChildLocation,
    ) -> Option<Child> {
        self.index_range_in(location.parent, location.pattern_id, 0..location.sub_index + 1)
    }
    /// includes location
    pub(crate) fn index_postfix_at(
        &mut self,
        location: &ChildLocation,
    ) -> Option<Child> {
        self.index_range_in(location.parent, location.pattern_id, location.sub_index..)
    }
    /// does not include location
    pub(crate) fn index_pre_context_at(
        &mut self,
        location: &ChildLocation,
    ) -> Option<Child> {
        self.index_range_in(location.parent, location.pattern_id, 0..location.sub_index)
    }
    /// does not include location
    pub(crate) fn index_post_context_at(
        &mut self,
        location: &ChildLocation,
    ) -> Option<Child> {
        self.index_range_in(location.parent, location.pattern_id, location.sub_index + 1..)
    }
    //pub(crate) fn index_single_split<D: IndexSide, R: AsChild>(
    //    &mut self,
    //    root: R,
    //    pos: NonZeroUsize,
    //) -> D::IndexResult {
    //    let vertex = self.graph.expect_vertex_data(&root);
    //    let patterns = vertex.get_children().clone();
    //    //println!("splitting {} at {}", self.index_string(root), pos);
    //    self.index_single_split_patterns::<D, R>(root, patterns, pos)
    //}
    //pub(crate) fn index_single_split_patterns<D: IndexSide, R: AsChild>(
    //    &mut self,
    //    root: R,
    //    patterns: ChildPatterns,
    //    pos: NonZeroUsize,
    //) -> D::IndexResult {
    //    let (perfect_split, remaining_splits)
    //        = SplitIndices::find_perfect_split(patterns, pos);
    //    // check if any perfect split segment is a single child
    //    if let Some((l, r, ind)) = perfect_split {
    //        let loc = PatternLocation::new(root.as_child(), ind.pattern_index);
    //        if (1, 1) == (l.len(), r.len()) {
    //            // early return when complete split present
    //            D::trivial(l.pop().unwrap(), r.pop().unwrap())
    //        } else {
    //            // at least one side not complete child, replace in root
    //            D::index_and_replace(self.graph, loc, l, r)
    //        }
    //    } else {
    //        // no perfect split
    //        // perform inner splits
    //        let (left, right) = remaining_splits
    //            .into_iter()
    //            .map(
    //                |SplitContext {
    //                     prefix,
    //                     key,
    //                     postfix,
    //                     loc,
    //                 }| {
    //                    let (l, r) = self.graph.index_split(key.index, key.offset);
    //                    ((SplitSegment::Pattern(prefix, loc.clone()), l), (SplitSegment::Pattern(postfix, loc), r))
    //                },
    //            )
    //            .unzip();
    //        let (left, right) = (
    //            self.graph.merge_left_splits(left),
    //            self.graph.merge_right_splits(right),
    //        );
    //        D::index_and_add(self.graph, root, left, right)
    //    }
    //}
    //pub(crate) fn index_subrange(
    //    &mut self,
    //    root: impl AsChild,
    //    range: impl PatternRangeIndex,
    //) -> IndexRangeResult {
    //    //println!("splitting {} at {:?}", hypergraph.index_string(root), range);
    //    let vertex = self.graph.expect_vertex_data(root.index()).clone();
    //    // range is a subrange of the index
    //    let patterns = vertex.get_children().clone();
    //    match SplitIndices::verify_range_split_indices(vertex.width, range) {
    //        DoubleSplitPositions::Double(lower, higher) =>
    //            self.process_double_splits(root, vertex, patterns, lower, higher),
    //        DoubleSplitPositions::SinglePrefix(single) =>
    //            self.index_single_split_patterns::<Left, _>(vertex, patterns, single).into(),
    //        DoubleSplitPositions::SinglePostfix(single) =>
    //            self.index_single_split_patterns::<Right, _>(vertex, patterns, single).into(),
    //        DoubleSplitPositions::None => IndexRangeResult::Full(root.as_child()),
    //    }
    //}
    //fn process_double_splits(
    //    &mut self,
    //    root: impl Indexed,
    //    vertex: VertexData,
    //    patterns: ChildPatterns,
    //    lower: NonZeroUsize,
    //    higher: NonZeroUsize,
    //) -> IndexRangeResult {
    //    // both positions in position in pattern
    //    match SplitIndices::build_double(&vertex, patterns, lower, higher) {
    //        Ok((pid, pre, left, _inner, right, post)) => {
    //            // two perfect splits
    //            let inner = self.graph.index_range_in(root, pid, left..right);
    //            let loc = PatternLocation::new(vertex.as_child(), pid);
    //            let pre = SplitSegment::with_location(pre, loc.clone());
    //            let post = SplitSegment::with_location(post, loc);
    //            IndexRangeResult::Infix(pre, inner, post)
    //        }
    //        Err(indices) => {
    //            // unperfect splits
    //            let cap = indices.len();
    //            let (left, inner, right) = indices
    //                .into_iter()
    //                .fold(
    //                    (Vec::with_capacity(cap), Vec::with_capacity(cap), Vec::with_capacity(cap)),
    //                    |(
    //                        mut la,
    //                        mut ia,
    //                        mut ra,
    //                    ),
    //                    (
    //                        pid,
    //                        split_index,
    //                    )| {
    //                        let loc = PatternLocation::new(vertex.as_child(), pid);
    //                        match split_index {
    //                            DoubleSplitIndex::Left(pre, _, infix, single, post) => {
    //                                // perfect split on left
    //                                let (l, r) = self.graph.index_split(single.index, single.offset);
    //                                la.push((pre, None));
    //                                ia.push((None, SplitSegment::Pattern(infix, loc), Some(l)));
    //                                ra.push((post, Some(r)));
    //                            }
    //                            DoubleSplitIndex::Right(pre, single, infix, _, post) => {
    //                                // perfect split on right
    //                                let (l, r) = self.graph.index_postfix(single.index, single.offset);
    //                                la.push((pre, Some(l)));
    //                                ia.push((Some(r), SplitSegment::Pattern(infix, loc), None));
    //                                ra.push((post, None));
    //                            }
    //                            DoubleSplitIndex::Infix(pre, left, infix, right, post) => {
    //                                // no perfect split
    //                                let (ll, lr) = self.graph.index_postfix(left.index, left.offset);
    //                                let (rl, rr) = self.graph.index_prefix(right.index, right.offset);
    //                                la.push((pre, Some(ll)));
    //                                ia.push((Some(lr), SplitSegment::Pattern(infix, loc), Some(rl)));
    //                                ra.push((post, Some(rr)));
    //                            }
    //                            DoubleSplitIndex::Inner(pre, (index, left, right), post) => {
    //                                match self.graph.index_subrange(index, left.get()..right.get()) {
    //                                    IndexRangeResult::Infix(l, i, r) => {
    //                                        la.push((pre, Some(l)));
    //                                        ia.push((None, i, None));
    //                                        ra.push((post, Some(r)));
    //                                    }
    //                                    IndexRangeResult::Prefix(l, r) => {
    //                                        la.push((pre, Some(l)));
    //                                        ra.push((post, Some(r)));
    //                                    }
    //                                    IndexRangeResult::Postfix(l, r) => {
    //                                        la.push((pre, Some(l)));
    //                                        ra.push((post, Some(r)));
    //                                    }
    //                                    IndexRangeResult::Full(c) => {
    //                                        la.push((pre, None));
    //                                        ia.push((None, SplitSegment::Child(c), None));
    //                                        ra.push((post, None));
    //                                    }
    //                                }
    //                            }
    //                        }
    //                        (la, ia, ra)
    //                    },
    //                );
    //            let left = self.graph.merge_left_optional_splits(left);
    //            let inner = self.graph.merge_inner_optional_splits(inner);
    //            let right = self.graph.merge_right_optional_splits(right);
    //            self.graph.add_pattern_to_node(
    //                root,
    //                left.clone().into_iter().chain(inner).chain(right.clone()),
    //            );
    //            (left, inner, right)
    //        }
    //    }
    //}
}