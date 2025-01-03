use crate::{
    join::context::pattern::PatternJoinContext, partition::info::{
        border::{
            trace::TraceBorders,
            BorderInfo,
        },
        range::{
            children::InfixChildren,
            role::{
                ChildrenOf, In, ModeChildrenOf, ModePatternCtxOf, Post, Pre, RangeRole
            },
        },
    }, traversal::cache::key::SplitKey
};

use super::Join;

pub trait JoinBorders<R: RangeRole<Mode = Join>>: TraceBorders<R> {
    fn get_child_splits(
        &self,
        ctx: &ModePatternCtxOf<'_, R>,
    ) -> Option<ModeChildrenOf<R>>;
}

impl JoinBorders<Post<Join>> for BorderInfo {
    fn get_child_splits(
        &self,
        ctx: &PatternJoinContext<'_>,
    ) -> Option<ChildrenOf<Post<Join>>> {
        self.inner_offset.map(|o| {
            ctx.sub_splits
                .get(&SplitKey::new(ctx.pattern[self.sub_index], o))
                .unwrap()
                .right
        })
    }
}

impl JoinBorders<Pre<Join>> for BorderInfo {
    fn get_child_splits(
        &self,
        ctx: &PatternJoinContext<'_>,
    ) -> Option<ChildrenOf<Pre<Join>>> {
        self.inner_offset.map(|o| {
            ctx.sub_splits
                .get(&SplitKey::new(ctx.pattern[self.sub_index], o))
                .unwrap()
                .left
        })
    }
}

impl JoinBorders<In<Join>> for (BorderInfo, BorderInfo) {
    fn get_child_splits(
        &self,
        ctx: &PatternJoinContext<'_>,
    ) -> Option<ChildrenOf<In<Join>>> {
        let (lc, rc) = (ctx.pattern[self.0.sub_index], ctx.pattern[self.1.sub_index]);
        match (self.0.inner_offset, self.1.inner_offset) {
            (Some(l), Some(r)) => Some(InfixChildren::Both(
                ctx.sub_splits.get(&SplitKey::new(lc, l)).unwrap().right,
                ctx.sub_splits.get(&SplitKey::new(rc, r)).unwrap().left,
            )),
            (None, Some(r)) => Some(InfixChildren::Right(
                ctx.sub_splits.get(&SplitKey::new(rc, r)).unwrap().left,
            )),
            (Some(l), None) => Some(InfixChildren::Left(
                ctx.sub_splits.get(&SplitKey::new(lc, l)).unwrap().right,
            )),
            (None, None) => None,
        }
    }
}
