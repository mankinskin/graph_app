use crate::*;

#[derive(Debug, Default)]
pub struct PartitionBundle {
    pub bundle: Vec<PatternPartitionInfo>,
    pub perfect: Perfect,
    pub delta: PatternSubDeltas,
}

#[derive(Debug, Default, IntoIterator)]
pub struct PatternSubDeltas {
    pub inner: PatternSubDeltasInner,
}
type PatternSubDeltasInner = HashMap<PatternId, usize>;
impl Deref for PatternSubDeltas {
    type Target = PatternSubDeltasInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl FromIterator<(PatternId, usize)> for PatternSubDeltas {
    fn from_iter<T: IntoIterator<Item = (PatternId, usize)>>(iter: T) -> Self {
        Self {
            inner: FromIterator::from_iter(iter),
        }
    }
}
impl Extend<(PatternId, usize)> for PatternSubDeltas {
    fn extend<T: IntoIterator<Item = (PatternId, usize)>>(&mut self, iter: T) {
        self.inner.extend(iter)
    }
}
impl std::ops::Add for PatternSubDeltas {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        self.into_iter()
            .map(|(pid, a)|
                (pid, a + rhs[&pid])
            )
            .collect()
    }
}

impl FromIterator<PatternPartitionResult> for PartitionBundle {
    fn from_iter<T: IntoIterator<Item = PatternPartitionResult>>(iter: T) -> Self {
        let (mut l, mut r) = (None, None);
        let (bundle, delta) =
            iter.into_iter()
                .map(|PatternPartitionResult {
                    info,
                    perfect: (pl, pr),
                    delta,
                }| {
                    l = l.or(pl);
                    r = r.or(pr);
                    let dmap = (info.pattern_id, delta);
                    (info, dmap)
                })
                .unzip();
        PartitionBundle {
            bundle,
            perfect: (l, r),
            delta,
        }
    }
}
impl<'p> PartitionBundle {
    pub fn join_patterns(
        self,
        ctx: &mut Partitioner<'p>,
    ) -> JoinedPatterns {
        let patterns = self.bundle.iter()
            .map(|info| {
                info.join(ctx)
            })
            .collect_vec();
        JoinedPatterns {
            patterns,
            perfect: self.perfect,
            delta: self.delta,
        }
    }
    pub fn join(
        self,
        ctx: &mut Partitioner<'p>,
    ) -> JoinedPartition {
        // collect infos about partition in each pattern
        self.join_patterns(ctx).join(ctx)
    }
}
#[derive(Debug)]
pub enum JoinedPattern {
    Trigram([Child; 3]),
    Bigram([Child; 2]),
}
impl<'p> Borrow<[Child]> for &'p JoinedPattern {
    fn borrow(&self) -> &[Child] {
        match self {
            JoinedPattern::Trigram(p) => p.borrow(),
            JoinedPattern::Bigram(p) => p.borrow(),
        }
    }
}
impl<'p> IntoIterator for &'p JoinedPattern {
    type Item = &'p Child;
    type IntoIter = std::slice::Iter<'p, Child>;
    fn into_iter(self) -> Self::IntoIter {
        match self {
            JoinedPattern::Trigram(p) => p.into_iter(),
            JoinedPattern::Bigram(p) => p.into_iter(),
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
#[derive(Debug)]
pub struct PatternPartitionInfo {
    pub pattern_id: PatternId,
    pub inner_range: Option<InnerRangeInfo>,
    pub left: Child,
    pub right: Child,
}
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