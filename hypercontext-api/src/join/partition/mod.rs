pub mod borders;

use std::borrow::Borrow;

use borders::JoinBorders;
use derivative::Derivative;
use derive_more::derive::{
    Deref,
    DerefMut,
};

use crate::{
    graph::vertex::{
        child::Child,
        pattern::{
            id::PatternId,
            Pattern,
        },
    },
    join::{
        context::node::{
            context::NodeJoinContext,
            kind::{
                DefaultJoin,
                JoinKind,
            },
        },
        joined::{
            JoinedPartition,
            JoinedPatterns,
        },
    },
    partition::{
        context::AsNodeTraceContext, info::{
            border::{perfect::BoolPerfect, trace::TraceBorders, visit::VisitBorders, PartitionBorder},
            range::{
                children::RangeChildren, mode::{
                    RangeInfoOf,
                    VisitMode,
                }, role::{
                    BordersOf,
                    InVisitMode,
                    ModeChildren,
                    ModeChildrenOf,
                    ModeContext,
                    ModePatternCtxOf,
                    PostVisitMode,
                    PreVisitMode,
                    RangeRole,
                }, splits::RangeOffsets, InnerRangeInfo, ModeRangeInfo, PatternRangeInfo
            },
            visit::VisitPartition,
            PartitionInfo,
        }, pattern::{
            AsPatternContext,
            AsPatternTraceContext,
            PatternTraceContext,
        }, splits::SubSplits
    },
};

impl<R: RangeRole<Mode = Join>> ModeRangeInfo<R> for JoinRangeInfo<R>
where
    R::Borders: JoinBorders<R>,
{
    fn info_pattern_range(
        borders: BordersOf<R>,
        ctx: &ModePatternCtxOf<R>,
    ) -> Result<PatternRangeInfo<R>, Child> {
        let perfect = borders.perfect();
        let range = borders.outer_range();
        let offsets = borders.offsets();
        let inner = borders.inner_info(ctx);
        let (delta, pat, pid) = {
            let ctx = ctx.as_pattern_trace_context();
            let delta = inner
                .as_ref()
                .and_then(|inner| {
                    let inner_pat = ctx.pattern.get(inner.range.clone()).unwrap();
                    (inner_pat.len() != 1).then(|| inner_pat.len().saturating_sub(1))
                })
                .unwrap_or(0);
            let pat = ctx.pattern.get(range.clone()).unwrap();
            (delta, pat, ctx.loc.id)
        };
        let children = (!perfect.all_perfect()).then(|| borders.get_child_splits(ctx).unwrap());
        match (pat.len(), children) {
            (0, _) => panic!("Empty range"),
            (1, Some(children)) => Err(children.to_child().unwrap()),
            (1, None) => Err(pat[0]),
            (_, children) => Ok(PatternRangeInfo {
                pattern_id: pid,
                info: JoinRangeInfo {
                    inner_range: inner,
                    delta,
                    offsets,
                    range,
                    children,
                },
            }),
        }
    }
}
impl<R: RangeRole<Mode = Join>> InnerRangeInfo<R>
where
    R::Borders: JoinBorders<R>,
{
    pub fn index_pattern_inner<K: JoinKind>(
        &self,
        ctx: &mut NodeJoinContext<K>,
    ) -> Child {
        match self
            .offsets
            .as_splits(ctx.as_trace_context())
            .join_partition(ctx)
        {
            Ok(inner) => inner.index,
            Err(p) => p,
        }
    }
}

#[derive(Debug, Clone)]
pub struct JoinRangeInfo<R: RangeRole<Mode = Join>> {
    pub inner_range: Option<InnerRangeInfo<R>>,
    pub range: R::Range,
    pub children: Option<ModeChildrenOf<R>>,
    pub offsets: R::Offsets,
    pub delta: usize,
}

impl<R: RangeRole<Mode = Join>> JoinRangeInfo<R>
where
    R::Borders: JoinBorders<R>,
{
    pub fn joined_pattern<K: JoinKind>(
        self,
        ctx: &mut NodeJoinContext<'_, K>,
        pattern_id: &PatternId,
    ) -> Pattern {
        let inner = self.inner_range.map(|r| r.index_pattern_inner(ctx));
        match (inner, self.children) {
            (inner, Some(children)) => children.insert_inner(inner).unwrap(),
            (None, None) => ctx
                .graph
                .expect_pattern_range(ctx.index.to_pattern_location(*pattern_id), self.range)
                .into_pattern(),
            (Some(_), None) => panic!("inner range without children"),
            //let pat = ctx.pattern.get(range.clone()).unwrap();
        }
    }
}

impl<R: RangeRole<Mode = Self>> VisitMode<R> for Join
where
    R::Borders: JoinBorders<R>,
{
    type RangeInfo = JoinRangeInfo<R>;
}

#[derive(Debug, Deref, DerefMut, Derivative)]
#[derivative(Hash, PartialEq, Eq)]
pub struct PatternJoinContext<'p> {
    #[deref]
    #[deref_mut]
    pub ctx: PatternTraceContext<'p>,
    //pub graph: RwLockWriteGuard<'p, Hypergraph>,
    #[derivative(Hash = "ignore", PartialEq = "ignore")]
    pub sub_splits: &'p SubSplits,
}

impl<'p> AsPatternTraceContext<'p> for PatternJoinContext<'p> {
    fn as_pattern_trace_context<'t>(&'t self) -> PatternTraceContext<'t>
    where
        Self: 't,
        'p: 't,
    {
        self.ctx
    }
}

impl<'p> From<PatternJoinContext<'p>> for PatternId {
    fn from(value: PatternJoinContext<'p>) -> Self {
        Self::from(value.ctx)
    }
}
impl<'p, K: JoinKind + 'p> AsPatternContext<'p> for NodeJoinContext<'p, K> {
    type PatternCtx<'a>
        = PatternJoinContext<'a>
    where
        Self: 'a,
        'a: 'p;
    fn as_pattern_context<'t>(
        &'p self,
        pattern_id: &PatternId,
    ) -> Self::PatternCtx<'t>
    where
        Self: 't,
        't: 'p,
    {
        let ctx = PatternTraceContext {
            loc: self.index.to_pattern_location(*pattern_id),
            pattern: self.as_trace_context().patterns.get(pattern_id).unwrap(),
        };
        PatternJoinContext {
            ctx,
            sub_splits: self.borrow().sub_splits,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Join;
impl<'a> ModeContext<'a> for Join {
    type NodeResult = NodeJoinContext<'a, DefaultJoin>;
    type PatternResult = PatternJoinContext<'a>;
}

impl<R: RangeRole<Mode = Join>> ModeChildren<R> for Join {
    type Result = R::Children;
}

impl PreVisitMode for Join {}
impl PostVisitMode for Join {}
impl InVisitMode for Join {}

impl<'a, R: RangeRole<Mode = Join>> PartitionInfo<R>
where
    R::Borders: JoinBorders<R>,
{
    pub fn to_joined_patterns(
        self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> JoinedPatterns<R> {
        JoinedPatterns::from_partition_info(self, ctx)
    }
    pub fn to_joined_partition(
        self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> JoinedPartition<R> {
        JoinedPartition::from_partition_info(self, ctx)
    }
}

pub trait JoinPartition<R: RangeRole<Mode = Join>>: VisitPartition<R>
where
    R::Borders: JoinBorders<R>,
{
    fn join_partition(
        self,
        ctx: &mut NodeJoinContext<'_>,
    ) -> Result<JoinedPartition<R>, Child> {
        match self.info_partition(ctx) {
            Ok(info) => Ok(JoinedPartition::from_partition_info(info, ctx)),
            Err(c) => Err(c),
        }
    }
}

impl<R: RangeRole<Mode = Join>, P: VisitPartition<R>> JoinPartition<R> for P where
    R::Borders: JoinBorders<R>
{
}
