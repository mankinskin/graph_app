use crate::{
    graph::vertex::{
        child::Child,
        pattern::{
            id::PatternId,
            Pattern,
            IntoPattern,
        },
    },
    join::{
        context::node::{
            context::NodeJoinContext,
        },
    },
    partition::{
        context::AsNodeTraceContext, info::{
            border::{perfect::BoolPerfect, trace::TraceBorders, visit::VisitBorders, PartitionBorder},
            range::{
                children::RangeChildren,
                role::{
                    BordersOf,
                    ModeChildrenOf,
                    ModePatternCtxOf,
                    RangeRole,
                }, InnerRangeInfo, ModeRangeInfo, PatternRangeInfo
            },
        }, pattern::{
            AsPatternTraceContext,
        },
    },
};

use super::{borders::JoinBorders, Join};
pub mod inner;

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
    pub fn joined_pattern(
        self,
        ctx: &mut NodeJoinContext<'_>,
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