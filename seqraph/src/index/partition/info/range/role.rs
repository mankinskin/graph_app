use crate::*;

#[derive(Debug, Clone)]
pub struct Outer;
#[derive(Debug, Clone)]
pub struct Inner;

pub trait RangeOffsets<K: RangeRole>: Debug + Clone + Copy {
    fn as_splits<'a, C: AsBundlingContext<'a>>(&'a self, ctx: C) -> K::Splits;
}
impl RangeOffsets<In> for (NonZeroUsize, NonZeroUsize) {
    fn as_splits<'a, C: AsBundlingContext<'a>>(&'a self, ctx: C) -> <In as RangeRole>::Splits {
        range_splits(ctx.as_bundling_context().patterns.iter(), *self)
    }
}
impl RangeOffsets<Pre> for NonZeroUsize {
    fn as_splits<'a, C: AsBundlingContext<'a>>(&'a self, ctx: C) -> <Pre as RangeRole>::Splits {
        position_splits(ctx.as_bundling_context().patterns.iter(), *self)
    }
}
impl RangeOffsets<Post> for NonZeroUsize {
    fn as_splits<'a, C: AsBundlingContext<'a>>(&'a self, ctx: C) -> <Post as RangeRole>::Splits {
        position_splits(ctx.as_bundling_context().patterns.iter(), *self)
    }
}
pub trait PatternSplits: Debug {
    type Split;
    type Offsets;
    fn get(&self, pid: &PatternId) -> Option<&Self::Split>;
    fn offsets(&self) -> Self::Offsets;
}
impl<'a> PatternSplits for OffsetSplitsRef<'a> {
    type Split = PatternSplitPos;
    type Offsets = usize;
    fn get(&self, pid: &PatternId) -> Option<&Self::Split> {
        self.splits.get(pid)
    }
    fn offsets(&self) -> Self::Offsets {
        self.offset.get()
    }
}
impl<'a> PatternSplits for OffsetSplits {
    type Split = PatternSplitPos;
    type Offsets = usize;
    fn get(&self, pid: &PatternId) -> Option<&Self::Split> {
        self.splits.get(pid)
    }
    fn offsets(&self) -> Self::Offsets {
        self.offset.get()
    }
}
impl<A: PatternSplits, B: PatternSplits> PatternSplits for (A, B) {
    type Split = (A::Split, B::Split);
    type Offsets = (A::Offsets, B::Offsets);
    fn get(&self, pid: &PatternId) -> Option<&Self::Split> {
        self.0.get(pid).map(|a| {
            let b = self.1.get(pid).unwrap();
            &(*a, *b)
        })
    }
    fn offsets(&self) -> Self::Offsets {
        (
            self.0.offsets(),
            self.1.offsets(),
        )
    }
}
pub trait RangeKind: Debug + Clone {
}
impl RangeKind for Inner {
}
impl RangeKind for Outer {
}
pub trait RangeChildren<K: RangeRole>: Debug + Clone {
    fn join_inner(self, inner: Child) -> JoinedPattern;
}
impl RangeChildren<Pre> for Child {
    fn join_inner(self, inner: Child) -> JoinedPattern {
        JoinedPattern::Bigram([inner, self])
    }
}
impl RangeChildren<Post> for Child {
    fn join_inner(self, inner: Child) -> JoinedPattern {
        JoinedPattern::Bigram([inner, self])
    }
}
#[derive(Debug, Clone)]
pub enum InfixChildren {
    Both(Child, Child),
    Left(Child),
    Right(Child),
}
impl RangeChildren<In> for InfixChildren {
    fn join_inner(self, inner: Child) -> JoinedPattern {
        match self {
            Self::Both(l, r) =>
                JoinedPattern::Trigram([l, inner, r]),
            Self::Left(l) =>
                JoinedPattern::Bigram([l, inner]),
            Self::Right(r) =>
                JoinedPattern::Bigram([inner, r]),
        }
    }
}
pub trait OffsetIndexRange<K: RangeRole>: PatternRangeIndex {
    fn get_partition<'a>(&self, vertex: &'a SplitVertexCache) -> PartitionRef<'a, K>;
}
pub trait RangeRole: Debug + Clone {
    type Perfect: BorderPerfect;
    type Offsets: RangeOffsets<Self>;
    type Kind: RangeKind;
    type Range: OffsetIndexRange<Self>;
    type PartitionSplits;
    type Children: RangeChildren<Self>;
    type Borders: PartitionBorders<Self, Splits = <Self::Splits as PatternSplits>::Split>;
    type Splits: PatternSplits + for<'a> AsPartition<'a, Self>;
    type SplitsRef<'a>: PatternSplits<Split = <Self::Splits as PatternSplits>::Split> + AsPartition<'a, Self>;
    fn to_partition(
        splits: Self::Splits,
    ) -> Partition<Self>;
}
impl OffsetIndexRange<In> for Range<usize> {
    fn get_partition<'a>(&self, vertex: &'a SplitVertexCache) -> PartitionRef<'a, In> {
        let lo = vertex.positions.iter().nth(self.start).unwrap();
        let ro = vertex.positions.iter().nth(self.end).unwrap();
        Infix(&lo, &ro).as_partition()
    }
}
impl OffsetIndexRange<Pre> for Range<usize> {
    fn get_partition<'a>(&self, vertex: &'a SplitVertexCache) -> PartitionRef<'a, Pre> {
        let ro = vertex.positions.iter().nth(self.end).unwrap();
        Prefix(&ro).as_partition()
    }
}
impl OffsetIndexRange<Post> for RangeFrom<usize> {
    fn get_partition<'a>(&self, vertex: &'a SplitVertexCache) -> PartitionRef<'a, Post> {
        let lo = vertex.positions.iter().nth(self.start).unwrap();
        Postfix(&lo).as_partition()
    }
}
#[derive(Debug, Clone)]
pub struct Pre;
impl RangeRole for Pre {
    type Range = Range<usize>;
    type Kind = Outer;
    type Children = Child;
    type PartitionSplits = ((), OffsetSplits);
    type Borders = BorderInfo<Right>;
    type Splits = OffsetSplits;
    type SplitsRef<'a> = OffsetSplitsRef<'a>;
    type Offsets = NonZeroUsize;
    type Perfect = Option<PatternId>;
    fn to_partition(
        splits: Self::Splits,
    ) -> Partition<Self> {
        Partition {
            offsets: splits,
        }
    }
}
#[derive(Debug, Clone)]
pub struct In;
impl RangeRole for In {
    type Range = Range<usize>;
    type Kind = Inner;
    type Children = InfixChildren;
    type PartitionSplits = (OffsetSplits, OffsetSplits);
    type Borders = (BorderInfo<Left>, BorderInfo<Right>);
    type Splits = (OffsetSplits, OffsetSplits);
    type SplitsRef<'a> = (OffsetSplitsRef<'a>, OffsetSplitsRef<'a>);
    type Offsets = (NonZeroUsize, NonZeroUsize);
    type Perfect = (Option<PatternId>, Option<PatternId>);
    fn to_partition(
        splits: Self::Splits,
    ) -> Partition<Self> {
        Partition {
            offsets: splits,
        }
    }
}
#[derive(Debug, Clone)]
pub struct Post;
impl RangeRole for Post {
    type Range = RangeFrom<usize>;
    type Kind = Outer;
    type Children = Child;
    type PartitionSplits = (OffsetSplits, ());
    type Borders = BorderInfo<Left>;
    type Splits = OffsetSplits;
    type SplitsRef<'a> = OffsetSplitsRef<'a>;
    type Offsets = NonZeroUsize;
    type Perfect = Option<PatternId>;
    fn to_partition(splits: Self::Splits) -> Partition<Self> {
        Partition {
            offsets: splits,
        }
    }
}
