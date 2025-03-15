use std::num::NonZeroUsize;

use derive_new::new;

use crate::{
    interval::partition::info::range::{
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
    traversal::split::vertex::{
        ToVertexSplits,
        VertexSplits,
    },
};

pub mod delta;
pub mod info;

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

impl<M: InVisitMode> ToPartition<In<M>> for (VertexSplits, VertexSplits) {
    fn to_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (self.0, self.1),
        }
    }
}

impl<M: InVisitMode> ToPartition<In<M>> for &(VertexSplits, VertexSplits) {
    fn to_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (self.0.clone(), self.1.clone()),
        }
    }
}

impl<M: PreVisitMode, A: ToVertexSplits> ToPartition<Pre<M>> for A {
    fn to_partition(self) -> Partition<Pre<M>> {
        Partition {
            offsets: self.to_vertex_splits(),
        }
    }
}

impl<M: PostVisitMode, A: ToVertexSplits> ToPartition<Post<M>> for A {
    fn to_partition(self) -> Partition<Post<M>> {
        Partition {
            offsets: self.to_vertex_splits(),
        }
    }
}

//
//

#[derive(new, Clone)]
pub struct Infix<A: ToVertexSplits, B: ToVertexSplits> {
    pub left: A,
    pub right: B,
}
impl<A: ToVertexSplits, B: ToVertexSplits> Infix<A, B> {}
impl<M: InVisitMode, A: ToVertexSplits, B: ToVertexSplits> ToPartition<In<M>> for Infix<A, B> {
    fn to_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (self.left.to_vertex_splits(), self.right.to_vertex_splits()),
        }
    }
}

#[derive(new, Clone)]
pub struct Prefix<A: ToVertexSplits> {
    pub split: A,
}

impl<A: ToVertexSplits> Prefix<A> {}
impl<M: PreVisitMode, B: ToVertexSplits> ToPartition<Pre<M>> for Prefix<B> {
    fn to_partition(self) -> Partition<Pre<M>> {
        Partition {
            offsets: self.split.to_vertex_splits(),
        }
    }
}

#[derive(new, Clone)]
pub struct Postfix<O: ToVertexSplits> {
    pub split: O,
}
impl<A: ToVertexSplits> Postfix<A> {}
impl<M: PostVisitMode, A: ToVertexSplits> ToPartition<Post<M>> for Postfix<A> {
    fn to_partition(self) -> Partition<Post<M>> {
        Partition {
            offsets: self.split.to_vertex_splits(),
        }
    }
}

pub fn to_non_zero_range(
    l: usize,
    r: usize,
) -> (NonZeroUsize, NonZeroUsize) {
    (NonZeroUsize::new(l).unwrap(), NonZeroUsize::new(r).unwrap())
}
