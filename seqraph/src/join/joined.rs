use std::borrow::Borrow;

use crate::join::{
    context::node::context::NodeJoinContext,
    delta::PatternSubDeltas,
    partition::info::{
        border::{
            join::JoinBorders,
            perfect::{
                BorderPerfect,
                SinglePerfect,
            },
        },
        PartitionInfo,
        range::role::{
            Join,
            RangeRole,
        },
    },
};
use hypercontext_api::graph::vertex::{
    child::Child,
    pattern::Pattern,
};

#[derive(Debug)]
pub struct JoinedPartition<K: RangeRole> {
    pub index: Child,
    pub perfect: K::Perfect,
    pub delta: PatternSubDeltas,
}

impl<'a, K: RangeRole<Mode = Join>> JoinedPartition<K>
where
    K::Borders: JoinBorders<K>,
{
    pub fn from_joined_patterns(
        pats: JoinedPatterns<K>,
        ctx: &mut NodeJoinContext<'a>,
    ) -> Self {
        // collect infos about partition in each pattern
        let index = ctx.graph.insert_patterns(pats.patterns);
        // todo: replace if perfect
        if let SinglePerfect(Some(pid)) = pats.perfect.complete() {
            let loc = ctx.index.to_pattern_location(pid);
            ctx.graph
                .replace_in_pattern(loc, pats.range.unwrap(), index);
        }
        Self {
            index,
            perfect: pats.perfect,
            delta: pats.delta,
        }
    }
    pub fn from_partition_info(
        info: PartitionInfo<K>,
        ctx: &mut NodeJoinContext<'a>,
    ) -> Self {
        // collect infos about partition in each pattern
        let pats = JoinedPatterns::from_partition_info(info, ctx);
        Self::from_joined_patterns(pats, ctx)
    }
}

impl<K: RangeRole> Borrow<Child> for JoinedPartition<K> {
    fn borrow(&self) -> &Child {
        &self.index
    }
}

impl<K: RangeRole> Borrow<Child> for &JoinedPartition<K> {
    fn borrow(&self) -> &Child {
        &self.index
    }
}

#[derive(Debug)]
pub struct JoinedPatterns<K: RangeRole> {
    pub patterns: Vec<Pattern>,
    pub perfect: K::Perfect,
    pub range: Option<K::Range>,
    pub delta: PatternSubDeltas,
}

impl<'a, K: RangeRole<Mode = Join>> JoinedPatterns<K>
where
    K::Borders: JoinBorders<K>,
{
    pub fn from_partition_info(
        info: PartitionInfo<K>,
        ctx: &mut NodeJoinContext<'a>,
    ) -> Self {
        // assert: no complete perfect child
        // todo: index inner ranges and get child splits
        //
        // index inner range
        // cases:
        // - (child, inner, child)
        // - (child, inner),
        // - (inner, child),
        // - (child, child),
        // - child: not possible, handled earlier
        let range = if let SinglePerfect(Some(pid)) = info.perfect.complete() {
            Some(info.patterns[&pid].range.clone())
        } else {
            None
        };
        let (delta, patterns) = info
            .patterns
            .into_iter()
            .map(|(pid, pinfo)| ((pid, pinfo.delta), pinfo.joined_pattern(ctx, &pid)))
            .unzip();
        Self {
            patterns,
            perfect: info.perfect,
            range,
            delta,
        }
    }
    pub fn to_joined_partition(
        self,
        ctx: &mut NodeJoinContext<'a>,
    ) -> JoinedPartition<K> {
        JoinedPartition::from_joined_patterns(self, ctx)
    }
}
//#[derive(Debug)]
//pub enum JoinedPattern {
//    Trigram([Child; 3]),
//    Bigram([Child; 2]),
//}
//impl From<BorderChildren<JoinedRangeInfoKind>> for JoinedPattern {
//    fn from(borders: BorderChildren<JoinedRangeInfoKind>) -> Self {
//        match borders {
//            BorderChildren::Infix(left, right, None) =>
//                JoinedPattern::Bigram([left, right]),
//            BorderChildren::Infix(left, right, Some(inner)) =>
//                JoinedPattern::Trigram([left, inner, right]),
//            BorderChildren::Prefix(inner, right) =>
//                JoinedPattern::Bigram([inner, right]),
//            BorderChildren::Postfix(left, inner) =>
//                JoinedPattern::Bigram([left, inner]),
//        }
//    }
//}
//impl<'p> Borrow<[Child]> for &'p JoinedPattern {
//    fn borrow(&self) -> &[Child] {
//        match self {
//            JoinedPattern::Trigram(p) => p.borrow(),
//            JoinedPattern::Bigram(p) => p.borrow(),
//        }
//    }
//}
//impl<'p> IntoIterator for &'p JoinedPattern {
//    type Item = &'p Child;
//    type IntoIter = std::slice::Iter<'p, Child>;
//    fn into_iter(self) -> Self::IntoIter {
//        match self {
//            JoinedPattern::Trigram(p) => p.into_iter(),
//            JoinedPattern::Bigram(p) => p.into_iter(),
//        }
//    }
//}
//impl Deref for JoinedPattern {
//    type Target = [Child];
//    fn deref(&self) -> &Self::Target {
//        match self {
//            Self::Trigram(p) => p,
//            Self::Bigram(p) => p,
//        }
//    }
//}
//impl<'p> From<&[Child]> for JoinedPattern {
//    fn from(value: &[Child]) -> Self {
//        JoinedPattern::Bigram(
//            value.try_into().expect("unmerged partition without inner range not a bigram")
//        )
//    }
//}
