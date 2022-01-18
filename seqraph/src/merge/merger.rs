use crate::{
    merge::merge_direction::*,
    index::*,
    vertex::*,
    Child,
    Hypergraph,
};

use std::collections::HashSet;
#[derive(Debug)]
pub struct Merger<'g, T: Tokenize, D: MergeDirection> {
    graph: &'g mut Hypergraph<T>,
    _ty: std::marker::PhantomData<D>
}
impl<'g, T: Tokenize, D: MergeDirection> Merger<'g, T, D> {
    pub fn new(graph: &'g mut Hypergraph<T>) -> Self {
        Self { graph, _ty: Default::default() }
    }
    pub fn try_merge_indices(
        &mut self,
        left: Child,
        right: Child,
    ) -> Result<Child, Pattern> {
        let p = [left, right].as_slice();
        self.graph
                .find_parent(p.as_pattern_view())
                .map(|found| self.graph.index_found(p, found))
                .map_err(|_| p.into_pattern())
    }
    pub(crate) fn merge_split(
        &mut self,
        outer: SplitSegment,
        inner: SplitSegment,
    ) -> Pattern {
        let (inner, loc) = match inner {
            SplitSegment::Pattern(inner, inner_loc) => (inner, Some(inner_loc)),
            SplitSegment::Child(inner_head) => (vec![inner_head], None),
        };
        let (inner_head, inner_tail) = D::split_inner_head(inner.clone());
        let (outer, cloc) = match outer {
            SplitSegment::Pattern(outer, cloc) => (outer, Some(cloc)),
            SplitSegment::Child(outer_head) => (vec![outer_head], None),
        };
        let (outer_head, outer_tail) = D::split_context_head(outer.clone()).unwrap();
        let (left, right) = D::merge_order(inner_head, outer_head);
        // try to find parent matching both
        let (inner, overlap, outer) = match self.try_merge_indices(left, right) {
            Ok(overlap) => (inner_tail, Some(overlap), outer_tail),
            Err(p) => (inner, None, outer)
        };
        // index inner context because we are going to reuse it
        let inner = self.graph.insert_pattern(inner);
        D::concat_inner_and_context(inner, overlap, outer)
    }
    /// returns minimal patterns of pattern split
    /// i.e. no duplicate subsequences with respect to entire index
    pub(crate) fn merge_splits(
        &mut self,
        splits: Vec<(SplitSegment, SplitSegment)>,
    ) -> Pattern {
        splits
            .into_iter()
            .try_fold(
                HashSet::new(),
                |mut acc, (context, inner)| {
                    let pat = self.merge_split(context, inner);
                    if pat.len() == 1 {
                        Err(pat.first().unwrap().clone())
                    } else {
                        acc.insert(pat);
                        Ok(acc)
                    }
                })
            .map(|patterns|
                match patterns.len() {
                    0 => panic!("No patterns after merge!"),
                    1 => {
                        let pattern = patterns.into_iter().next().unwrap();
                        if pattern.is_empty() {
                            panic!("Empty pattern after merge!")
                        } else {
                            pattern
                        }
                    }
                    _ => self.graph.insert_patterns(patterns).into_pattern()
                }
            )
            .unwrap_or_else(IntoPattern::into_pattern)
    }
    // returns minimal patterns of pattern split
    // i.e. no duplicate subsequences with respect to entire index
    //pub(crate) fn merge_optional_splits(
    //    &mut self,
    //    splits: impl IntoIterator<Item = (SplitSegment, Option<SplitSegment>)>,
    //) -> Pattern {
    //    splits
    //        .into_iter()
    //        .try_fold(
    //            HashSet::new(),
    //            |mut acc, (context, inner)|
    //            if let Some(inner) = inner {
    //                let pat = self.merge_split(context, inner);
    //                if pat.len() == 1 {
    //                    Err(pat.first().unwrap().clone())
    //                } else {
    //                    acc.insert(pat);
    //                    Ok(acc)
    //                }
    //            } else {
    //                if !context.is_empty() {
    //                    acc.insert(context);
    //                }
    //                Ok(acc)
    //            }
    //        )
    //        .map(|patterns|
    //            match patterns.len() {
    //                0 => panic!("No patterns after merge!"),
    //                1 => {
    //                    let pattern = patterns.into_iter().next().unwrap();
    //                    if pattern.is_empty() {
    //                        panic!("Empty pattern after merge!")
    //                    } else {
    //                        pattern
    //                    }
    //                }
    //                _ => self.graph.insert_patterns(patterns).into_pattern()
    //            }
    //        )
    //        .unwrap_or_else(IntoPattern::into_pattern)
    //}
    // minimal means:
    // - no two indices are adjacient more than once
    // - no two patterns of the same index share an index border
    // returns minimal patterns of pattern split
    // i.e. no duplicate subsequences with respect to entire index
    //pub(crate) fn merge_inner_optional_splits(
    //    &mut self,
    //    splits: Vec<(Option<SplitSegment>, SplitSegment, Option<SplitSegment>)>,
    //) -> Child {
    //    match splits
    //        .into_iter()
    //        .try_fold(HashSet::new(), |mut acc, (left, infix, right)| {
    //            match (left, right) {
    //                (Some(left), Some(right)) => self.add_inner_split(acc, left, infix, right),
    //                (Some(left), None) => {
    //                    let p = self.graph.right_merger().merge_split(infix, left);
    //                    match p.len() {
    //                        1 => Err(p.first().unwrap().to_owned()),
    //                        _ => {
    //                            acc.insert(p);
    //                            Ok(acc)
    //                        }
    //                    }
    //                }
    //                (None, Some(right)) => {
    //                    let p = self.graph.left_merger().merge_split(infix, right);
    //                    match p.len() {
    //                        1 => Err(p.first().unwrap().to_owned()),
    //                        _ => {
    //                            acc.insert(p);
    //                            Ok(acc)
    //                        }
    //                    }
    //                }
    //                (None, None) => match infix.len() {
    //                    1 => Err(infix.unwrap_child()),
    //                    0 => panic!("Empty inner pattern in merge patterns"),
    //                    _ => {
    //                        acc.insert(infix.unwrap_pattern());
    //                        Ok(acc)
    //                    }
    //                },
    //            }
    //        }) {
    //        Ok(patterns) => {
    //            self.graph.insert_patterns(patterns)
    //        }
    //        Err(child) => child,
    //    }
    //}
    //fn add_inner_split(
    //    &mut self,
    //    mut acc: HashSet<Pattern>,
    //    left: SplitSegment,
    //    infix: SplitSegment,
    //    right: SplitSegment,
    //) -> Result<HashSet<Pattern>, Child> {
    //    match infix.len() {
    //        0 => {
    //            let (l, _ltail) = Left::split_context_head(left).unwrap();
    //            let (r, _rtail) = Right::split_context_head(right).unwrap();
    //            match self.try_merge_indices(l, r) {
    //                Ok(c) => Err(c),
    //                Err(pat) => {
    //                    acc.insert(pat);
    //                    Ok(acc)
    //                }
    //            }
    //        }
    //        1 => {
    //            let (l, _) = Left::split_context_head(left.clone()).unwrap();
    //            let (i, _) = Right::split_context_head(infix).unwrap();
    //            let (r, _) = Right::split_context_head(right.clone()).unwrap();
    //            match self.try_merge_indices(l, i).into() {
    //                Ok(lc) => match self.try_merge_indices(lc, r) {
    //                    Ok(c) => Err(c),
    //                    Err(_) => {
    //                        match self.try_merge_indices(i, r) {
    //                            Ok(rc) => {
    //                                acc.insert(lc.into_iter().chain(right).collect());
    //                                acc.insert(left.into_iter().chain(rc).collect());
    //                            }
    //                            Err(_) => {
    //                                acc.insert(lc.into_iter().chain(right).collect());
    //                            }
    //                        }
    //                        Ok(acc)
    //                    }
    //                },
    //                Err(_) => {
    //                    match self.try_merge_indices(i, r) {
    //                        Ok(c) => {
    //                            acc.insert(left.into_iter().chain(c).collect());
    //                        }
    //                        Err(_) => {
    //                            acc.insert(left.into_iter().chain(i).chain(right).collect());
    //                        }
    //                    };
    //                    Ok(acc)
    //                }
    //            }
    //        }
    //        _ => {
    //            let left = self.graph.right_merger().merge_split(infix, left);
    //            let right = self.graph.left_merger().merge_split(left, right);
    //            acc.insert(right);
    //            Ok(acc)
    //        }
    //    }
    //}
}
