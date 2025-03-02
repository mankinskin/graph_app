use std::{
    cmp::Ordering,
    num::NonZeroUsize,
};

use crate::graph::vertex::{
    pattern::IntoPattern,
    wide::Wide,
};

use super::PatternSplitPos;

/// Side refers to border (front is indexing before front border, back is indexing after back border)
pub trait SplitSide: std::fmt::Debug + Sync + Send + Unpin + Clone + 'static {
    fn token_pos_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<PatternSplitPos>;
}

#[derive(Debug, Clone)]
pub struct SplitBack;

impl SplitSide for SplitBack {
    fn token_pos_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<PatternSplitPos> {
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
pub struct SplitFront;

impl SplitSide for SplitFront {
    fn token_pos_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<PatternSplitPos> {
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
