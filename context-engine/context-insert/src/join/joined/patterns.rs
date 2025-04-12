use crate::{
    interval::partition::{
        delta::PatternSubDeltas,
        info::{
            PartitionInfo,
            border::perfect::{
                BorderPerfect,
                SinglePerfect,
            },
            range::role::RangeRole,
        },
    },
    join::{
        context::{
            node::context::NodeJoinContext,
            pattern::borders::JoinBorders,
        },
        partition::{
            Join,
            info::{
                JoinPartitionInfo,
                pattern_info::JoinPatternInfo,
            },
        },
    },
};
use context_search::graph::vertex::pattern::{
    Pattern,
    id::PatternId,
};

use super::partition::JoinedPartition;

#[derive(Debug)]
pub struct JoinedPatterns<R: RangeRole>
{
    pub patterns: Vec<Pattern>,
    pub perfect: R::Perfect,
    pub range: Option<R::Range>,
    pub delta: PatternSubDeltas,
}

impl<'a: 'b, 'b, R: RangeRole<Mode = Join> + 'a> JoinedPatterns<R>
where
    R::Borders: JoinBorders<R>,
{
    pub fn from_partition_info<'c>(
        info: JoinPartitionInfo<R>,
        ctx: &'c mut NodeJoinContext<'a>,
    ) -> Self
    where
        'b: 'c,
    {
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
        let range = if let SinglePerfect(Some(pid)) = info.perfect.complete()
        {
            Some(info.patterns[&pid].range.clone())
        }
        else
        {
            None
        };
        let perfect = info.perfect.clone();
        let (delta, patterns) = PartitionInfo::from(info)
            .patterns
            .into_iter()
            .map(|(pid, pinfo): (PatternId, JoinPatternInfo<_>)| {
                ((pid, pinfo.delta), pinfo.join_pattern(ctx, &pid))
            })
            .unzip();
        Self {
            patterns,
            perfect,
            range,
            delta,
        }
    }
    pub fn to_joined_partition(
        self,
        ctx: &'b mut NodeJoinContext<'a>,
    ) -> JoinedPartition<R>
    {
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
