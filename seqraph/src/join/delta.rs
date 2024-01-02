use crate::shared::*;

#[derive(Debug, Default, IntoIterator, DerefMut)]
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
