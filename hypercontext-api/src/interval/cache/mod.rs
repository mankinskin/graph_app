use std::num::NonZeroUsize;

use crate::{
    graph::vertex::{
        child::Child,
        location::SubLocation,
        pattern::{
            id::PatternId,
            Pattern,
        },
        wide::Wide,
    },
    interval::partition::split::VertexSplits,
    traversal::cache::entry::position::SubSplitLocation,
};

use super::side::{
    SplitBack,
    SplitSide,
};
use std::fmt::Debug;

pub mod vertex;

pub mod ctx;
pub mod leaves;
pub mod position;

#[derive(Clone, Debug, Copy, Hash, Eq, PartialEq)]
pub struct PosKey {
    pub index: Child,
    pub pos: NonZeroUsize,
}

impl PosKey {
    pub fn new<P: TryInto<NonZeroUsize>>(
        index: Child,
        pos: P,
    ) -> Self
    where
        P::Error: Debug,
    {
        Self {
            index,
            pos: pos.try_into().unwrap(),
        }
    }
}

impl From<Child> for PosKey {
    fn from(index: Child) -> Self {
        Self {
            index,
            pos: NonZeroUsize::new(index.width()).unwrap(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TraceState {
    pub index: Child,
    pub offset: NonZeroUsize,
    pub prev: PosKey,
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
