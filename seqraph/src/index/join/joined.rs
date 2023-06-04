use crate::*;
#[derive(Debug)]
pub struct JoinedPartition<K: RangeRole> {
    pub index: Child,
    pub perfect: K::Perfect,
    pub delta: PatternSubDeltas,
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
    pub patterns: Vec<JoinedPattern>,
    pub perfect: K::Perfect,
    pub delta: PatternSubDeltas,
}

impl<'p, K: RangeRole> JoinedPatterns<K> {
    pub fn patterns(&self) -> Vec<Pattern> {
        self.patterns.iter()
            .map(|p| p.into_pattern())
            .collect()
    }
    pub fn join(
        self,
        ctx: &mut JoinContext<'p>,
    ) -> JoinedPartition<K> {
        // collect infos about partition in each pattern
        let index = ctx.graph.insert_patterns(
            self.patterns()
        );
        JoinedPartition {
            index,
            perfect: self.perfect,
            delta: self.delta,
        }
    }
}
#[derive(Debug)]
pub enum JoinedPattern {
    Trigram([Child; 3]),
    Bigram([Child; 2]),
}
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
impl<'p> From<&[Child]> for JoinedPattern {
    fn from(value: &[Child]) -> Self {
        JoinedPattern::Bigram(
            value.try_into().expect("unmerged partition without inner range not a bigram")
        )
    }
}