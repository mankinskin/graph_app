use crate::{
    split::*,
    Indexed,
};
use std::num::NonZeroUsize;
impl<'g, T: Tokenize + 'g> IndexSplitter<'g, T> {
    pub(crate) fn index_subrange(
        &mut self,
        root: impl Indexed + Clone,
        range: impl PatternRangeIndex,
    ) -> RangeSplitResult {
        let root = root.index();
        //println!("splitting {} at {:?}", hypergraph.index_string(root), range);
        let vertex = self.graph.expect_vertex_data(root).clone();
        // range is a subrange of the index
        let patterns = vertex.get_children().clone();
        match SplitIndices::verify_range_split_indices(vertex.width, range) {
            DoubleSplitPositions::Double(lower, higher) =>
                self.process_double_splits(root, vertex, patterns, lower, higher),
            DoubleSplitPositions::Single(single) =>
                self.single_split_patterns(root, patterns, single).1.into(),
            DoubleSplitPositions::None => RangeSplitResult::Full(Child::new(root, vertex.width)),
        }
    }
    fn process_double_splits(
        &mut self,
        root: impl Indexed,
        vertex: VertexData,
        patterns: ChildPatterns,
        lower: NonZeroUsize,
        higher: NonZeroUsize,
    ) -> RangeSplitResult {
        // both positions in position in pattern
        match SplitIndices::build_double(&vertex, patterns, lower, higher) {
            Ok((_pattern_id, pre, _left, inner, _right, post)) => {
                (pre.into(), inner.into(), post.into())
            }
            Err(indices) => {
                // unperfect splits
                let cap = indices.len();
                let (left, inner, right) = indices
                    .into_iter()
                    .fold(
                        (Vec::with_capacity(cap), Vec::with_capacity(cap), Vec::with_capacity(cap)),
                        |(mut la, mut ia, mut ra), (_pattern_id, split_index)| {
                            match split_index {
                                DoubleSplitIndex::Left(pre, _, infix, single, post) => {
                                    let (l, r) = self.split_index(single.index, single.offset);
                                    la.push((pre, None));
                                    ia.push((None, SplitSegment::Pattern(infix), Some(l)));
                                    ra.push((post, Some(r)));
                                }
                                DoubleSplitIndex::Right(pre, single, infix, _, post) => {
                                    let (l, r) = self.split_index(single.index, single.offset);
                                    la.push((pre, Some(l)));
                                    ia.push((Some(r), SplitSegment::Pattern(infix), None));
                                    ra.push((post, None));
                                }
                                DoubleSplitIndex::Infix(pre, left, infix, right, post) => {
                                    let (ll, lr) = self.split_index(left.index, left.offset);
                                    let (rl, rr) = self.split_index(right.index, right.offset);
                                    la.push((pre, Some(ll)));
                                    ia.push((Some(lr), SplitSegment::Pattern(infix), Some(rl)));
                                    ra.push((post, Some(rr)));
                                }
                                DoubleSplitIndex::Inner(pre, (index, left, right), post) => {
                                    match self.index_subrange(index, left.get()..right.get()) {
                                        RangeSplitResult::Double(l, i, r) => {
                                            la.push((pre, Some(l)));
                                            ia.push((None, i, None));
                                            ra.push((post, Some(r)));
                                        }
                                        RangeSplitResult::Single(l, r) => {
                                            la.push((pre, Some(l)));
                                            ra.push((post, Some(r)));
                                        }
                                        RangeSplitResult::Full(c) => {
                                            la.push((pre, None));
                                            ia.push((None, SplitSegment::Child(c), None));
                                            ra.push((post, None));
                                        }
                                    }
                                }
                            }
                            (la, ia, ra)
                        },
                    );
                let left = self.graph.merge_left_optional_splits(left);
                let inner = self.graph.merge_inner_optional_splits(inner);
                let right = self.graph.merge_right_optional_splits(right);
                self.graph.add_pattern_to_node(
                    root,
                    left.clone().into_iter().chain(inner).chain(right.clone()),
                );
                (left, SplitSegment::Child(inner), right)
            }
        }.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn split_range_1() {
        let mut graph = Hypergraph::default();
        if let [a, b, w, x, y, z] = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ])[..]
        {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let by = graph.insert_pattern([b, y]);
            let yz = graph.insert_pattern([y, z]);
            let wx = graph.insert_pattern([w, x]);
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([vec![xab, y], vec![x, a, by]]);
            let wxab = graph.insert_patterns([vec![wx, ab], vec![w, xab]]);
            let wxaby = graph.insert_patterns([vec![w, xaby], vec![wx, a, by], vec![wxab, y]]);
            let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);
            let wxabyz = graph.insert_patterns([vec![w, xabyz], vec![wxaby, z], vec![wx, ab, yz]]);

            let _ = graph.index_subrange(wxabyz, 0..);
            let _ = graph.index_subrange(wxabyz, 1..);
            let _ = graph.index_subrange(wxabyz, 1..3);
            let _ = graph.index_subrange(wxabyz, 2..5);
            let _ = graph.index_subrange(wxabyz, 3..);
        } else {
            panic!();
        }
    }
}
