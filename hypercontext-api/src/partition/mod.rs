use std::num::NonZeroUsize;

use derive_new::new;

use crate::partition::{
    info::range::{
        mode::{
            InVisitMode,
            PostVisitMode,
            PreVisitMode,
        },
        role::{
            In,
            Post,
            Pre,
            RangeRole,
        },
    },
    splits::offset::{
        OffsetSplit,
        ToOffsetSplit,
    },
};

pub mod context;
pub mod delta;
pub mod info;
pub mod pattern;
pub mod splits;

#[derive(new, Clone, Copy)]
pub struct Infix<A: ToOffsetSplit, B: ToOffsetSplit> {
    pub left: A,
    pub right: B,
}
impl<M: InVisitMode, A: ToOffsetSplit, B: ToOffsetSplit> ToPartition<In<M>> for Infix<A, B> {
    fn to_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (self.left.to_offset_splits(), self.right.to_offset_splits()),
        }
    }
}

#[derive(new, Clone)]
pub struct Prefix<O: ToOffsetSplit> {
    pub split: O,
}

impl<M: PreVisitMode, B: ToOffsetSplit> ToPartition<Pre<M>> for Prefix<B> {
    fn to_partition(self) -> Partition<Pre<M>> {
        Partition {
            offsets: self.split.to_offset_splits(),
        }
    }
}

#[derive(new, Clone)]
pub struct Postfix<O: ToOffsetSplit> {
    pub split: O,
}
impl<M: PostVisitMode, A: ToOffsetSplit> ToPartition<Post<M>> for Postfix<A> {
    fn to_partition(self) -> Partition<Post<M>> {
        Partition {
            offsets: self.split.to_offset_splits(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Partition<R: RangeRole> {
    pub offsets: R::Splits,
}
impl<R: RangeRole> Partition<R> {
    pub fn new(offsets: impl ToPartition<R>) -> Self {
        offsets.to_partition()
    }
}

pub trait ToPartition<R: RangeRole>: Clone {
    fn to_partition(self) -> Partition<R>;
}

impl<R: RangeRole> ToPartition<R> for Partition<R> {
    fn to_partition(self) -> Partition<R> {
        self
    }
}

impl<M: InVisitMode> ToPartition<In<M>> for (OffsetSplit, OffsetSplit) {
    fn to_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (self.0, self.1),
        }
    }
}

impl<M: InVisitMode> ToPartition<In<M>> for &(OffsetSplit, OffsetSplit) {
    fn to_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (self.0.clone(), self.1.clone()),
        }
    }
}

impl<M: PreVisitMode, A: ToOffsetSplit> ToPartition<Pre<M>> for A {
    fn to_partition(self) -> Partition<Pre<M>> {
        Partition {
            offsets: self.to_offset_splits(),
        }
    }
}

impl<M: PostVisitMode, A: ToOffsetSplit> ToPartition<Post<M>> for A {
    fn to_partition(self) -> Partition<Post<M>> {
        Partition {
            offsets: self.to_offset_splits(),
        }
    }
}

pub fn to_non_zero_range(
    l: usize,
    r: usize,
) -> (NonZeroUsize, NonZeroUsize) {
    (NonZeroUsize::new(l).unwrap(), NonZeroUsize::new(r).unwrap())
}
