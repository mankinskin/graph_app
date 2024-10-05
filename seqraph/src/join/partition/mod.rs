use std::num::NonZeroUsize;

use derive_new::new;

use crate::join::partition::{
    info::range::role::{
        In,
        InVisitMode,
        Post,
        PostVisitMode,
        Pre,
        PreVisitMode,
        RangeRole,
    },
    splits::offset::{
        ToOffsetSplits,
        OffsetSplits,
    },
};

pub mod info;
pub mod splits;

#[derive(new, Clone, Copy)]
pub struct Infix<A: ToOffsetSplits, B: ToOffsetSplits> {
    pub left: A,
    pub right: B,
}

#[derive(new, Clone)]
pub struct Prefix<O: ToOffsetSplits> {
    pub split: O,
}

#[derive(new, Clone)]
pub struct Postfix<O: ToOffsetSplits> {
    pub split: O,
}

#[derive(Debug, Clone)]
pub struct Partition<K: RangeRole> {
    pub offsets: K::Splits,
}

pub trait ToPartition<K: RangeRole>: Clone {
    fn to_partition(self) -> Partition<K>;
}

impl<K: RangeRole> ToPartition<K> for Partition<K> {
    fn to_partition(self) -> Partition<K> {
        self
    }
}

impl<M: InVisitMode, A: ToOffsetSplits, B: ToOffsetSplits> ToPartition<In<M>> for Infix<A, B> {
    fn to_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (self.left.to_offset_splits(), self.right.to_offset_splits()),
        }
    }
}

impl<M: InVisitMode> ToPartition<In<M>> for (OffsetSplits, OffsetSplits) {
    fn to_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (self.0, self.1),
        }
    }
}

impl<M: InVisitMode> ToPartition<In<M>> for &(OffsetSplits, OffsetSplits) {
    fn to_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (self.0.clone(), self.1.clone()),
        }
    }
}

impl<M: PreVisitMode, A: ToOffsetSplits> ToPartition<Pre<M>> for A {
    fn to_partition(self) -> Partition<Pre<M>> {
        Partition {
            offsets: self.to_offset_splits(),
        }
    }
}

impl<M: PostVisitMode, A: ToOffsetSplits> ToPartition<Post<M>> for A {
    fn to_partition(self) -> Partition<Post<M>> {
        Partition {
            offsets: self.to_offset_splits(),
        }
    }
}

impl<M: PreVisitMode, B: ToOffsetSplits> ToPartition<Pre<M>> for Prefix<B> {
    fn to_partition(self) -> Partition<Pre<M>> {
        Partition {
            offsets: self.split.to_offset_splits(),
        }
    }
}

impl<M: PostVisitMode, A: ToOffsetSplits> ToPartition<Post<M>> for Postfix<A> {
    fn to_partition(self) -> Partition<Post<M>> {
        Partition {
            offsets: self.split.to_offset_splits(),
        }
    }
}

pub fn to_non_zero_range(
    l: usize,
    r: usize,
) -> (NonZeroUsize, NonZeroUsize) {
    (NonZeroUsize::new(l).unwrap(), NonZeroUsize::new(r).unwrap())
}

#[cfg(test)]
mod tests {
    #[test]
    fn first_partition() {}

    #[test]
    fn inner_partition() {
        //let cache = SplitCache {
        //    entries: HashMap::from([]),
        //    leaves: vec![],
        //};
        //let patterns = vec![];
        //let (lo, ro) = to_non_zero_range(1, 3);
        //let (ls, rs) = range_splits(&patterns, (lo, ro));
        //let (l, r) = ((&lo, ls), (&ro, rs));
        //let bundle = (l, r).info_bundle();
    }

    #[test]
    fn last_partition() {}
}
