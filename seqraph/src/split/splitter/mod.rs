use crate::{
    split::*,
    direction::*,
    Indexed,
    VertexIndex, read::PatternLocation,
};
use std::{
    cmp::PartialEq,
    num::NonZeroUsize,
};
mod single;
mod range;
pub use range::*;

pub type Split = (Pattern, Pattern);

#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd)]
pub struct PatternSplit {
    pub(crate) prefix: Pattern,
    pub(crate) inner: IndexSplit,
    pub(crate) postfix: Pattern,
}
impl PatternSplit {
    pub fn new(
        prefix: Pattern,
        inner: impl Into<IndexSplit>,
        postfix: Pattern,
    ) -> Self {
        Self {
            prefix,
            inner: inner.into(),
            postfix,
        }
    }
}
#[derive(Debug, Clone, Eq, Ord, PartialOrd, Default)]
pub struct IndexSplit {
    pub(crate) splits: Vec<PatternSplit>,
}
impl IndexSplit {
    pub fn new(inner: impl IntoIterator<Item = impl Into<PatternSplit>>) -> Self {
        Self {
            splits: inner.into_iter().map(Into::into).collect(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.splits.is_empty()
    }
    pub fn add_split<T: Into<PatternSplit>>(
        &mut self,
        split: T,
    ) {
        self.splits.push(split.into());
    }
}
impl PartialEq for IndexSplit {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        let a: BTreeSet<_> = self.splits.iter().collect();
        let b: BTreeSet<_> = other.splits.iter().collect();
        a == b
    }
}
impl From<Split> for PatternSplit {
    fn from((prefix, postfix): Split) -> Self {
        Self {
            prefix,
            inner: Default::default(),
            postfix,
        }
    }
}
impl<T: Into<IndexSplit>> From<(Pattern, T, Pattern)> for PatternSplit {
    fn from((prefix, inner, postfix): (Pattern, T, Pattern)) -> Self {
        Self::new(prefix, inner, postfix)
    }
}
impl<T: Into<PatternSplit>> From<Vec<T>> for IndexSplit {
    fn from(splits: Vec<T>) -> Self {
        Self {
            splits: splits.into_iter().map(Into::into).collect(),
        }
    }
}
impl<T: Into<PatternSplit>> From<T> for IndexSplit {
    fn from(split: T) -> Self {
        Self::from(vec![split])
    }
}

pub enum RangeSplitResult {
    Full(Child),
    Single(SplitSegment, SplitSegment),
    Double(SplitSegment, SplitSegment, SplitSegment),
}
impl From<SingleSplitResult> for RangeSplitResult {
    fn from((l, r): SingleSplitResult) -> Self {
        Self::Single(l, r)
    }
}
//impl From<IndexRangeResult> for RangeSplitResult {
//    fn from(r: IndexRangeResult) -> Self {
//        match r {
//            Self::Double(l, i.into(), r)
//        }
//    }
//}
pub type SingleSplitResult = (SplitSegment, SplitSegment);


#[derive(Debug)]
pub struct Splitter<'g, T: Tokenize> {
    graph: &'g mut Hypergraph<T>,
}
impl<'g, T: Tokenize + 'g> Splitter<'g, T> {
    pub fn new(graph: &'g mut Hypergraph<T>) -> Self {
        Self { graph }
    }
    pub(crate) fn split_index(
        &'g mut self,
        root: impl VertexedMut<'g, 'g>,
        pos: NonZeroUsize,
    ) -> SingleSplitResult {
        let vertex = self.graph.expect_vertex_data(&root);
        let patterns = vertex.get_children().clone();
        self.single_split_patterns(root, patterns, pos)
    }
    // TODO: maybe move into merger
    //pub(crate) fn resolve_perfect_split_range(
    //    &mut self,
    //    pat: Pattern,
    //    parent: impl Vertexed,
    //    //pattern_index: PatternId,
    //    //range: impl PatternRangeIndex + Clone,
    //) -> SplitSegment {
    //    if pat.len() <= 1 {
    //        SplitSegment::Child(*pat.first().expect("Empty perfect split half!"))
    //    //} else if parent.vertex(self.graph).children.len() == 1 {
    //    //    SplitSegment::Pattern(pat)
    //    } else {
    //        //let c = self.graph.insert_pattern(pat);
    //        //self.graph
    //        //    .replace_in_pattern(parent, pattern_index, range, vec![c]);
    //        //SplitSegment::Child(c)
    //        SplitSegment::Pattern(pat)
    //    }
    //}
    // Get perfect split or pattern split contexts
    //pub(crate) fn try_perfect_split(
    //    &self,
    //    root: impl Indexed,
    //    pos: NonZeroUsize,
    //) -> Result<(Split, IndexInParent), Vec<SplitContext>> {
    //    let current_node = self.get_vertex_data(root).unwrap();
    //    let children = current_node.get_children().clone();
    //    let child_slices = children.into_iter().map(|(i, p)| (i, p.into_iter()));
    //    let split_indices = Splitter::find_single_split_indices(child_slices, pos);
    //    match Splitter::perfect_split_search(current_node, split_indices)
    //        .into_iter()
    //        .collect()
    //    {
    //        Ok(s) => Err(s),
    //        Err(s) => Ok(s),
    //    }
    //}
}
