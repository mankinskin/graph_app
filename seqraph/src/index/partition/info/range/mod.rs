pub mod role;
pub use role::*;
pub mod children;
pub use children::*;
pub mod splits;
pub use splits::*;
use crate::*;

#[derive(Debug, Clone)]
pub struct InnerRangeInfo<K: RangeRole>
    where K::Mode: ModeChildren::<K>
{
    pub range: K::Range,
    pub offsets: K::Offsets,
    pub children: ModeChildrenOf<K>,
}
impl<K: RangeRole<Mode = Join>> InnerRangeInfo<K> {
    pub fn join_inner(
        self,
        inner: Child,
    ) -> JoinedPattern {
        self.children.join_inner(inner)
    }
}
/// created for unmerged ranges
#[derive(Debug, Clone)]
pub struct RangeInfo<K: RangeRole> {
    pub inner_range: Option<InnerRangeInfo<K>>,
    pub range: K::Range,
    pub offsets: K::Offsets,
    pub delta: usize,
}
impl<'a, K: RangeRole<Mode = Join>> RangeInfo<K> {
    pub fn join_pattern_inner<'t>(
        self,
        pattern_id: PatternId,
        ctx: &mut JoinContext<'a>,
    ) -> JoinedPattern
    {
        let loc = ctx.index.to_pattern_location(pattern_id);
        if let Some(inner_range) = self.inner_range {
            let index = match inner_range.offsets
                .as_splits(&*ctx)
                .join_partition(ctx)
            {
                Ok(inner) => {
                    // replace range and with new index
                    ctx.graph.replace_in_pattern(
                        loc,
                        inner_range.range.clone(),
                        inner.index,
                    );
                    inner.index
                }
                Err(p) => p
            };
            inner_range.join_inner(index)
        } else {
            ctx.graph.get_child_pattern_range(
                loc,
                self.range
            ).unwrap().into()
        }
    }
}