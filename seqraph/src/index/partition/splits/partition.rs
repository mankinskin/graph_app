use crate::*;

pub struct Partition<K: RangeRole> {
    pub offsets: K::Splits,
}
pub struct PartitionRef<'a, K: RangeRole> {
    pub offsets: K::SplitsRef<'a>,
}
pub trait AsPartition<'a, K: RangeRole>: 'a {
    fn as_partition<'t>(&'t self) -> PartitionRef<'t, K> where 'a: 't;
}
impl<'a, K: RangeRole + 'a> AsPartition<'a, K> for PartitionRef<'a, K> {
    fn as_partition<'t>(&'t self) -> PartitionRef<'t, K> where 'a: 't {
        *self
    }
}
impl<'a, K: RangeRole + 'a> AsPartition<'a, K> for Partition<K> {
    fn as_partition<'t>(&'t self) -> PartitionRef<'t, K> where 'a: 't {
        self.offsets.as_partition()
    }
}
impl<'a, A: AsOffsetSplits<'a>, B: AsOffsetSplits<'a>> AsPartition<'a, In> for Infix<'a, A, B> {
    fn as_partition<'t>(&'t self) -> PartitionRef<'t, In> where 'a: 't {
        (self.0, self.1).as_partition()
    }
}
impl<'a, A: AsOffsetSplits<'a>, B: AsOffsetSplits<'a>> AsPartition<'a, In> for (A, B) {
    fn as_partition<'t>(&'t self) -> PartitionRef<'t, In> where 'a: 't {
        PartitionRef {
            offsets: (
                self.0.as_offset_splits(),
                self.1.as_offset_splits(),
            ),
        }
    }
}
impl<'a, A: AsOffsetSplits<'a>> AsPartition<'a, Pre> for A {
    fn as_partition<'t>(&'t self) -> PartitionRef<'t, Pre> where 'a: 't {
        PartitionRef {
            offsets: self.as_offset_splits(),
        }
    }
}
impl<'a, A: AsOffsetSplits<'a>> AsPartition<'a, Post> for A {
    fn as_partition<'t>(&'t self) -> PartitionRef<'t, Post> where 'a: 't {
        PartitionRef {
            offsets: self.as_offset_splits(),
        }
    }
}
pub struct Infix<
    'a,
    A: AsOffsetSplits<'a>,
    B: AsOffsetSplits<'a>,
>(pub &'a A, pub &'a B);
pub struct Prefix<'a, O: AsOffsetSplits<'a>>(pub &'a O);
pub struct Postfix<'a, O: AsOffsetSplits<'a>>(pub &'a O);

impl<'a, B: AsOffsetSplits<'a>> AsPartition<'a, Pre> for Prefix<'a, B> {
    fn as_partition<'t>(&'t self) -> PartitionRef<'t, Pre> where 'a: 't {
        PartitionRef {
            offsets: self.0.as_offset_splits(),
        }
    }
}
//impl<'a, A: AsOffsetSplits<'a>> AsPartition<'a, Post> for A {
//    fn as_partition<'t>(&'t self) -> PartitionRef<'t, Post> where 'a: 't {
//        PartitionRef {
//            offsets: self.as_offset_splits(),
//        }
//    }
//}
impl<'a, A: AsOffsetSplits<'a>> AsPartition<'a, Post> for Postfix<'a, A> {
    fn as_partition<'t>(&'t self) -> PartitionRef<'t, Post> where 'a: 't {
        PartitionRef {
            offsets: self.0.as_offset_splits(),
        }
    }
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