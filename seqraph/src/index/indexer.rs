use crate::{
    vertex::*,
    direction::*,
    index::*,
    Indexed,
    Hypergraph,
    search::*,
};
use std::{
    cmp::PartialEq,
    num::NonZeroUsize,
};

//pub type ChildSplits = (Vec<(SplitSegment, SplitSegment)>, Vec<(SplitSegment, SplitSegment)>);

#[derive(Debug)]
pub struct Indexer<'g, T: Tokenize> {
    graph: &'g mut Hypergraph<T>,
}
impl<'g, T: Tokenize + 'g> Indexer<'g, T> {
    pub fn new(graph: &'g mut Hypergraph<T>) -> Self {
        Self { graph }
    }
    pub(crate) fn index_found(
        &mut self,
        found_path: FoundPath,
    ) -> (Child, Pattern) {
        let FoundPath {
                root,
                start_path,
                end_path,
                remainder,
            } = found_path;
        (match parent_match.parent_range {
                FoundRange::Complete => {
                    //println!("Found full index {}", self.graph.index_string(&index));
                    index
                }
                FoundRange::Prefix(post) => {
                    //println!("Found prefix of {} width {}", self.graph.index_string(&index), index.width);
                    let width = pattern_width(&post);
                    //println!("postfix {} width {}", self.graph.pattern_string(&post), width);
                    //println!("{:#?}", &post);
                    let width = index.width - width;
                    let pos = NonZeroUsize::new(width)
                        .expect("returned full length postfix remainder");
                    //println!("prefix width {}", pos.get());
                    let (l, _) = self.index_prefix(index, pos);
                    l
                }
                FoundRange::Postfix(pre) => {
                    //println!("Found postfix of {}", self.graph.index_string(&index));
                    //println!("prefix {}", self.graph.pattern_string(&pre));
                    let width = pattern_width(pre);
                    let pos = NonZeroUsize::new(width)
                        .expect("returned zero length prefix remainder");
                    let (_, r) = self.index_postfix(index, pos);
                    r
                }
                FoundRange::Infix(pre, post) => {
                    //println!("Found infix of {}", self.graph.index_string(&index));
                    //println!("postfix {}", self.graph.pattern_string(&post));
                    //println!("prefix {}", self.graph.pattern_string(&pre));
                    let pre_width = pattern_width(pre);
                    let post_width = pattern_width(post);
                    //println!("{}, {}, {}", pre_width, post_width, index.width);
                    //println!("{}", self.index_string(index));
                    self.index_subrange(index, pre_width..index.width - post_width).unwrap_child()
                }
            }, parent_match.remainder.unwrap_or_default())
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