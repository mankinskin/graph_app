use std::{
    fmt::Debug,
    num::NonZeroUsize,
    ops::{
        Range,
        RangeFrom,
    },
};

use crate::{
    graph::vertex::child::Child,
    partition::{
        info::{
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
                mode::ModeInfo,
                splits::{
                    PatternSplits,
                    RangeOffsets,
                },
            },
        },
        splits::offset::OffsetSplit,
        Partition,
        ToPartition,
    },
};

use super::{
    mode::{
        InVisitMode,
        ModeChildren,
        ModeContext,
        PostVisitMode,
        PreVisitMode,
    },
    splits::OffsetIndexRange,
};

#[derive(Debug, Clone, Copy)]
pub struct Outer;

#[derive(Debug, Clone, Copy)]
pub struct Inner;

pub type OffsetsOf<R> = <R as RangeRole>::Offsets;
pub type PerfectOf<R> = <R as RangeRole>::Perfect;
pub type BooleanPerfectOf<R> = <PerfectOf<R> as BorderPerfect>::Boolean;
pub type ChildrenOf<R> = <R as RangeRole>::Children;
pub type RangeOf<R> = <R as RangeRole>::Range;
pub type ModeOf<R> = <R as RangeRole>::Mode;
pub type BordersOf<R> = <R as RangeRole>::Borders;
pub type ModeChildrenOf<R> = <ModeOf<R> as ModeChildren<R>>::Result;
pub type ModePatternCtxOf<'a, R> = <<R as RangeRole>::Mode as ModeContext>::PatternResult<'a>;
pub type ModeNodeCtxOf<'a, 'b, R> = <<R as RangeRole>::Mode as ModeContext>::NodeContext<'a, 'b>;

pub trait RangeKind: Debug + Clone {}

impl RangeKind for Inner {}

impl RangeKind for Outer {}

pub trait RangeRole: Debug + Clone + Copy {
    type Mode: ModeInfo<Self>; // todo: use to change join/trace
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

#[derive(Debug, Clone, Default, Copy)]
pub struct Pre<M: PreVisitMode>(std::marker::PhantomData<M>);

impl<M: PreVisitMode> RangeRole for Pre<M> {
    type Mode = M;
    type Range = Range<usize>;
    type Kind = Outer;
    type Children = Child;
    type PartitionSplits = ((), OffsetSplit);
    type Borders = BorderInfo;
    type Splits = OffsetSplit;
    type Offsets = NonZeroUsize;
    type Perfect = SinglePerfect;
    fn to_partition(splits: Self::Splits) -> Partition<Self> {
        Partition { offsets: splits }
    }
}

#[derive(Debug, Clone, Default, Copy)]
pub struct In<M: InVisitMode>(std::marker::PhantomData<M>);

impl<M: InVisitMode> RangeRole for In<M> {
    type Mode = M;
    type Range = Range<usize>;
    type Kind = Inner;
    type Children = InfixChildren;
    type PartitionSplits = (OffsetSplit, OffsetSplit);
    type Borders = (BorderInfo, BorderInfo);
    type Splits = (OffsetSplit, OffsetSplit);
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
    type PartitionSplits = (OffsetSplit, ());
    type Borders = BorderInfo;
    type Splits = OffsetSplit;
    type Offsets = NonZeroUsize;
    type Perfect = SinglePerfect;
    fn to_partition(splits: Self::Splits) -> Partition<Self> {
        Partition { offsets: splits }
    }
}