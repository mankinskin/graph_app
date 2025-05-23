pub mod bands;
pub mod iterator;
pub mod state;

use std::{
    cmp::Ordering,
    num::NonZeroUsize,
};

use crate::{
    graph::vertex::{
        pattern::{
            IntoPattern,
            id::PatternId,
        },
        wide::Wide,
    },
    trace::cache::position::SubSplitLocation,
};

use std::fmt::Debug;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ChildTracePos {
    pub inner_offset: Option<NonZeroUsize>,
    pub sub_index: usize,
}
impl From<(usize, Option<NonZeroUsize>)> for ChildTracePos {
    fn from((sub_index, inner_offset): (usize, Option<NonZeroUsize>)) -> Self {
        Self {
            sub_index,
            inner_offset,
        }
    }
}
impl From<SubSplitLocation> for (PatternId, ChildTracePos) {
    fn from(sub: SubSplitLocation) -> Self {
        (sub.location.pattern_id, ChildTracePos {
            inner_offset: sub.inner_offset,
            sub_index: sub.location.sub_index,
        })
    }
}

/// Side refers to border (front is indexing before front border, back is indexing after back border)
pub trait TraceSide:
    std::fmt::Debug + Sync + Send + Unpin + Clone + 'static
{
    fn trace_child_pos(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<ChildTracePos>;
}

#[derive(Debug, Clone)]
pub struct TraceBack;

impl TraceSide for TraceBack {
    fn trace_child_pos(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<ChildTracePos> {
        let mut offset = offset.get();
        pattern
            .into_pattern()
            .into_iter()
            .enumerate()
            .find_map(|(i, c)|
            // returns current index when remaining offset is smaller than current child
            match c.width().cmp(&offset) {
                Ordering::Less => {
                    offset -= c.width();
                    None
                }
                Ordering::Equal => {
                    offset = 0;
                    None
                }
                Ordering::Greater => Some((i, NonZeroUsize::new(offset))),
            })
            .map(Into::into)
    }
}

#[derive(Debug, Clone)]
pub struct TraceFront;

impl TraceSide for TraceFront {
    fn trace_child_pos(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<ChildTracePos> {
        let mut offset = offset.get();
        pattern
            .into_pattern()
            .into_iter()
            .enumerate()
            .find_map(|(i, c)|
            // returns current index when remaining offset does not exceed current child
            match c.width().cmp(&offset) {
                Ordering::Less => {
                    offset -= c.width();
                    None
                }
                Ordering::Equal => {
                    offset = 0;
                    Some((i, NonZeroUsize::new(offset)))
                }
                Ordering::Greater => Some((i, NonZeroUsize::new(offset))),
            })
            .map(Into::into)
    }
}
