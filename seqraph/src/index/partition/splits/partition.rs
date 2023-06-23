use crate::*;

#[derive(Debug, Clone)]
pub struct Partition<K: RangeRole> {
    pub offsets: K::Splits,
}
//#[derive(Clone, Copy)]
//pub struct PartitionRef<'a, K: RangeRole>
//    where K::Splits: 'a
//{
//    pub offsets: &'a K::Splits,
//}
pub trait AsPartition<K: RangeRole>: VisitPartition<K> {
    fn as_partition(self) -> Partition<K>;
}
impl<K: RangeRole> AsPartition<K> for Partition<K>
{
    fn as_partition(self) -> Partition<K> {
        self
    }
}
//impl<'a, K: RangeRole + 'a> AsPartition<'a, K> for &'a Partition<K> {
//    fn as_partition<'t>(self) -> &'t Partition<K> where 'a: 't {
//        PartitionRef {
//            offsets: (&self.offsets).as_ref()
//        }
//    }
//}
impl<'a, M: InVisitMode, A: AsOffsetSplits, B: AsOffsetSplits> AsPartition<In<M>> for Infix<A, B> {
    fn as_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (
                self.left.as_offset_splits(),
                self.right.as_offset_splits(),
            ),
        }
    }
}
impl<'a, M: InVisitMode> AsPartition<In<M>> for (OffsetSplits, OffsetSplits) {
    fn as_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (
                self.0,
                self.1,
            )
        }
    }
}
impl<'a, M: InVisitMode> AsPartition<In<M>> for &'a (OffsetSplits, OffsetSplits) {
    fn as_partition(self) -> Partition<In<M>> {
        Partition {
            offsets: (
                self.0.clone(),
                self.1.clone(),
            )
        }
    }
}
impl<'a, M: PreVisitMode, A: AsOffsetSplits> AsPartition<Pre<M>> for A {
    fn as_partition(self) -> Partition<Pre<M>> {
        Partition {
            offsets: self.as_offset_splits(),
        }
    }
}
impl<'a, M: PostVisitMode, A: AsOffsetSplits> AsPartition<Post<M>> for A {
    fn as_partition(self) -> Partition<Post<M>> {
        Partition {
            offsets: self.as_offset_splits(),
        }
    }
}
impl<'a, M: PreVisitMode, B: AsOffsetSplits> AsPartition<Pre<M>> for Prefix<B> {
    fn as_partition(self) -> Partition<Pre<M>> {
        Partition {
            offsets: self.split.as_offset_splits(),
        }
    }
}
impl<'a, M: PostVisitMode, A: AsOffsetSplits> AsPartition<Post<M>> for Postfix<A> {
    fn as_partition(self) -> Partition<Post<M>> {
        Partition {
            offsets: self.split.as_offset_splits(),
        }
    }
}

#[derive(new, Clone, Copy)]
pub struct Infix<
    A: AsOffsetSplits,
    B: AsOffsetSplits,
> {
    pub left: A,
    pub right: B,
}
#[derive(new, Clone)]
pub struct Prefix<O: AsOffsetSplits> {
    pub split: O,
}
#[derive(new, Clone)]
pub struct Postfix<O: AsOffsetSplits> {
    pub split: O,
}

//pub trait IntoPartition<K: RangeRole> {
//    fn into_partition<'p>(self, ctx: &'p mut Partitioner<'p>) -> Partition<K>;
//}
//
//impl<K: RangeRole> IntoPartition<K> for <K::Kind as RangeKind>::Offsets {
//    fn into_partition<'p>(self, ctx: &'p mut Partitioner<'p>) -> Partition<K> {
//        Partition {
//            offsets: self.as_splits(ctx),
//        }
//    }
//}