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
        let vertex = self.graph.expect_vertex_data(root).clone();
        let patterns = vertex.get_children().clone();
        //println!("splitting {} at {}", self.index_string(root), pos);
        self.process_single_splits(vertex, root, patterns, pos)
    }
    pub(crate) fn process_single_splits(
        &mut self,
        vertex: VertexData,
        root: impl Indexed + Clone,
        patterns: ChildPatterns,
        pos: NonZeroUsize,
    ) -> (Option<PatternId>, SingleSplitResult) {
        let (perfect_split, remaining_splits)
            = SplitIndices::find_perfect_split(&vertex, patterns, pos);
        if let Some(((pl, pr), ind)) = perfect_split {
            (
                Some(ind.pattern_index),
                (
                    self.resolve_perfect_split_segment(pl),
                    self.resolve_perfect_split_segment(pr),
                ),
            )
        } else {
            let (left, right)
                = self.build_child_splits(remaining_splits);
            let mut minimizer = SplitMinimizer::new(self.graph);
            let left = minimizer.merge_left_splits(left);
            let right = minimizer.merge_right_splits(right);
            self.graph
                .add_pattern_to_node(root, left.clone().into_iter().chain(right.clone()));
            (None, (left, right))
        }
    }
    pub(crate) fn resolve_perfect_split_segment(
        &mut self,
        pat: Pattern,
    ) -> SplitSegment {
        if pat.len() <= 1 {
            SplitSegment::Child(*pat.first().expect("Empty perfect split half!"))
        } else {
            SplitSegment::Pattern(pat)
        }
    }
    pub(crate) fn build_child_splits(
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
        SplitIndices::find_perfect_split(&vertex, patterns, pos)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        graph::tests::*,
        r#match::*,
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
