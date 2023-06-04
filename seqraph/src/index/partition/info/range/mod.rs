pub mod role;
pub use role::*;
use crate::*;

#[derive(Debug, Clone)]
pub struct InnerRangeInfo<K: RangeRole> {
    pub range: K::Range,
    pub offsets: K::Offsets,
    pub children: K::Children,
}
impl<K: RangeRole> InnerRangeInfo<K> {
    pub fn join<'p>(
        &self,
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
impl<'p, K: RangeRole> RangeInfo<K> {
    pub fn join_pattern_inner(
        self,
        pattern_id: PatternId,
        ctx: &mut JoinContext<'p>,
    ) -> JoinedPattern {
        let loc = ctx.index.to_pattern_location(pattern_id);
        if let Some(inner_range) = self.inner_range {
            let index = match K::to_partition(
                inner_range.offsets.as_splits(ctx)
            ).join_partition(ctx) {
                Ok(inner) => {
                    // replace range and with new index
                    ctx.graph.replace_in_pattern(
                        loc,
                        self.inner_range.unwrap().range,
                        [inner.index],
                    );
                    inner.index
                }
                Err(p) => p
            };
            self.inner_range.unwrap().join(index)
        } else {
            ctx.graph.get_child_pattern_range(
                loc,
                self.range
            ).unwrap().into()
        }
    }
}