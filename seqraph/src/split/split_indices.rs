use crate::{
    split::*,
    VertexIndex,
};
use std::{
    borrow::Borrow,
    num::NonZeroUsize,
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SplitIndex {
    pos: TokenPosition,
    index: VertexIndex,
    index_pos: IndexPosition,
}
pub type SingleSplitIndices = Vec<(PatternId, SplitIndex)>;
pub(crate) struct SplitIndices;
impl<'g> SplitIndices {
    /// Get perfect split if it exists and remaining pattern split contexts
    pub(crate) fn find_perfect_split(
        patterns: ChildPatterns,
        pos: NonZeroUsize,
    ) -> (Option<(Split, IndexInParent)>, Vec<SplitContext>) {
        let split_indices = Self::build_single(&patterns, pos);
        Self::separate_perfect_single_split(patterns, split_indices)
    }
    /// Find single split indices and positions of multiple patterns
    pub(crate) fn build_single(
        patterns: impl IntoIterator<Item = (impl Borrow<PatternId>, impl IntoPattern<Item = impl AsChild>)>,
        pos: NonZeroUsize,
    ) -> SingleSplitIndices {
        patterns
            .into_iter()
            .map(move |(i, pattern)| {
                let split =
                    Self::find_ancestor_split_index(pattern, pos).expect("Split not in pattern");
                (*i.borrow(), split)
            })
            .collect()
    }
    /// separate perfect and remaining split indices
    pub(crate) fn separate_perfect_single_split(
        patterns: ChildPatterns,
        split_indices: impl IntoIterator<Item = (PatternId, SplitIndex)> + 'g,
    ) -> (Option<(Split, IndexInParent)>, Vec<SplitContext>) {
        let len = patterns.len();
        Self::perfect_split_search(patterns, split_indices)
            .into_iter()
            .fold((None, Vec::with_capacity(len)), |(pa, mut sa), r| match r {
                Ok(s) => {
                    sa.push(s);
                    (pa, sa)
                }
                Err(s) => (Some(s), sa),
            })
    }
    /// search for a perfect split in the split indices (offset = 0)
    pub(crate) fn perfect_split_search<'a>(
        patterns: ChildPatterns,
        split_indices: impl IntoIterator<Item = (PatternId, SplitIndex)> + 'a,
    ) -> impl IntoIterator<Item = Result<SplitContext, (Split, IndexInParent)>> + 'a {
        split_indices
            .into_iter()
            .map(move |(pattern_index, split_index)| {
                let pattern = patterns.get(&pattern_index).unwrap();
                Self::separate_pattern_split(pattern_index, split_index)
                    .map(
                        move |(
                            key,
                            IndexInParent {
                                replaced_index: split_index,
                                ..
                            },
                        )| {
                            let (prefix, postfix) = split_context(pattern, split_index);
                            SplitContext {
                                prefix,
                                key,
                                postfix,
                            }
                        },
                    )
                    .map_err(
                        |ind @ IndexInParent {
                             replaced_index: split_index,
                             ..
                         }| {
                            (split_pattern_at_index(pattern, split_index), ind)
                        },
                    )
            })
    }
    /// search for a perfect split in the split indices (offset = 0)
    pub(crate) fn separate_pattern_split(
        pattern_index: PatternId,
        split_index: SplitIndex,
    ) -> Result<(SplitKey, IndexInParent), IndexInParent> {
        let SplitIndex {
            index_pos,
            pos,
            index,
        } = split_index;
        let index_in_parent = IndexInParent {
            pattern_index,
            replaced_index: index_pos,
        };
        NonZeroUsize::new(pos)
            .map(|offset| (SplitKey::new(index, offset), index_in_parent.clone()))
            .ok_or(index_in_parent)
    }
    /// find split position in index in pattern
    pub(crate) fn find_ancestor_split_index(
        pattern: impl IntoPattern<Item = impl AsChild>,
        pos: NonZeroUsize,
    ) -> Option<SplitIndex> {
        let mut skipped = 0;
        let pos: TokenPosition = pos.into();
        // find child overlapping with cut pos or
        pattern.into_iter().enumerate().find_map(|(i, child)| {
            let child = child.as_child();
            if skipped + child.width() <= pos {
                skipped += child.width();
                None
            } else {
                Some(SplitIndex {
                    index_pos: i,
                    pos: pos - skipped,
                    index: child.index,
                })
            }
        })
    }
    // build intermediate split kind for multiple patterns
    pub(crate) fn build_double(
        current_node: &VertexData,
        patterns: impl IntoIterator<Item = (impl Borrow<PatternId>, impl IntoPattern<Item = impl AsChild> + Clone)>,
        left: NonZeroUsize,
        right: NonZeroUsize,
    ) -> DoubleSplitIndices {
        match patterns
            .into_iter()
            .try_fold(vec![], move |mut acc, (pattern_index, pattern)| {
                let pattern_index = *pattern_index.borrow();
                let left_split = SplitIndices::find_ancestor_split_index(pattern.clone(), left)
                    .expect("left split not in pattern");
                let right_split = SplitIndices::find_ancestor_split_index(pattern, right)
                    .expect("right split not in pattern");
                let left = SplitIndices::separate_pattern_split(pattern_index, left_split);
                let right = SplitIndices::separate_pattern_split(pattern_index, right_split);
                let pattern = current_node.get_child_pattern(&pattern_index).unwrap();
                match (left, right) {
                    (Ok((left, left_ind)), Ok((right, right_ind))) => {
                        // both unperfect
                        let left_index = left_ind.replaced_index;
                        let right_index = right_ind.replaced_index;
                        let new = match right_index - left_index {
                            0 => {
                                let (prefix, postfix) = split_pattern_at_index(pattern, left_index);
                                (
                                    pattern_index,
                                    DoubleSplitIndex::Inner(
                                        prefix,
                                        (left.index, left.offset, right.offset),
                                        postfix,
                                    ),
                                )
                            }
                            _ => {
                                let (prefix, infix, postfix) =
                                    double_split_context(pattern, left_index, right_index);
                                (
                                    pattern_index,
                                    DoubleSplitIndex::Infix(prefix, left, infix, right, postfix),
                                )
                            }
                        };
                        acc.push(new);
                        Ok(acc)
                    }
                    (Ok((left, left_ind)), Err(right_ind)) => {
                        // only right perfect
                        let left_index = left_ind.replaced_index;
                        let right_index = right_ind.replaced_index;
                        let (prefix, rem) = split_context(pattern, left_index);
                        let (infix, postfix) =
                            split_pattern_at_index(&rem, right_index - left_index);
                        let new = (
                            pattern_index,
                            DoubleSplitIndex::Right(prefix, left, infix, right_index, postfix),
                        );
                        acc.push(new);
                        Ok(acc)
                    }
                    (Err(left_ind), Ok((right, right_ind))) => {
                        // only left perfect
                        let left_index = left_ind.replaced_index;
                        let right_index = right_ind.replaced_index;
                        let (prefix, rem) = split_pattern_at_index(pattern, left_index);
                        let (infix, postfix) = split_context(&rem, right_index - left_index);
                        let new = (
                            pattern_index,
                            DoubleSplitIndex::Left(prefix, left_index, infix, right, postfix),
                        );
                        acc.push(new);
                        Ok(acc)
                    }
                    (Err(left_ind), Err(right_ind)) => {
                        // both perfect
                        let left_index = left_ind.replaced_index;
                        let right_index = right_ind.replaced_index;
                        let (prefix, rem) = split_pattern_at_index(pattern, left_index);
                        let (infix, postfix) =
                            split_pattern_at_index(&rem, right_index - left_index);
                        Err((
                            pattern_index,
                            prefix,
                            left_index,
                            infix,
                            right_index,
                            postfix,
                        ))
                    }
                }
            }) {
            Ok(indices) => Err(indices),
            Err(split) => Ok(split),
        }
    }
    pub(crate) fn verify_range_split_indices(
        width: usize,
        range: impl PatternRangeIndex,
    ) -> DoubleSplitPositions {
        if range.contains(&0) && range.contains(&width) {
            return DoubleSplitPositions::None;
        }
        let lower = if let Bound::Included(&lo) = range.start_bound() {
            lo
        } else if let Bound::Excluded(&lo) = range.start_bound() {
            lo.checked_sub(1).unwrap_or_default()
        } else {
            0
        };
        let higher = if let Bound::Included(&hi) = range.end_bound() {
            hi.checked_add(1).unwrap_or(width)
        } else if let Bound::Excluded(&hi) = range.end_bound() {
            hi
        } else {
            width
        };
        if let Some(lower) = NonZeroUsize::new(lower) {
            let higher = NonZeroUsize::new(higher)
                .filter(|higher| higher.get() > lower.get())
                .expect("PatternRangeIndex with higher <= lower bound!");
            if higher.get() < width {
                DoubleSplitPositions::Double(lower, higher)
            } else {
                DoubleSplitPositions::SinglePostfix(lower)
            }
        } else {
            // lower bound out
            DoubleSplitPositions::SinglePrefix(
                NonZeroUsize::new(higher).expect("upper bound is zero dispite check"),
            )
        }
    }
}