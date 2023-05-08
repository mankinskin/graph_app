use crate::*;

#[derive(Debug, Default)]
pub struct PartitionBundle {
    pub bundle: Vec<PatternPartitionInfo>,
    pub perfect: (Option<PatternId>, Option<PatternId>),
}
impl FromIterator<(PatternPartitionInfo, Perfect)> for PartitionBundle {
    fn from_iter<T: IntoIterator<Item = (PatternPartitionInfo, Perfect)>>(iter: T) -> Self {
        let (mut l, mut r) = (None, None);
        PartitionBundle {
            bundle: iter.into_iter()
                .map(|(info, (pl, pr))| {
                    l = l.or(pl);
                    r = r.or(pr);
                    info
                })
                .collect(),
            perfect: (l, r),
        }
    }
}
impl<'p> PartitionBundle {
    pub fn join_patterns(
        self,
        ctx: &mut Partitioner<'p>,
    ) -> IndexedPatterns {
        let patterns = self.bundle.iter()
            .map(|info| {
                info.join(ctx)
            })
            .collect_vec();
        IndexedPatterns {
            patterns,
            perfect: self.perfect
        }
    }
    pub fn join(
        self,
        ctx: &mut Partitioner<'p>,
    ) -> IndexedPartition {
        // collect infos about partition in each pattern
        let patterns = self.join_patterns(ctx);
        let index = ctx.graph.insert_patterns(
            patterns.patterns.into_iter().map(|p|
                (p.borrow() as &[Child]).into_iter().cloned().collect_vec()
            )
        );
        IndexedPartition {
            index,
            perfect: patterns.perfect,
        }
    }
}
#[derive(Debug)]
pub struct PatternPartitionInfo {
    pub pattern_id: PatternId,
    pub inner_range: Option<InnerRangeInfo>,
    pub left: Child,
    pub right: Child,
}
#[derive(Debug)]
pub enum JoinedPattern {
    Trigram([Child; 3]),
    Bigram([Child; 2]),
}
impl Borrow<[Child]> for JoinedPattern {
    fn borrow(&self) -> &[Child] {
        match self {
            Self::Trigram(p) => p.borrow(),
            Self::Bigram(p) => p.borrow(),
        }
    }
}
impl Deref for JoinedPattern {
    type Target = [Child];
    fn deref(&self) -> &Self::Target {
        match self {
            Self::Trigram(p) => p,
            Self::Bigram(p) => p,
        }
    }
}
pub type Perfect = (Option<PatternId>, Option<PatternId>);
impl<'p> PatternPartitionInfo {
    /// index inner range and re
    pub fn join(
        &self,
        ctx: &mut Partitioner<'p>,
    ) -> JoinedPattern {
        if let Some(context) = self.inner_range.as_ref().map(|range_info| {
            // index inner range
            let inner = ctx.index_partition(
                range_splits(ctx.patterns().iter(), range_info.offsets)
            );
            // replace range and with new index
            let loc = ctx.index.to_pattern_location(self.pattern_id);
            ctx.graph.insert_range_in(
                loc,
                range_info.range.clone(),
            ).unwrap()
        }) {
            JoinedPattern::Trigram([self.left, context, self.right])
        } else {
            JoinedPattern::Bigram([self.left, self.right])
        }
    }
}