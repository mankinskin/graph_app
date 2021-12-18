use crate::{
    merge::*,
    split::*,
};
use std::{
    num::NonZeroUsize,
};
type ChildSplits = (Vec<(Pattern, SplitSegment)>, Vec<(Pattern, SplitSegment)>);
impl<'g, T: Tokenize + 'g> IndexSplitter<'g, T> {
    pub(crate) fn split_index_with_pid(
        &mut self,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> (Option<PatternId>, SingleSplitResult) {
        let root = root.index();
        let vertex = self.graph.expect_vertex_data(root);
        let patterns = vertex.get_children().clone();
        //println!("splitting {} at {}", self.index_string(root), pos);
        self.single_split_patterns(root, patterns, pos)
    }
    pub(crate) fn with_perfect_split(
        p: Pattern,
        splits: Vec<(Pattern, SplitSegment)>
    ) -> Vec<(Pattern, Option<SplitSegment>)> {
        splits
            .into_iter()
            .map(|(c, i)| (c, Some(i)))
            .chain(Some((p, None)))
            .collect()
    }
    pub(crate) fn single_split_patterns(
        &mut self,
        root: impl Indexed,
        patterns: ChildPatterns,
        pos: NonZeroUsize,
    ) -> (Option<PatternId>, SingleSplitResult) {
        let (perfect_split, remaining_splits)
            = SplitIndices::find_perfect_split(patterns, pos);

        // check if any perfect split segment is a single child 
        let (left, right) = if let Some(((pl, pr), ind)) = perfect_split {
            let (pl, pr): (SplitSegment, SplitSegment) = (pl.into(), pr.into());
            match (pl, pr) {
                // early return when complete split present
                (lc@SplitSegment::Child(_), rc@SplitSegment::Child(_)) => (lc, rc),
                // at least one side not complete child
                (SplitSegment::Pattern(pl), SplitSegment::Pattern(pr)) => {
                    let (left, right)
                        = self.split_inner_indices(remaining_splits);

                    let mut minimizer = SplitMinimizer::new(self.graph);
                    let left = Self::with_perfect_split(pl, left);
                    let left = minimizer.merge_left_optional_splits(left);
                    let right = Self::with_perfect_split(pr, right);
                    let right = minimizer.merge_right_optional_splits(right);

                    self.graph
                        .add_pattern_to_node(root, left.clone().into_iter().chain(right.clone()));
                    (left, right)
                },
                (lc@SplitSegment::Child(_), SplitSegment::Pattern(pr)) => {
                    let (_, right)
                        = self.split_inner_indices(remaining_splits);

                    let mut minimizer = SplitMinimizer::new(self.graph);
                    let right = Self::with_perfect_split(pr, right);
                    let right = minimizer.merge_right_optional_splits(right);

                    self.graph
                        .replace_in_pattern(root, ind.pattern_index, 1.., right.clone());
                    (lc, right)
                }
                (SplitSegment::Pattern(pl), rc@SplitSegment::Child(_)) => {
                    let (left, _)
                        = self.split_inner_indices(remaining_splits);

                    let mut minimizer = SplitMinimizer::new(self.graph);
                    let left = Self::with_perfect_split(pl, left);
                    let left = minimizer.merge_right_optional_splits(left);

                    let end = self.graph
                        .expect_vertex_data(&root).expect_child_pattern(&ind.pattern_index).len();
                    self.graph
                        .replace_in_pattern(root, ind.pattern_index, 0..end-1, left.clone());
                    (left, rc)
                }
            }
        } else {
            // no perfect split
            let (left, right)
                = self.split_inner_indices(remaining_splits);
            let mut minimizer = SplitMinimizer::new(self.graph);
            let (left, right ) = (
                minimizer.merge_left_splits(left),
                minimizer.merge_right_splits(right),
            );
            self.graph
                .add_pattern_to_node(root, left.clone().into_iter().chain(right.clone()));
            (left, right)
        };
        (None, (left, right))
    }
    pub(crate) fn split_inner_indices(
        &mut self,
        child_splits: Vec<SplitContext>,
    ) -> ChildSplits {
        child_splits
            .into_iter()
            .map(
                |SplitContext {
                     prefix,
                     key,
                     postfix,
                 }| {
                    // recurse
                    let (l, r) = self.split_index(key.index, key.offset);
                    ((prefix, l), (postfix, r))
                },
            )
            .unzip()
    }
    #[cfg(test)]
    pub(crate) fn get_perfect_split_separation(
        &mut self,
        root: impl Indexed,
        pos: NonZeroUsize,
    ) -> (Option<(Split, IndexInParent)>, Vec<SplitContext>) {
        let root = root.index();
        let vertex = self.graph.expect_vertex_data(root).clone();
        let patterns = vertex.get_children().clone();
        //println!("splitting {} at {}", self.index_string(root), pos);
        SplitIndices::find_perfect_split(patterns, pos)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph::tests::*,
        search::*,
    };
    use pretty_assertions::assert_eq;
    use itertools::*;
    #[test]
    fn split_index_1() {
        let (
            graph,
            _a,
            _b,
            c,
            _d,
            _e,
            _f,
            _g,
            _h,
            _i,
            ab,
            _bc,
            _cd,
            _bcd,
            abc,
            _abcd,
            _ef,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            _ababababcdefghi,
        ) = &mut *context_mut();
        let (left, right) = graph.split_index(*abc, NonZeroUsize::new(2).unwrap());
        assert_eq!(left, SplitSegment::Child(Child::new(ab, 2)), "left");
        assert_eq!(right, SplitSegment::Child(Child::new(c, 1)), "right");
    }
    #[test]
    fn split_index_2() {
        let (
            graph,
            _a,
            _b,
            _c,
            d,
            _e,
            _f,
            _g,
            _h,
            _i,
            _ab,
            _bc,
            _cd,
            _bcd,
            abc,
            abcd,
            _ef,
            _cdef,
            _efghi,
            _abab,
            _ababab,
            _ababababcdefghi,
        ) = &mut *context_mut();
        let (left, right) = graph.split_index(*abcd, NonZeroUsize::new(3).unwrap());
        assert_eq!(left, SplitSegment::Child(Child::new(abc, 3)), "left");
        assert_eq!(right, SplitSegment::Child(Child::new(d, 1)), "right");
    }
    fn split_index_3_impl() {
        let mut graph = Hypergraph::default();
        if let [a, b, c, d] = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('c'),
            Token::Element('d'),
        ])[..]
        {
            // wxabyzabbyxabyz
            let ab = graph.insert_pattern([a, b]);
            let bc = graph.insert_pattern([b, c]);
            let cd = graph.insert_pattern([c, d]);
            let abc = graph.insert_patterns([vec![ab, c], vec![a, bc]]);
            let bcd = graph.insert_patterns([vec![bc, d], vec![b, cd]]);
            let abcd = graph.insert_patterns([vec![abc, d], vec![a, bcd]]);

            let (left, right) = graph.split_index(abcd, NonZeroUsize::new(2).unwrap());
            assert_eq!(left, SplitSegment::Child(ab), "left");
            assert_eq!(right, SplitSegment::Child(cd), "right");
        } else {
            panic!();
        }
    }
    #[test]
    fn split_index_3() {
        split_index_3_impl()
    }
    #[test]
    fn split_index_4() {
        let mut graph = Hypergraph::default();
        if let [a, b, _w, x, y, z] = graph.insert_tokens([
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
            let xab = graph.insert_pattern([x, ab]);
            let xaby = graph.insert_patterns([vec![xab, y], vec![x, a, by]]);
            let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);

            let (left, right) = graph.split_index(xabyz, NonZeroUsize::new(2).unwrap());
            println!("{:#?}", graph);
            let xa_found = graph.find_pattern(vec![x, a]);
            let xa = if let SearchFound {
                index: xa,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = xa_found.expect("xa not found")
            {
                Some(xa)
            } else {
                None
            }
            .expect("xa");

            let byz_found = graph.find_pattern(vec![b, y, z]);
            let byz = if let SearchFound {
                index: byz,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = byz_found.expect("byz not found")
            {
                Some(byz)
            } else {
                None
            }
            .expect("byz");
            assert_eq!(left, SplitSegment::Child(xa), "left");
            assert_eq!(right, SplitSegment::Child(byz), "left");
        } else {
            panic!();
        }
    }
    #[test]
    fn split_index_5() {
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
            let xa = graph.insert_pattern([x, a]);
            let xab = graph.insert_patterns([vec![x, ab], vec![xa, b]]);
            let xaby = graph.insert_patterns([vec![xab, y], vec![xa, by]]);
            let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);
            let wxabyzabbyxabyz = graph.insert_pattern([w, xabyz, ab, by, xabyz]);

            // split wxabyzabbyxabyz at 3
            let (left, right) = graph.split_index(wxabyzabbyxabyz, NonZeroUsize::new(3).unwrap());

            let xa_found = graph.find_pattern(vec![w, xa]);
            let wxa = if let SearchFound {
                index: wxa,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = xa_found.expect("wxa not found")
            {
                Some(wxa)
            } else {
                None
            }
            .unwrap();
            assert_eq!(left, SplitSegment::Child(wxa), "left");

            let byzabbyxabyz_found = graph.find_pattern(vec![by, z, ab, by, xabyz]);
            let byzabbyxabyz = if let SearchFound {
                index: byzabbyxabyz,
                parent_match:
                    ParentMatch {
                        parent_range: FoundRange::Complete,
                        ..
                    },
                ..
            } = byzabbyxabyz_found.expect("byzabbyxabyz not found")
            {
                Some(byzabbyxabyz)
            } else {
                None
            }
            .expect("byzabbyxabyz");

            assert_eq!(right, SplitSegment::Child(byzabbyxabyz), "left");
        } else {
            panic!();
        }
    }
    fn split_index_6_impl() {
        let mut graph = Hypergraph::default();
        let (a, b, w, x, y, z) = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('w'),
            Token::Element('x'),
            Token::Element('y'),
            Token::Element('z'),
        ]).into_iter().next_tuple().unwrap();
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

        let (left, right) = graph.split_index(wxabyz, NonZeroUsize::new(3).unwrap());
        let wxa_found = graph.find_pattern(vec![w, x, a]);
        let wxa = if let SearchFound {
            index: wxa,
            parent_match:
                ParentMatch {
                    parent_range: FoundRange::Complete,
                    ..
                },
            ..
        } = wxa_found.expect("wxa not found") {
            Some(wxa)
        } else {
            None
        }
        .unwrap();
        let byz_found = graph.find_pattern(vec![b, y, z]);
        println!("{:#?}", byz_found);
        let byz = if let SearchFound {
            index: byz,
            parent_match:
                ParentMatch {
                    parent_range: FoundRange::Complete,
                    ..
                },
            ..
        } = byz_found.expect("byz not found")
        {
            println!("byz = {}", graph.index_string(byz));
            Some(byz)
        } else {
            None
        }
        .unwrap();

        assert_eq!(left, SplitSegment::Child(wxa), "left");
        assert_eq!(right, SplitSegment::Child(byz), "right");
    }
    #[test]
    fn split_index_6() {
        split_index_6_impl()
    }
    //#[bench]
    //fn bench_split_child_patterns_6(b: &mut test::Bencher) {
    //    b.iter(split_child_patterns_6_impl)
    //}
    //#[bench]
    //fn bench_split_child_patterns_3(b: &mut test::Bencher) {
    //    b.iter(split_child_patterns_3_impl)
    //}
}
