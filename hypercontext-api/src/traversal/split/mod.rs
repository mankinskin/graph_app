pub mod cache;
pub mod context;
pub mod node;
pub mod pattern;
pub mod vertex;

use std::{
    collections::VecDeque,
    fmt::Debug,
    num::NonZeroUsize,
};

use crate::{
    graph::{
        getters::vertex::VertexSet,
        vertex::{
            child::Child,
            location::SubLocation,
            pattern::{
                id::PatternId,
                Pattern,
            },
        },
    },
    interval::side::{
        SplitBack,
        SplitSide,
    },
    traversal::traversable::Traversable,
    HashMap,
};
use cache::{
    leaves::Leaves,
    PosKey,
};
use vertex::VertexSplits;

use super::{
    cache::entry::position::{
        Offset,
        SubSplitLocation,
    },
    trace::TraceState,
};

#[derive(Debug, Default)]
pub struct SplitStates {
    pub leaves: Leaves,
    pub queue: VecDeque<TraceState>,
}
impl Iterator for SplitStates {
    type Item = TraceState;
    fn next(&mut self) -> Option<Self::Item> {
        self.queue.pop_front()
    }
}
impl SplitStates {
    /// kind of like filter_leaves but from subsplits to trace states
    pub fn filter_trace_states<Trav: Traversable>(
        &mut self,
        trav: Trav,
        index: &Child,
        pos_splits: impl IntoIterator<Item = (Offset, Vec<SubSplitLocation>)>,
    ) {
        let (perfect, next) = {
            let graph = trav.graph();
            let node = graph.expect_vertex(index);
            pos_splits
                .into_iter()
                .flat_map(|(parent_offset, locs)| {
                    let len = locs.len();
                    locs.into_iter().map(move |sub|
                    // filter sub locations without offset (perfect splits)
                    sub.inner_offset.map(|offset|
                        TraceState {
                            index: *node.expect_child_at(&sub.location),
                            offset,
                            prev: PosKey {
                                index: *index,
                                pos: parent_offset,
                            },
                        }
                    ).ok_or_else(||
                        (len == 1).then(||
                            PosKey::new(*index, parent_offset)
                        )
                    ))
                })
                .fold((Vec::new(), Vec::new()), |(mut p, mut n), res| {
                    match res {
                        Ok(s) => n.push(s),
                        Err(Some(k)) => p.push(k),
                        Err(None) => {}
                    }
                    (p, n)
                })
        };
        self.leaves.extend(perfect);
        self.queue.extend(next);
    }
}

pub fn position_splits<'a>(
    patterns: impl IntoIterator<Item = (&'a PatternId, &'a Pattern)>,
    pos: NonZeroUsize,
) -> VertexSplits {
    VertexSplits {
        pos,
        splits: patterns
            .into_iter()
            .map(|(pid, pat)| {
                let pos = SplitBack::token_pos_split(pat, pos).unwrap();
                (*pid, pos)
            })
            .collect(),
    }
}

pub(crate) fn range_splits<'a>(
    patterns: impl Iterator<Item = (&'a PatternId, &'a Pattern)>,
    parent_range: (NonZeroUsize, NonZeroUsize),
) -> (VertexSplits, VertexSplits) {
    let (ls, rs) = patterns
        .map(|(pid, pat)| {
            let lpos = SplitBack::token_pos_split(pat, parent_range.0).unwrap();
            let rpos = SplitBack::token_pos_split(pat, parent_range.1).unwrap();
            ((*pid, lpos), (*pid, rpos))
        })
        .unzip();
    (
        VertexSplits {
            pos: parent_range.0,
            splits: ls,
        },
        VertexSplits {
            pos: parent_range.1,
            splits: rs,
        },
    )
}

pub(crate) fn cleaned_position_splits<'a>(
    patterns: impl Iterator<Item = (&'a PatternId, &'a Pattern)>,
    parent_offset: NonZeroUsize,
) -> Result<Vec<SubSplitLocation>, SubLocation> {
    patterns
        .map(|(pid, pat)| {
            let pos = SplitBack::token_pos_split(pat, parent_offset).unwrap();
            let location = SubLocation::new(*pid, pos.sub_index);
            if pos.inner_offset.is_some() || pat.len() > 2 {
                // can't be clean
                Ok(SubSplitLocation {
                    location,
                    inner_offset: pos.inner_offset,
                })
            } else {
                // must be clean
                Err(location)
            }
        })
        .collect()
}

pub trait SplitInner: Debug + Clone {}

impl<T: Debug + Clone> SplitInner for T {}

#[derive(Debug, Clone)]
pub struct Split<T: SplitInner = Child> {
    pub left: T,
    pub right: T,
}

impl<T: SplitInner> Split<T> {
    pub fn new(
        left: T,
        right: T,
    ) -> Self {
        Self { left, right }
    }
}

impl<I, T: SplitInner + Extend<I> + IntoIterator<Item = I>> Split<T> {
    pub fn infix(
        &mut self,
        mut inner: Split<T>,
    ) {
        self.left.extend(inner.left);
        inner.right.extend(self.right.clone());
        self.right = inner.right;
    }
}

pub type SplitMap = HashMap<PosKey, Split>;
//pub trait HasSplitMap {
//    fn split_map(&self) -> &SplitMap;
//}
//
//impl HasSplitMap for SplitMap {
//    fn split_map(&self) -> &SplitMap {
//        self
//    }
//}
//impl HasSplitMap for PosSplits<SplitVertexCache> {
//    fn split_map(&self) -> &SubSplits {
//        &self.into_iter().collect()
//    }
//}
