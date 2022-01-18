use crate::{
    vertex::*,
    direction::*,
    Hypergraph,
};
use std::num::NonZeroUsize;

mod indexer;
use indexer::*;
mod split_indices;
use split_indices::*;
mod split_segment;
pub use split_segment::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IndexSplitResult {
    Prefix(Child, SplitSegment),
    Postfix(SplitSegment, Child),
}
impl IndexSplitResult {
    pub fn prefix(self) -> Option<(Child, SplitSegment)> {
        match self {
            Self::Prefix(c, s) => Some((c, s)),
            Self::Postfix(_, _) => None,
        }
    }
    pub fn postfix(self) -> Option<(SplitSegment, Child)> {
        match self {
            Self::Prefix(_, _) => None,
            Self::Postfix(c, s) => Some((c, s)),
        }
    }
    pub fn unwrap_prefix(self) -> (Child, SplitSegment) {
        self.prefix()
            .expect("called IndexSplitResult::unwrap_prefix on a `Postfix` value")
    }
    pub fn unwrap_postfix(self) -> (SplitSegment, Child) {
        self.postfix()
            .expect("called IndexSplitResult::unwrap_postfix on a `Prefix` value")
    }
}
pub enum IndexRangeResult {
    Full(Child),
    Prefix(Child, SplitSegment),
    Postfix(SplitSegment, Child),
    Infix(SplitSegment, Child, SplitSegment),
}
impl IndexRangeResult {
    pub fn unwrap_child(self) -> Child {
        match self {
            Self::Full(c) => c,
            Self::Prefix(c, _) => c,
            Self::Postfix(_, c) => c,
            Self::Infix(_, c, _) => c,
        }
    }
}
impl From<IndexSplitResult> for IndexRangeResult {
    fn from(r: IndexSplitResult) -> Self {
        match r {
            IndexSplitResult::Prefix(i, c) => Self::Prefix(i, c),
            IndexSplitResult::Postfix(c, i) => Self::Postfix(c, i),
        }
    }
}
pub trait IntoSplit: Clone {
    fn into_split(self) -> (SplitSegment, SplitSegment);
    fn concat(self) -> Pattern {
        let (l, r) = self.into_split();
        [l.into_pattern(), r.into_pattern()].concat()
    }
}
impl IntoSplit for (Child, SplitSegment) {
    fn into_split(self) -> (SplitSegment, SplitSegment) {
        let (l, r) = self;
        (l.into(), r)
    }
    fn concat(self) -> Pattern {
        let (l, r) = self;
        [l.into_pattern(), r.into_pattern()].concat()
    }
}
impl IntoSplit for (SplitSegment, Child) {
    fn into_split(self) -> (SplitSegment, SplitSegment) {
        let (l, r) = self;
        (l, r.into())
    }
    fn concat(self) -> Pattern {
        let (l, r) = self;
        [l.into_pattern(), r.into_pattern()].concat()
    }
}
impl IntoSplit for (Child, Child) {
    fn into_split(self) -> (SplitSegment, SplitSegment) {
        let (l, r) = self;
        (l.into(), r.into())
    }
    fn concat(self) -> Pattern {
        let (l, r) = self;
        vec![l, r]
    }
}
pub(crate) trait IndexSide {
    type IndexResult: IntoSplit;
    fn trivial(lc: Child, rc: Child) -> Self::IndexResult;
    fn index_and_add<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        root: impl AsChild,
        l: Pattern,
        r: Pattern,
    ) -> Self::IndexResult;
    fn index_and_replace<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        loc: PatternLocation,
        l: Pattern,
        r: Pattern,
    ) -> Self::IndexResult;
    //fn replace_index_prefix_ready<T: Tokenize>(
    //    graph: &mut Hypergraph<T>,
    //    l: Child,
    //    r: SplitSegment,
    //) -> Self::IndexResult {
    //    let (i, c) = Self::pick_index_side(l.into(), r);
    //    let i = match i {
    //        // if i is left, it must be a child, we do nothing
    //        // if i is right, but a child, we just return it
    //        SplitSegment::Child(c) => c,
    //        // if i is right and a pattern, we index it
    //        SplitSegment::Pattern(p, loc) => {
    //            let i = graph.insert_pattern(p);
    //            graph.replace_range_at(loc, 1.., i.clone());
    //            i
    //        },
    //    };
    //    Self::build_result(i, c.into())
    //}
    //fn replace_index_postfix_ready<T: Tokenize>(
    //    graph: &mut Hypergraph<T>,
    //    l: SplitSegment,
    //    r: Child,
    //) -> Self::IndexResult {
    //    let (i, c) = Self::pick_index_side(l, r.into());
    //    let i = match i {
    //        // if i is right, it must be a child, we do nothing
    //        // if i is left, but a child, we just return it
    //        SplitSegment::Child(c) => c,
    //        // if i is left and a pattern, we index it
    //        SplitSegment::Pattern(p, loc) => {
    //            let i = graph.insert_pattern(p);
    //            let end = loc.parent.vertex(&graph).expect_pattern_len(&loc.pattern_id);
    //            graph.replace_range_at(loc, 0..end-1, i.clone());
    //            i
    //        },
    //    };
    //    Self::build_result(i, c.into())
    //}
}
impl IndexSide for Left {
    type IndexResult = (Child, SplitSegment);
    fn index_and_add<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        root: impl AsChild,
        l: Pattern,
        r: Pattern,
    ) -> Self::IndexResult {
        let l = graph.insert_pattern(l);
        let pid = graph.add_pattern_to_node(root, [[l].as_slice(), r.as_slice()].concat());
        let loc = PatternLocation::new(root.as_child(), pid);
        (l, SplitSegment::with_location(r, loc))
    }
    fn index_and_replace<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        loc: PatternLocation,
        l: Pattern,
        r: Pattern,
    ) -> Self::IndexResult {
        let l = match l.len() {
            1 => l.pop().unwrap(),
            len => {
                let i = graph.insert_pattern(l);
                graph.replace_range_at(loc, 0..len, i.clone());
                i
            },
        };
        (l, SplitSegment::with_location(r, loc))
    }
    fn trivial(lc: Child, rc: Child) -> Self::IndexResult {
        (lc, rc.into())
    }
}
impl IndexSide for Right {
    type IndexResult = (SplitSegment, Child);
    fn index_and_add<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        root: impl AsChild,
        l: Pattern,
        r: Pattern,
    ) -> Self::IndexResult {
        let r = graph.insert_pattern(r);
        let pid = graph.add_pattern_to_node(root, [l.as_slice(), [r].as_slice()].concat());
        let loc = PatternLocation::new(root.as_child(), pid);
        (SplitSegment::with_location(l, loc), r)
    }
    fn index_and_replace<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        loc: PatternLocation,
        l: Pattern,
        r: Pattern,
    ) -> Self::IndexResult {
        let r = match r.len() {
            1 => r.pop().unwrap(),
            len => {
                let range = {
                    let l = l.len();
                    l..l + len
                };
                let i = graph.insert_pattern(r);
                graph.replace_range_at(loc, range, i.clone());
                i
            },
        };
        (SplitSegment::with_location(l, loc), r)
    }
    fn trivial(lc: Child, rc: Child) -> Self::IndexResult {
        (lc.into(), rc)
    }
}
impl IndexSide for Both {
    type IndexResult = (Child, Child);
    fn index_and_add<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        root: impl AsChild,
        l: Pattern,
        r: Pattern,
    ) -> Self::IndexResult {
        let l = graph.insert_pattern(l);
        let r = graph.insert_pattern(r);
        let _pid = graph.add_pattern_to_node(root, [l, r].as_slice());
        (l, r)
    }
    fn index_and_replace<T: Tokenize>(
        graph: &mut Hypergraph<T>,
        loc: PatternLocation,
        l: Pattern,
        r: Pattern,
    ) -> Self::IndexResult {
        match (l.len(), r.len()) {
            (1, 1) => (l.pop().unwrap(), r.pop().unwrap()),
            (llen, rlen) => {
                let l = graph.insert_pattern(l);
                let r = graph.insert_pattern(r);
                graph.replace_range_at(loc, 0..llen + rlen, [l, r].as_slice());
                (l, r)
            },
        }
    }
    fn trivial(lc: Child, rc: Child) -> Self::IndexResult {
        (lc, rc)
    }
}

impl<'t, 'g, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub fn indexer(&'g mut self) -> Indexer<'g, T> {
        Indexer::new(&mut self)
    }
    // create index from token position range in index
    pub fn index_subrange(
        &mut self,
        root: impl AsChild,
        range: impl PatternRangeIndex,
    ) -> IndexRangeResult {
        self.indexer().index_subrange(root, range)
    }
    pub fn index_prefix(
        &mut self,
        root: impl AsChild,
        pos: NonZeroUsize,
    ) -> (Child, SplitSegment) {
        self.indexer().index_single_split::<Left, _>(root.as_child(), pos)
    }
    pub fn index_postfix(
        &mut self,
        root: impl AsChild,
        pos: NonZeroUsize,
    ) -> (SplitSegment, Child) {
        self.indexer().index_single_split::<Right, _>(root.as_child(), pos)
    }
    pub fn index_split(
        &mut self,
        root: impl AsChild,
        pos: NonZeroUsize,
    ) -> (Child, Child) {
        self.indexer().index_single_split::<Both, _>(root.as_child(), pos)
    }
}