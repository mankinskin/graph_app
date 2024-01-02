pub mod role;
pub use role::*;
pub mod children;
pub use children::*;
pub mod splits;
pub use splits::*;
pub mod mode;
pub use mode::*;
use crate::shared::*;

#[derive(Debug)]
pub struct PatternRangeInfo<K: RangeRole> {
    pub pattern_id: PatternId,
    pub info: RangeInfoOf<K>,
}

pub trait ModeRangeInfo<K: RangeRole>: Debug {
    fn info_pattern_range<'a>(
        borders: BordersOf<K>,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Result<PatternRangeInfo<K>, Child>;
}
impl<K: RangeRole<Mode=Trace>> ModeRangeInfo<K> for TraceRangeInfo<K> {

    fn info_pattern_range<'a>(
        borders: BordersOf<K>,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Result<PatternRangeInfo<K>, Child> {
        let range = borders.outer_range();
        let inner = borders.inner_info(ctx);
        let (pat, pid) = {
            let ctx = ctx.as_pattern_trace_context();
            let pat = ctx.pattern.get(range.clone()).unwrap();
            (pat, ctx.loc.id)
        };
        if pat.len() != 1 {
            Ok(PatternRangeInfo {
                pattern_id: pid,
                info: TraceRangeInfo {
                    inner_range: inner,
                },
            })
        } else {
            Err(pat[0])
        }

    }
}
impl<K: RangeRole<Mode=Join>> ModeRangeInfo<K> for JoinRangeInfo<K>
    where K::Borders: JoinBorders<K>
{
    fn info_pattern_range<'a>(
        borders: BordersOf<K>,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Result<PatternRangeInfo<K>, Child> {
        let perfect = borders.perfect();
        let range = borders.outer_range();
        let offsets = borders.offsets();
        let inner = borders.inner_info(ctx);
        let (delta, pat, pid) = {
            let ctx = ctx.as_pattern_trace_context();
            let delta = inner.as_ref()
                .and_then(|inner| {
                    let inner_pat = ctx.pattern.get(inner.range.clone()).unwrap();
                    (inner_pat.len() != 1)
                        .then(|| inner_pat.len().saturating_sub(1))
                })
                .unwrap_or(0);
            let pat = ctx.pattern.get(range.clone()).unwrap();
            (delta, pat, ctx.loc.id)
        };
        let children = (!perfect.all_perfect()).then(||
            borders.get_child_splits(ctx).unwrap()
        );
        match (pat.len(), children) {
            (0, _) => panic!("Empty range"),
            (1, Some(children)) =>
                Err(children.to_child().unwrap()),
            (1, None) => Err(pat[0]),
            (_, children) =>
                Ok(PatternRangeInfo {
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

#[derive(Debug, Clone)]
pub struct InnerRangeInfo<K: RangeRole> {
    pub range: K::Range,
    pub offsets: K::Offsets,
}
impl<'a, K: RangeRole<Mode = Join>> InnerRangeInfo<K>
    where K::Borders: JoinBorders<K>
{
    pub fn index_pattern_inner<'t>(
        &self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> Child {
        match self.offsets
            .as_splits(ctx.as_trace_context())
            .join_partition(ctx)
        {
            Ok(inner) => inner.index,
            Err(p) => p
        }
    }
}
#[derive(Debug, Clone)]
pub struct TraceRangeInfo<K: RangeRole<Mode=Trace>> {
    pub inner_range: Option<InnerRangeInfo<K>>,
}
#[derive(Debug, Clone)]
pub struct JoinRangeInfo<K: RangeRole<Mode=Join>> {
    pub inner_range: Option<InnerRangeInfo<K>>,
    pub range: K::Range,
    pub children: Option<ModeChildrenOf<K>>,
    pub offsets: K::Offsets,
    pub delta: usize,
}
impl<'a, K: RangeRole<Mode = Join>> JoinRangeInfo<K>
    where K::Borders: JoinBorders<K>
{
    pub fn joined_pattern<'t>(
        self,
        ctx: &mut NodeJoinContext<'a>,
        pattern_id: &PatternId,
    ) -> Pattern {
        let inner = self.inner_range.map(|r|
            r.index_pattern_inner(ctx)
        );
        match (inner, self.children) {
            (inner, Some(children)) =>
                children.insert_inner(inner).unwrap(),
            (None, None) =>
                ctx.graph.expect_pattern_range(
                    ctx.index.to_pattern_location(*pattern_id),
                    self.range,
                ).into_pattern(),
            (Some(_), None) =>
                panic!("inner range without children"),
            //let pat = ctx.pattern.get(range.clone()).unwrap();
        }
    }
}