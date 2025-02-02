//type OppositeContextRange<D, Ty> =
//    <<Ty as IndexSide<D>>::Opposite as IndexSide<D>>::ContextRange;

use std::{
    cmp::Ordering,
    num::NonZeroUsize,
    ops::Range,
};

use hypercontext_api::graph::vertex::{
    pattern::{
        pattern_range::{
            PatternRangeIndex,
            StartInclusive,
        },
        IntoPattern,
    },
    wide::Wide,
};

pub mod relative;

/// Side refers to border (front is indexing before front border, back is indexing after back border)
pub trait IndexSide: std::fmt::Debug + Sync + Send + Unpin + Clone + 'static {
    type ContextRange: PatternRangeIndex + StartInclusive;
    type InnerRange: PatternRangeIndex + StartInclusive;
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<(usize, Option<NonZeroUsize>)>;
}

#[derive(Debug, Clone)]
pub struct IndexBack;

impl IndexSide for IndexBack {
    type InnerRange = Range<usize>;
    type ContextRange = Range<usize>;
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<(usize, Option<NonZeroUsize>)> {
        let mut offset = offset.get();
        pattern.into_iter().enumerate().find_map(|(i, c)|
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
    }
}

#[derive(Debug, Clone)]
pub struct IndexFront;

impl IndexSide for IndexFront {
    type InnerRange = Range<usize>;
    type ContextRange = Range<usize>;
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<(usize, Option<NonZeroUsize>)> {
        let mut offset = offset.get();
        pattern.into_iter().enumerate().find_map(|(i, c)|
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
    }
}
