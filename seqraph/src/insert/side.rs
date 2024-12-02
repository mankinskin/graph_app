use std::{
    cmp::Ordering,
    num::NonZeroUsize,
};

use crate::graph::vertex::{
    pattern::IntoPattern,
    wide::Wide,
};

/// Side refers to border (front is indexing before front border, back is indexing after back border)
pub trait IndexSide: std::fmt::Debug + Sync + Send + Unpin + Clone + 'static {
    fn token_offset_split(
        pattern: impl IntoPattern,
        offset: NonZeroUsize,
    ) -> Option<(usize, Option<NonZeroUsize>)>;
}

#[derive(Debug, Clone)]
pub struct IndexBack;

impl IndexSide for IndexBack {
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

#[cfg(test)]
mod tests {
    use std::{
        borrow::Borrow,
        num::NonZeroUsize,
    };

    use crate::{
        insert::side::{
            IndexBack,
            IndexFront,
            IndexSide,
        },
        mock,
    };
    use crate::graph::vertex::{
        child::Child,
        pattern::pattern_width,
    };

    #[test]
    fn token_offset_split() {
        let pattern = mock::pattern_from_widths([1, 1, 3, 1, 1]);
        let width = pattern_width(&pattern);
        assert_eq!(
            IndexBack::token_offset_split(
                pattern.borrow() as &[Child],
                NonZeroUsize::new(2).unwrap(),
            ),
            Some((2, None)),
        );
        assert_eq!(
            IndexFront::token_offset_split(
                pattern.borrow() as &[Child],
                NonZeroUsize::new(width - 2).unwrap(),
            ),
            Some((2, None)),
        );
        assert_eq!(
            IndexFront::token_offset_split(
                pattern.borrow() as &[Child],
                NonZeroUsize::new(width - 4).unwrap(),
            ),
            Some((2, NonZeroUsize::new(1))),
        );
    }
}
