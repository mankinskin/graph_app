use crate::*;

#[derive(Debug, Clone, Copy)]
pub struct Outer;
#[derive(Debug, Clone, Copy)]
pub struct Inner;

#[derive(Debug, Clone, Copy)]
pub struct Join;

#[derive(Debug, Clone, Copy)]
pub struct Trace;

pub type ModeOf<K> = <K as RangeRole>::Mode;
pub type ModeChildrenOf<K> = <ModeOf<K> as ModeChildren::<K>>::Result;

pub trait ModeChildren<K: RangeRole> {
    type Result: Clone + Debug;
}
impl<K: RangeRole<Mode = Trace>> ModeChildren<K> for Trace {
    type Result = (); 
}
impl<K: RangeRole<Mode = Join>> ModeChildren<K> for Join {
    type Result = K::Children; 
}

pub trait VisitMode<K: RangeRole<Mode = Self>>: Debug + Clone + Copy + ModeChildren<K> + for<'a> ModeContext<'a> {
    fn border_children<'a>(
        borders: &K::Borders<'a>,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> ModeChildrenOf<K>;
}
impl<K: RangeRole<Mode = Self>> VisitMode<K> for Trace {
    fn border_children<'a>(
        _borders: &K::Borders<'a>,
        _ctx: &ModePatternCtxOf<'a, K>,
    ) -> ModeChildrenOf<K> {
        ()
    }
}
impl<K: RangeRole<Mode = Self>> VisitMode<K> for Join
    where for<'a> K::Borders<'a>: JoinBorders<'a, K>
{
    fn border_children<'a>(
        borders: &K::Borders<'a>,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> ModeChildrenOf<K> {
        borders.children(ctx).expect("inner range needs children")
    }
}

pub trait RangeKind: Debug + Clone {
}
impl RangeKind for Inner {
}
impl RangeKind for Outer {
}
pub trait RangeRole: Debug + Clone + Copy {
    type Mode: VisitMode<Self>; // todo: use to change join/trace
    type Perfect: BorderPerfect;
    type Offsets: RangeOffsets<Self>;
    type Kind: RangeKind;
    type Range: OffsetIndexRange<Self>;
    type PartitionSplits;
    type Children: RangeChildren<Self>;
    type Borders<'a>: VisitBorders<'a, Self, Splits = <Self::Splits as PatternSplits>::Pos>;
    type Splits: PatternSplits + AsPartition<Self>;
    //type SplitsRef<'a>: PatternSplitsRef<'a, Ref<'a> = Self::SplitsRef<'a>, Pos = <Self::Splits as PatternSplits>::Pos> + Copy + AsPartition<'a, Self>;
    fn to_partition(
        splits: Self::Splits,
    ) -> Partition<Self>;
}
pub trait OffsetIndexRange<K: RangeRole>: PatternRangeIndex {
    fn get_splits<'a>(&self, vertex: &'a SplitVertexCache) -> K::Splits;
}
impl<M: InVisitMode> OffsetIndexRange<In<M>> for Range<usize> {
    fn get_splits<'a>(&self, vertex: &'a SplitVertexCache) -> <In<M> as RangeRole>::Splits {
        let lo = vertex.positions.iter().nth(self.start).unwrap();
        let ro = vertex.positions.iter().nth(self.end).unwrap();
        (
            lo.as_offset_splits(),
            ro.as_offset_splits(),
        )
    }
}
impl<M: PreVisitMode> OffsetIndexRange<Pre<M>> for Range<usize> {
    fn get_splits<'a>(&self, vertex: &'a SplitVertexCache) -> <Pre<M> as RangeRole>::Splits {
        let ro = vertex.positions.iter().nth(self.end).unwrap();
        ro.as_offset_splits()
    }
}
impl<M: PostVisitMode> OffsetIndexRange<Post<M>> for RangeFrom<usize> {
    fn get_splits<'a>(&self, vertex: &'a SplitVertexCache) -> <Post<M> as RangeRole>::Splits {
        let lo = vertex.positions.iter().nth(self.start).unwrap();
        lo.as_offset_splits()
    }
}
#[derive(Debug, Clone, Default, Copy)]
pub struct Pre<M: PreVisitMode>(std::marker::PhantomData<M>);

impl<M: PreVisitMode> RangeRole for Pre<M> {
    type Mode = M;
    type Range = Range<usize>;
    type Kind = Outer;
    type Children = Child;
    type PartitionSplits = ((), OffsetSplits);
    type Borders<'a> = BorderInfo;
    type Splits = OffsetSplits;
    //type SplitsRef<'a> = OffsetSplitsRef<'a>;
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
pub trait PreVisitMode: VisitMode<Pre<Self>> + ModeChildren<Pre<Self>> {}
impl PreVisitMode for Trace {}
impl PreVisitMode for Join {}

pub trait PostVisitMode: VisitMode<Post<Self>> + ModeChildren<Post<Self>> {}
impl PostVisitMode for Trace {}
impl PostVisitMode for Join {}

pub trait InVisitMode: VisitMode<In<Self>> + ModeChildren<In<Self>> + PreVisitMode + PostVisitMode {}
impl InVisitMode for Trace {}
impl InVisitMode for Join {}

#[derive(Debug, Clone, Default, Copy)]
pub struct In<M: InVisitMode>(std::marker::PhantomData<M>);
impl<M: InVisitMode> RangeRole for In<M> {
    type Mode = M;
    type Range = Range<usize>;
    type Kind = Inner;
    type Children = InfixChildren;
    type PartitionSplits = (OffsetSplits, OffsetSplits);
    type Borders<'a> = (BorderInfo, BorderInfo);
    type Splits = (OffsetSplits, OffsetSplits);
    //type SplitsRef<'a> = (OffsetSplitsRef<'a>, OffsetSplitsRef<'a>);
    type Offsets = (NonZeroUsize, NonZeroUsize);
    type Perfect = (Option<PatternId>, Option<PatternId>);
    //type SubOffsets = (Option<NonZeroUsize>, Option<NonZeroUsize>);
    fn to_partition(
        splits: Self::Splits,
    ) -> Partition<Self> {
        Partition {
            offsets: splits,
        }
    }
}
#[derive(Debug, Clone, Default, Copy)]
pub struct Post<M: PostVisitMode>(std::marker::PhantomData<M>);
impl<M: PostVisitMode> RangeRole for Post<M> {
    type Mode = M;
    type Range = RangeFrom<usize>;
    type Kind = Outer;
    type Children = Child;
    type PartitionSplits = (OffsetSplits, ());
    type Borders<'a> = BorderInfo;
    type Splits = OffsetSplits;
    //type SplitsRef<'a> = OffsetSplitsRef<'a>;
    type Offsets = NonZeroUsize;
    type Perfect = Option<PatternId>;
    //type SubOffsets = Option<NonZeroUsize>;
    fn to_partition(splits: Self::Splits) -> Partition<Self> {
        Partition {
            offsets: splits,
        }
    }
}
