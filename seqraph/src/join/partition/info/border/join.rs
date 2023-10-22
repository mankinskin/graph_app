use crate::*;

pub trait JoinBorders<'a, K: RangeRole<Mode = Join>>: VisitBorders<'a, K> {
    fn get_child_splits(
        &self,
        ctx: &ModePatternCtxOf<'a, K>,
    ) -> Option<ModeChildrenOf<K>>;
}
impl<'a> JoinBorders<'a, Post<Join>> for BorderInfo {
    fn get_child_splits(
        &self,
        ctx: &PatternJoinContext<'a>,
    ) -> Option<ChildrenOf<Post<Join>>> {
        self.inner_offset.map(|o|
            ctx.sub_splits.get(&SplitKey::new(ctx.pattern[self.sub_index], o)).unwrap().right
        )
    }
}

impl<'a> JoinBorders<'a, Pre<Join>> for BorderInfo {
    fn get_child_splits(
        &self,
        ctx: &PatternJoinContext<'a>,
    ) -> Option<ChildrenOf<Pre<Join>>> {
        self.inner_offset.map(|o|
            ctx.sub_splits.get(&SplitKey::new(ctx.pattern[self.sub_index], o)).unwrap().left
        )
    }
}
impl<'a> JoinBorders<'a, In<Join>> for (BorderInfo, BorderInfo) {
    fn get_child_splits(
        &self,
        ctx: &PatternJoinContext<'a>,
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