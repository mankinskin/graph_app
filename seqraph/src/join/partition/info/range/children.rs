use crate::*;

#[derive(Debug, Clone)]
pub enum InfixChildren {
    Both(Child, Child),
    Left(Child),
    Right(Child),
}
impl InfixChildren {
    pub fn to_joined_pattern(self) -> Result<JoinedPattern, Child> {
        match self {
            InfixChildren::Both(l, r) => Ok(
                JoinedPattern::Bigram([l, r])
            ),
            InfixChildren::Left(c) |
            InfixChildren::Right(c) => Err(c),
        }
    }
}
pub trait RangeChildren<K: RangeRole>: Debug + Clone {
    fn insert_inner(self, inner: Option<Child>) -> Result<JoinedPattern, Child>;
}
impl<M: PreVisitMode> RangeChildren<Pre<M>> for Child {
    fn insert_inner(self, inner: Option<Child>) -> Result<JoinedPattern, Child> {
        if let Some(inner) = inner {
            Ok(JoinedPattern::Bigram([inner, self]))
        } else {
            Err(self)
        }
    }
}
impl<M: PostVisitMode> RangeChildren<Post<M>> for Child {
    fn insert_inner(self, inner: Option<Child>) -> Result<JoinedPattern, Child> {
        if let Some(inner) = inner {
            Ok(JoinedPattern::Bigram([inner, self]))
        } else {
            Err(self)
        }
    }
}
impl<M: InVisitMode> RangeChildren<In<M>> for InfixChildren {
    fn insert_inner(self, inner: Option<Child>) -> Result<JoinedPattern, Child> {
        if let Some(inner) = inner {
            Ok(match self {
                Self::Both(l, r) =>
                    JoinedPattern::Trigram([l, inner, r]),
                Self::Left(l) =>
                    JoinedPattern::Bigram([l, inner]),
                Self::Right(r) =>
                    JoinedPattern::Bigram([inner, r]),
            })
        } else {
            self.to_joined_pattern()
        }
    }
}