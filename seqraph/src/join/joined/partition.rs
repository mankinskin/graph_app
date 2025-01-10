use std::borrow::Borrow;

use crate::join::{
        context::node::context::NodeJoinContext,
        partition::{
            borders::JoinBorders,
            Join, JoinPartitionInfo,
        },
    };
use hypercontext_api::{
    graph::vertex::child::Child,
    partition::{
        delta::PatternSubDeltas,
        info::{
            border::perfect::{
                BorderPerfect,
                SinglePerfect,
            },
            range::role::RangeRole,
            PartitionInfo,
        },
    },
};

use super::patterns::JoinedPatterns;

#[derive(Debug)]
pub struct JoinedPartition<R: RangeRole> {
    pub index: Child,
    pub perfect: R::Perfect,
    pub delta: PatternSubDeltas,
}

impl<'a: 'b, 'b: 'c, 'c, R: RangeRole<Mode = Join> + 'a> JoinedPartition<R>
where
    R::Borders: JoinBorders<R>,
{
    pub fn from_joined_patterns(
        pats: JoinedPatterns<R>,
        ctx: &'c mut NodeJoinContext<'a>,
    ) -> Self {
        // collect infos about partition in each pattern
        let index = ctx.trav.insert_patterns(pats.patterns);
        // todo: replace if perfect
        if let SinglePerfect(Some(pid)) = pats.perfect.complete() {
            let loc = ctx.index.to_pattern_location(pid);
            ctx.trav
                .replace_in_pattern(loc, pats.range.unwrap(), index);
        }
        Self {
            index,
            perfect: pats.perfect,
            delta: pats.delta,
        }
    }
    pub fn from_partition_info(
        info: JoinPartitionInfo<R>,
        ctx: &'c mut NodeJoinContext<'a>,
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
