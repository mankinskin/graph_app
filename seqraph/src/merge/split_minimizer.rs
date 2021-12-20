use crate::{
    merge::merge_direction::*,
    search::*,
    split::*,
    vertex::*,
    Child,
    Hypergraph,
};

use std::collections::HashSet;
#[derive(Debug)]
pub struct SplitMinimizer<'g, T: Tokenize, D: MergeDirection> {
    graph: &'g mut Hypergraph<T>,
    _ty: std::marker::PhantomData<D>
}
impl<'g, T: Tokenize, D: MergeDirection> SplitMinimizer<'g, T, D> {
    pub fn new(graph: &'g mut Hypergraph<T>) -> Self {
        Self { graph, _ty: Default::default() }
    }
    /// minimize a pattern which has been merged at pos
    pub fn try_merge_indices(
        &mut self,
        left: Child,
        right: Child,
    ) -> Result<Child, Pattern> {
        //println!("pos: {}, len: {}", pos, pattern.len());
        let p = vec![left, right];
        // find pattern over merge position
        self.graph
            .find_parent(p.as_pattern_view())
            .map(
                |SearchFound {
                     index: found,
                     pattern_id,
                     sub_index,
                     parent_match,
                 }| {
                    if parent_match.parent_range.matches_completely() {
                        found
                    } else {
                        // create new index and replace in parent
                        let new = self.graph.insert_pattern(p.clone());
                        self.graph.replace_in_pattern(
                            found,
                            pattern_id,
                            sub_index..=sub_index + 1,
                            new,
                        );
                        new
                    }
                },
            )
            .map_err(|_| p.into_pattern())
    }
    fn merge_split(
        &mut self,
        context: SplitSegment,
        inner: SplitSegment,
    ) -> SplitSegment {
        if let Some((outer_head, outer_tail)) = D::split_context_head(context) {
            let (inner_head, inner_tail) = D::split_inner_head(inner);
            let (left, right) = D::merge_order(inner_head, outer_head);
            // try to find parent matching both
            self.try_merge_indices(left, right)
                .map_err(|pat| D::concat_inner_and_context(inner_tail, pat, outer_tail))
                .into()
        } else {
            inner
        }
    }
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    pub(crate) fn merge_splits(
        &mut self,
        splits: Vec<(Pattern, SplitSegment)>,
    ) -> SplitSegment {
        self.merge_optional_splits(splits.into_iter().map(|(p, c)| (p, Some(c))))
    }
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    pub(crate) fn merge_optional_splits(
        &mut self,
        splits: impl IntoIterator<Item = (Pattern, Option<SplitSegment>)>,
    ) -> SplitSegment {
        splits
            .into_iter()
            .try_fold(
                HashSet::new(),
                |mut acc, (context, inner)|
                if let Some(inner) = inner {
                    match self.merge_split(context.into(), inner) {
                        SplitSegment::Pattern(pat) => {
                            acc.insert(pat);
                            Ok(acc)
                        }
                        // stop when single child is found
                        SplitSegment::Child(c) => Err(c),
                    }
                } else {
                    if !context.is_empty() {
                        acc.insert(context);
                    }
                    Ok(acc)
                }
            )
            .map(|patterns|
                match patterns.len() {
                    0 => panic!("No patterns after merge!"),
                    1 => {
                        let pattern = patterns.into_iter().next().unwrap();
                        match pattern.len() {
                            0 => panic!("Empty pattern after merge!"),
                            1 => pattern.into_iter().next().unwrap().into(),
                            _ => pattern.into()
                        }
                    }
                    _ => self.graph.insert_patterns(patterns).into()
                }
            )
            .unwrap_or_else(SplitSegment::Child)
    }
    /// minimal means:
    /// - no two indices are adjacient more than once
    /// - no two patterns of the same index share an index border
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    pub(crate) fn merge_inner_optional_splits(
        &mut self,
        splits: Vec<(Option<SplitSegment>, SplitSegment, Option<SplitSegment>)>,
    ) -> Child {
        match splits
            .into_iter()
            .try_fold(HashSet::new(), |mut acc, (left, infix, right)| {
                match (left, right) {
                    (Some(left), Some(right)) => self.add_inner_split(acc, left, infix, right),
                    (Some(left), None) => {
                        match self.graph.right_merger().merge_split(infix, left) {
                            SplitSegment::Child(c) => Err(c),
                            SplitSegment::Pattern(pat) => {
                                acc.insert(pat);
                                Ok(acc)
                            }
                        }
                    }
                    (None, Some(right)) => {
                        match self.graph.left_merger().merge_split(infix, right) {
                            SplitSegment::Child(c) => Err(c),
                            SplitSegment::Pattern(pat) => {
                                acc.insert(pat);
                                Ok(acc)
                            }
                        }
                    }
                    (None, None) => match infix.len() {
                        1 => Err(infix.unwrap_child()),
                        0 => panic!("Empty inner pattern in merge patterns"),
                        _ => {
                            acc.insert(infix.unwrap_pattern());
                            Ok(acc)
                        }
                    },
                }
            }) {
            Ok(patterns) => {
                self.graph.insert_patterns(patterns)
                //println!(
                //    "created {} from [\n{}]",
                //    hypergraph.index_string(c),
                //    patterns.into_iter().fold(String::new(), |acc, p| {
                //        format!("{}{},\n", acc, hypergraph.pattern_string(p))
                //    })
                //);
            }
            Err(child) => child,
        }
    }
    fn add_inner_split(
        &mut self,
        mut acc: HashSet<Pattern>,
        left: SplitSegment,
        infix: SplitSegment,
        right: SplitSegment,
    ) -> Result<HashSet<Pattern>, Child> {
        match infix.len() {
            0 => {
                let (l, _ltail) = MergeLeft::split_context_head(left).unwrap();
                let (r, _rtail) = MergeRight::split_context_head(right).unwrap();
                match self.try_merge_indices(l, r).into() {
                    SplitSegment::Child(c) => Err(c),
                    SplitSegment::Pattern(pat) => {
                        acc.insert(pat);
                        Ok(acc)
                    }
                }
            }
            1 => {
                let (l, _) = MergeLeft::split_context_head(left.clone()).unwrap();
                let (i, _) = MergeRight::split_context_head(infix).unwrap();
                let (r, _) = MergeRight::split_context_head(right.clone()).unwrap();
                match self.try_merge_indices(l, i).into() {
                    SplitSegment::Child(lc) => match self.try_merge_indices(lc, r).into() {
                        SplitSegment::Child(c) => Err(c),
                        SplitSegment::Pattern(_) => {
                            match self.try_merge_indices(i, r).into() {
                                SplitSegment::Child(rc) => {
                                    acc.insert(lc.into_iter().chain(right).collect());
                                    acc.insert(left.into_iter().chain(rc).collect());
                                }
                                SplitSegment::Pattern(_) => {
                                    acc.insert(lc.into_iter().chain(right).collect());
                                }
                            }
                            Ok(acc)
                        }
                    },
                    SplitSegment::Pattern(_) => {
                        match self.try_merge_indices(i, r).into() {
                            SplitSegment::Child(c) => {
                                acc.insert(left.into_iter().chain(c).collect());
                            }
                            SplitSegment::Pattern(_) => {
                                acc.insert(left.into_iter().chain(i).chain(right).collect());
                            }
                        };
                        Ok(acc)
                    }
                }
            }
            _ => {
                let left = self.graph.right_merger().merge_split(infix, left);
                let right = self.graph.left_merger().merge_split(left, right).unwrap_pattern();
                acc.insert(right);
                Ok(acc)
            }
        }
    }
}
