use crate::*;

#[derive(Debug, Clone)]
pub enum InfixChildren {
    Both(Child, Child),
    Left(Child),
    Right(Child),
}
pub trait RangeChildren<K: RangeRole>: Debug + Clone {
    fn join_inner(self, inner: Child) -> JoinedPattern;
}
impl<M: PreVisitMode> RangeChildren<Pre<M>> for Child {
    fn join_inner(self, inner: Child) -> JoinedPattern {
        JoinedPattern::Bigram([inner, self])
    }
}
impl<M: PostVisitMode> RangeChildren<Post<M>> for Child {
    fn join_inner(self, inner: Child) -> JoinedPattern {
        JoinedPattern::Bigram([inner, self])
    }
}
impl<M: InVisitMode> RangeChildren<In<M>> for InfixChildren {
    fn join_inner(self, inner: Child) -> JoinedPattern {
        match self {
            Self::Both(l, r) =>
                JoinedPattern::Trigram([l, inner, r]),
            Self::Left(l) =>
                JoinedPattern::Bigram([l, inner]),
            Self::Right(r) =>
                JoinedPattern::Bigram([inner, r]),
        }
    }
}