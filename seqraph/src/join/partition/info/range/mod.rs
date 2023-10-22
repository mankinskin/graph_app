pub mod role;
pub use role::*;
pub mod children;
pub use children::*;
pub mod splits;
pub use splits::*;
use crate::*;

#[derive(Debug)]
pub struct PatternRangeInfo<K: RangeRole> {
    pub pattern_id: PatternId,
    pub info: RangeInfo<K>,
    pub perfect: <K::Perfect as BorderPerfect>::Boolean,
}

#[derive(Debug, Clone)]
pub struct InnerRangeInfo<K: RangeRole>
    where K::Mode: ModeChildren::<K>
{
    pub range: K::Range,
    pub offsets: K::Offsets,
}
/// created for unmerged ranges
#[derive(Debug, Clone)]
pub struct RangeInfo<K: RangeRole> {
    pub inner_range: Option<InnerRangeInfo<K>>,
    pub range: K::Range,
    pub children: ModeChildrenOf<K>,
    pub offsets: K::Offsets,
    pub delta: usize,
}
impl<'a, K: RangeRole<Mode = Join>> RangeInfo<K> {
    //pub fn pattern_with_inner(
    //    self,
    //    inner: Child,
    //) -> JoinedPattern {
    //    self.children.insert_inner(inner)
    //}
    pub fn joined_pattern<'t>(
        self,
        ctx: &mut NodeJoinContext<'a>,
        pattern_id: &PatternId,
    ) -> JoinedPattern {
        let inner = self.index_pattern_inner(ctx, pattern_id);
        self.children.insert_inner(inner).unwrap()
    }
    pub fn index_pattern_inner<'t>(
        &self,
        ctx: &mut NodeJoinContext<'a>,
        pattern_id: &PatternId,
    ) -> Option<Child> {
        if let Some(inner_range) = &self.inner_range {
            let index = match inner_range.offsets
                .as_splits(ctx.as_trace_context())
                .join_partition(ctx)
            {
                Ok(inner) => {
                    // replace range and with new index
                    let loc = ctx.index.to_pattern_location(*pattern_id);
                    ctx.graph.replace_in_pattern(
                        loc,
                        inner_range.range.clone(),
                        inner.index,
                    );
                    inner.index
                }
                Err(p) => p
            };
            Some(index)
        } else {
            None
        }
    }
}