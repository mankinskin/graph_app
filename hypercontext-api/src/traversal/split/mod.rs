pub mod cache;
pub mod context;
pub mod pattern;
pub mod run;
pub mod trace;
pub mod vertex;

use std::{
    fmt::Debug,
    num::NonZeroUsize,
};

use crate::{
    graph::vertex::{
        child::Child,
        location::SubLocation,
        pattern::{
            id::PatternId,
            Pattern,
        },
    },
    interval::side::{
        SplitBack,
        SplitSide,
    },
    HashMap,
};
use cache::position::PosKey;
use vertex::VertexSplits;

use super::cache::entry::position::SubSplitLocation;

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
