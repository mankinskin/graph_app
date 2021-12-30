use crate::{
    split::*,
    direction::*,
};
use std::{
    num::NonZeroUsize,
};
type ChildSplits = (Vec<(Pattern, SplitSegment)>, Vec<(Pattern, SplitSegment)>);
pub(crate) trait IndexSide {
    fn trivial(lc: Child, rc: Child) -> IndexSplitResult;
    fn insert_index_to_root<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        root: impl Indexed,
        l: SplitSegment,
        r: SplitSegment,
    ) -> IndexSplitResult {
        let (i, c) = Self::pick_index_side(l, r);
        let i = match i {
            SplitSegment::Child(i) => i,
            SplitSegment::Pattern(p) => graph.insert_pattern(p),
        };
        graph.add_pattern_to_node(root, Self::concat_index_and_context(i, c.clone()));
        Self::build_result(i, c)
    }
    fn replace_index_in_root<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        root: impl Indexed,
        pid: PatternId,
        l: SplitSegment,
        r: SplitSegment,
    ) -> IndexSplitResult {
        let range = Self::replace_range(&l, &r);
        let (i, c) = Self::pick_index_side(l, r);
        let i = match i {
            SplitSegment::Child(i) => i,
            SplitSegment::Pattern(p) => {
                let i = graph.insert_pattern(p);
                graph.replace_in_pattern(root, pid, range, i.clone());
                i
            },
        };
        Self::build_result(i, c)
    }
    fn replace_index_prefix_ready<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        root: impl Indexed,
        pid: PatternId,
        l: Child,
        r: SplitSegment,
    ) -> IndexSplitResult {
        let (i, c) = Self::pick_index_side(l.into(), r);
        let i = match i {
            // if i is left, it must be a child, we do nothing
            // if i is right, but a child, we just return it
            SplitSegment::Child(c) => c,
            // if i is right and a pattern, we index it
            SplitSegment::Pattern(p) => {
                let i = graph.insert_pattern(p);
                graph.replace_in_pattern(root, pid, 1.., i.clone());
                i
            },
        };
        Self::build_result(i, c)
    }
    fn replace_index_postfix_ready<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        root: impl Indexed,
        pid: PatternId,
        l: SplitSegment,
        r: Child,
    ) -> IndexSplitResult {
        let (i, c) = Self::pick_index_side(l, r.into());
        let i = match i {
            // if i is right, it must be a child, we do nothing
            // if i is left, but a child, we just return it
            SplitSegment::Child(c) => c,
            // if i is left and a pattern, we index it
            SplitSegment::Pattern(p) => {
                let i = graph.insert_pattern(p);
                let end = graph
                    .expect_vertex_data(&root).expect_child_pattern(&pid).len();
                graph.replace_in_pattern(root, pid, 0..end-1, i.clone());
                i
            },
        };
        Self::build_result(i, c)
    }
    fn pick_index_side(l: SplitSegment, r: SplitSegment) -> (SplitSegment, SplitSegment);
    fn concat_index_and_context(i: Child, c: SplitSegment) -> Pattern;
    fn build_result(i: Child, c: SplitSegment) -> IndexSplitResult;
    fn replace_range(l: &SplitSegment, r: &SplitSegment) -> std::ops::Range<usize>;
}
impl IndexSide for Left {
    fn build_result(i: Child, c: SplitSegment) -> IndexSplitResult {
        IndexSplitResult::Prefix(i, c)
    }
    fn trivial(lc: Child, rc: Child) -> IndexSplitResult {
        Self::build_result(lc, rc.into())
    }
    fn pick_index_side(l: SplitSegment, r: SplitSegment) -> (SplitSegment, SplitSegment) {
        (l, r)
    }
    fn concat_index_and_context(i: Child, c: SplitSegment) -> Pattern {
        i.clone().into_iter().chain(c.clone()).collect()
    }
    fn replace_range(l: &SplitSegment, _: &SplitSegment) -> std::ops::Range<usize> {
        0..l.len()
    }
}
impl IndexSide for Right {
    fn build_result(i: Child, c: SplitSegment) -> IndexSplitResult {
        IndexSplitResult::Postfix(c, i)
    }
    fn trivial(lc: Child, rc: Child) -> IndexSplitResult {
        Self::build_result(rc, lc.into())
    }
    fn pick_index_side(l: SplitSegment, r: SplitSegment) -> (SplitSegment, SplitSegment) {
        (r, l)
    }
    fn concat_index_and_context(i: Child, c: SplitSegment) -> Pattern {
        c.clone().into_iter().chain(i.clone()).collect()
    }
    fn replace_range(l: &SplitSegment, r: &SplitSegment) -> std::ops::Range<usize> {
        let l = l.len();
        l..l + r.len()
    }
}

impl<'g, T: Tokenize + 'g> IndexSplitter<'g, T> {
    pub(crate) fn index_single_split<D: IndexSide, R: Indexed>(
        &mut self,
        root: R,
        pos: NonZeroUsize,
    ) -> IndexSplitResult {
        let vertex = self.graph.expect_vertex_data(&root);
        let patterns = vertex.get_children().clone();
        //println!("splitting {} at {}", self.index_string(root), pos);
        self.index_single_split_patterns::<D, R>(root, patterns, pos)
    }
    pub(crate) fn index_single_split_patterns<D: IndexSide, R: Indexed>(
        &mut self,
        root: R,
        patterns: ChildPatterns,
        pos: NonZeroUsize,
    ) -> IndexSplitResult {
        let (perfect_split, remaining_splits)
            = SplitIndices::find_perfect_split(patterns, pos);
        // check if any perfect split segment is a single child
        if let Some(((pl, pr), ind)) = perfect_split {
            let (pl, pr): (SplitSegment, SplitSegment) = (pl.into(), pr.into());
            match (pl, pr) {
                // early return when complete split present
                (SplitSegment::Child(lc), SplitSegment::Child(rc)) => D::trivial(lc, rc),
                // at least one side not complete child
                (SplitSegment::Pattern(pl), SplitSegment::Pattern(pr)) => {
                    // perform inner splits
                    let (left, right)
                        = self.split_child_indices(remaining_splits);

                    let left = Self::append_perfect_split(pl, left);
                    let left = self.graph.merge_left_optional_splits(left);
                    let right = Self::append_perfect_split(pr, right);
                    let right = self.graph.merge_right_optional_splits(right);
                    D::replace_index_in_root(self.graph, root, ind.pattern_index, left, right)
                },
                (SplitSegment::Child(lc), SplitSegment::Pattern(pr)) => {
                    // perform inner splits
                    let (_, right)
                        = self.split_child_indices(remaining_splits);

                    let right = Self::append_perfect_split(pr, right);
                    let right = self.graph.merge_right_optional_splits(right);

                    D::replace_index_prefix_ready(self.graph, root, ind.pattern_index, lc, right)
                }
                (SplitSegment::Pattern(pl), SplitSegment::Child(rc)) => {
                    // perform inner splits
                    let (left, _)
                        = self.split_child_indices(remaining_splits);

                    let left = Self::append_perfect_split(pl, left);
                    let left = self.graph.merge_right_optional_splits(left);
                    D::replace_index_postfix_ready(self.graph, root, ind.pattern_index, left, rc)
                }
            }
        } else {
            // no perfect split
            // perform inner splits
            let (left, right)
                = self.split_child_indices(remaining_splits);
            let (left, right) = (
                self.graph.merge_left_splits(left),
                self.graph.merge_right_splits(right),
            );
            D::insert_index_to_root(self.graph, root, left, right)
        }
    }
    pub(crate) fn single_split_patterns(
        &mut self,
        root: impl Indexed,
        patterns: ChildPatterns,
        pos: NonZeroUsize,
    ) -> SingleSplitResult {
        let (perfect_split, remaining_splits)
            = SplitIndices::find_perfect_split(patterns, pos);
        // check if any perfect split segment is a single child
        if let Some(((pl, pr), ind)) = perfect_split {
            let (pl, pr): (SplitSegment, SplitSegment) = (pl.into(), pr.into());
            match (pl, pr) {
                // early return when complete split present
                (lc@SplitSegment::Child(_), rc@SplitSegment::Child(_)) => (lc, rc),
                // at least one side not complete child
                (SplitSegment::Pattern(pl), SplitSegment::Pattern(pr)) => {
                    // perform inner splits
                    let (left, right)
                        = self.split_child_indices(remaining_splits);

                    let left = Self::append_perfect_split(pl, left);
                    let left = self.graph.merge_left_optional_splits(left);
                    let right = Self::append_perfect_split(pr, right);
                    let right = self.graph.merge_right_optional_splits(right);
                    self.graph.add_pattern_to_node(root, left.clone().into_iter().chain(right.clone()));
                    (left, right)
                },
                (lc@SplitSegment::Child(_), SplitSegment::Pattern(pr)) => {
                    // perform inner splits
                    let (_, right)
                        = self.split_child_indices(remaining_splits);

                    let right = Self::append_perfect_split(pr, right);
                    let right = self.graph.merge_right_optional_splits(right);

                    self.graph
                        .replace_in_pattern(root, ind.pattern_index, 1.., right.clone());
                    (lc, right)
                }
                (SplitSegment::Pattern(pl), rc@SplitSegment::Child(_)) => {
                    // perform inner splits
                    let (left, _)
                        = self.split_child_indices(remaining_splits);

                    let left = Self::append_perfect_split(pl, left);
                    let left = self.graph.merge_right_optional_splits(left);
                    let end = self.graph
                        .expect_vertex_data(&root).expect_child_pattern(&ind.pattern_index).len();
                    self.graph
                        .replace_in_pattern(root, ind.pattern_index, 0..end-1, left.clone());
                    (left, rc)
                }
            }
        } else {
            // no perfect split
            // perform inner splits
            let (left, right)
                = self.split_child_indices(remaining_splits);
            let (left, right) = (
                self.graph.merge_left_splits(left),
                self.graph.merge_right_splits(right),
            );
            self.graph
                .add_pattern_to_node(root, left.clone().into_iter().chain(right.clone()));
            (left, right)
        }
    }
    pub(crate) fn split_child_indices(
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
    pub(crate) fn append_perfect_split(
        p: Pattern,
        splits: Vec<(Pattern, SplitSegment)>
    ) -> Vec<(Pattern, Option<SplitSegment>)> {
        splits
            .into_iter()
            .map(|(c, i)| (c, Some(i)))
            .chain(Some((p, None)))
            .collect()
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
        let (left, right) = graph.index_splitter().split_index(*abc, NonZeroUsize::new(2).unwrap());
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
        let (left, right) = graph.index_splitter().split_index(*abcd, NonZeroUsize::new(3).unwrap());
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

            let (left, right) = graph.index_splitter().split_index(abcd, NonZeroUsize::new(2).unwrap());
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
            let xa = graph.insert_pattern([x, a]);
            let xab = graph.insert_patterns([[x, ab], [xa, b]]);
            let xaby = graph.insert_patterns([vec![xab, y], vec![xa, by]]);
            let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);

            let (left, right) = graph.index_splitter().split_index(xabyz, NonZeroUsize::new(2).unwrap());
            println!("{:#?}", graph);

            let byz_found = graph.find_ancestor(vec![b, y, z]);
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
            assert_eq!(right, SplitSegment::Child(byz), "right");
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
            let (left, right) = graph.index_splitter().split_index(wxabyzabbyxabyz, NonZeroUsize::new(3).unwrap());

            assert_eq!(left, SplitSegment::Pattern(vec![w, xa]), "left");

            println!("{:#?}", right);
            let byz_found = graph.find_ancestor(vec![by, z]);
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

            assert_eq!(right, SplitSegment::Pattern(vec![byz, ab, by, xabyz]), "left");
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

        let (left, right) = graph.index_splitter().split_index(wxabyz, NonZeroUsize::new(3).unwrap());
        let wxa_found = graph.find_ancestor(vec![w, x, a]);
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
        let byz_found = graph.find_ancestor(vec![b, y, z]);
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
