use std::{
    fmt::Debug,
    hash::Hash,
    num::NonZeroUsize,
    ops::{
        Range,
        RangeFrom,
    },
};

use crate::{
    graph::vertex::{
        child::Child,
        pattern::pattern_range::PatternRangeIndex,
    }, partition::{
        context::{AsNodeTraceContext, NodeTraceContext}, info::{
            border::{
                perfect::{
                    BorderPerfect,
                    DoublePerfect,
                    SinglePerfect,
                },
                visit::VisitBorders,
                BorderInfo,
            },
            range::{
                children::{
                    InfixChildren,
                    RangeChildren,
                },
                mode::VisitMode,
                splits::{
                    PatternSplits,
                    RangeOffsets,
                },
            },
        }, pattern::{AsPatternContext, AsPatternTraceContext, PatternTraceContext}, splits::offset::{
            OffsetSplits,
            ToOffsetSplits,
        }, Partition, ToPartition
    }, split::cache::vertex::SplitVertexCache
};

#[derive(Debug, Clone, Copy)]
pub struct Outer;

#[derive(Debug, Clone, Copy)]
pub struct Inner;


#[derive(Debug, Clone, Copy)]
pub struct Trace;

pub type OffsetsOf<R> = <R as RangeRole>::Offsets;
pub type PerfectOf<R> = <R as RangeRole>::Perfect;
pub type BooleanPerfectOf<R> = <PerfectOf<R> as BorderPerfect>::Boolean;
pub type ChildrenOf<R> = <R as RangeRole>::Children;
pub type RangeOf<R> = <R as RangeRole>::Range;
pub type ModeOf<R> = <R as RangeRole>::Mode;
pub type BordersOf<R> = <R as RangeRole>::Borders;
pub type ModeChildrenOf<R> = <ModeOf<R> as ModeChildren<R>>::Result;
pub type PatternCtxOf<'a, R> = <<R as RangeRole>::Mode as ModeContext<'a>>::PatternResult;
pub type ModeNodeCtxOf<'a, R> = <<R as RangeRole>::Mode as ModeContext<'a>>::NodeResult;
pub type ModePatternCtxOf<'a, R> = <<R as RangeRole>::Mode as ModeContext<'a>>::PatternResult;

pub trait ModeContext<'a> {
    type NodeResult: AsNodeTraceContext<'a>
        + AsPatternContext<'a, PatternCtx<'a> = Self::PatternResult>;
    type PatternResult: AsPatternTraceContext<'a> + Hash + Eq;
}

impl<'a> ModeContext<'a> for Trace {
    type NodeResult = NodeTraceContext<'a>;
    type PatternResult = PatternTraceContext<'a>;
}

pub trait ModeChildren<R: RangeRole> {
    type Result: Clone + Debug;
}

impl<R: RangeRole<Mode = Trace>> ModeChildren<R> for Trace {
    type Result = ();
}

pub trait RangeKind: Debug + Clone {}

impl RangeKind for Inner {}

impl RangeKind for Outer {}

pub trait RangeRole: Debug + Clone + Copy {
    type Mode: VisitMode<Self>; // todo: use to change join/trace
    type Perfect: BorderPerfect;
    type Offsets: RangeOffsets<Self>;
    type Kind: RangeKind;
    type Range: OffsetIndexRange<Self>;
    type PartitionSplits;
    type Children: RangeChildren<Self>;
    type Borders: VisitBorders<Self, Splits = <Self::Splits as PatternSplits>::Pos>;
    type Splits: PatternSplits + ToPartition<Self>;
    fn to_partition(splits: Self::Splits) -> Partition<Self>;
}

pub trait OffsetIndexRange<R: RangeRole>: PatternRangeIndex {
    fn get_splits(
        &self,
        vertex: &SplitVertexCache,
    ) -> R::Splits;
}

impl<M: InVisitMode> OffsetIndexRange<In<M>> for Range<usize> {
    fn get_splits(
        &self,
        vertex: &SplitVertexCache,
    ) -> <In<M> as RangeRole>::Splits {
        let lo = vertex.positions.iter().nth(self.start).unwrap();
        let ro = vertex.positions.iter().nth(self.end).unwrap();
        (lo.to_offset_splits(), ro.to_offset_splits())
    }
}

impl<M: PreVisitMode> OffsetIndexRange<Pre<M>> for Range<usize> {
    fn get_splits(
        &self,
        vertex: &SplitVertexCache,
    ) -> <Pre<M> as RangeRole>::Splits {
        let ro = vertex.positions.iter().nth(self.end).unwrap();
        ro.to_offset_splits()
    }
}

impl<M: PostVisitMode> OffsetIndexRange<Post<M>> for RangeFrom<usize> {
    fn get_splits(
        &self,
        vertex: &SplitVertexCache,
    ) -> <Post<M> as RangeRole>::Splits {
        let lo = vertex.positions.iter().nth(self.start).unwrap();
        lo.to_offset_splits()
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
    type Borders = BorderInfo;
    type Splits = OffsetSplits;
    type Offsets = NonZeroUsize;
    type Perfect = SinglePerfect;
    fn to_partition(splits: Self::Splits) -> Partition<Self> {
        Partition { offsets: splits }
    }
}

pub trait PreVisitMode: VisitMode<Pre<Self>> {}

impl PreVisitMode for Trace {}

pub trait PostVisitMode: VisitMode<Post<Self>> {}

impl PostVisitMode for Trace {}


pub trait InVisitMode: VisitMode<In<Self>> + PreVisitMode + PostVisitMode {}

impl InVisitMode for Trace {}


#[derive(Debug, Clone, Default, Copy)]
pub struct In<M: InVisitMode>(std::marker::PhantomData<M>);

impl<M: InVisitMode> RangeRole for In<M> {
    type Mode = M;
    type Range = Range<usize>;
    type Kind = Inner;
    type Children = InfixChildren;
    type PartitionSplits = (OffsetSplits, OffsetSplits);
    type Borders = (BorderInfo, BorderInfo);
    type Splits = (OffsetSplits, OffsetSplits);
    type Offsets = (NonZeroUsize, NonZeroUsize);
    type Perfect = DoublePerfect;
    fn to_partition(splits: Self::Splits) -> Partition<Self> {
        Partition { offsets: splits }
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
    type Borders = BorderInfo;
    type Splits = OffsetSplits;
    type Offsets = NonZeroUsize;
    type Perfect = SinglePerfect;
    fn to_partition(splits: Self::Splits) -> Partition<Self> {
        Partition { offsets: splits }
    }
}
