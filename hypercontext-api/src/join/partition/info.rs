use crate::{
    graph::vertex::{
        child::Child,
        pattern::{
            id::PatternId,
            IntoPattern,
            Pattern,
        },
    },
    join::context::node::context::NodeJoinContext,
    partition::{
        context::AsNodeTraceContext, info::{
            border::{
                perfect::BoolPerfect,
                trace::TraceBorders,
                visit::VisitBorders,
                PartitionBorder,
            },
            range::{
                children::RangeChildren, role::{
                    BordersOf,
                    ModeChildrenOf,
                    ModePatternCtxOf,
                    RangeRole,
                }, splits::RangeOffsets, InnerRangeInfo, ModeRangeInfo, PatternRangeInfo
            },
        }, pattern::HasPatternTraceContext
    },
};

use super::{
    borders::JoinBorders, join::JoinPartition, Join
};

#[derive(Debug, Clone)]
pub struct JoinPatternInfo<R: RangeRole<Mode = Join>> {
    pub inner_range: Option<InnerRangeInfo<R>>,
    pub range: R::Range,
    pub children: Option<ModeChildrenOf<R>>,
    pub offsets: R::Offsets,
    pub delta: usize,
}

impl<R: RangeRole<Mode = Join>> JoinPatternInfo<R>
where
    R::Borders: JoinBorders<R>,
{
    pub fn join_pattern<'a: 'b, 'b: 'c, 'c>(
        self,
        ctx: &'c mut NodeJoinContext<'a, 'b>,
        pattern_id: &PatternId,
    ) -> Pattern
        where R: 'a
    {
        let index = ctx.index;
        let inner = self.inner_range.map(|r| r.index_pattern_inner(ctx));
        match (inner, self.children) {
            (inner, Some(children)) => children.insert_inner(inner).unwrap(),
            (None, None) => ctx
                .graph
                .expect_pattern_range(index.to_pattern_location(*pattern_id), self.range)
                .into_pattern(),
            (Some(_), None) => panic!("inner range without children"),
            //let pat = ctx.pattern.get(range.clone()).unwrap();
        }
    }
}

impl<R: RangeRole<Mode = Join>> ModeRangeInfo<R> for JoinPatternInfo<R>
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
            let ctx = ctx.pattern_trace_context();
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
                info: JoinPatternInfo {
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
    pub fn index_pattern_inner<'a: 'b, 'b: 'c, 'c>(
        &self,
        ctx: &'c mut NodeJoinContext<'a, 'b>,
    ) -> Child
        where Self: 'a
    {
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