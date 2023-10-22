use crate::*;

pub trait BoolPerfect: Default + Debug + Clone {
    type Result: BorderPerfect<Boolean = Self>;
    fn then_some(self, pid: PatternId) -> Self::Result;
    fn all_perfect(&self) -> bool;
}
impl BoolPerfect for bool {
    type Result = Option<PatternId>;
    fn then_some(self, pid: PatternId) -> Self::Result {
        self.then_some(pid)
    }
    fn all_perfect(&self) -> bool {
        *self
    }
}
impl BoolPerfect for (bool, bool) {
    type Result = (Option<PatternId>, Option<PatternId>);
    fn then_some(self, pid: PatternId) -> Self::Result {
        (
            self.0.then_some(pid),
            self.1.then_some(pid),
        )
    }
    fn all_perfect(&self) -> bool {
        self.0 && self.1
    }
}
pub trait BorderPerfect: Default + Debug + Clone {
    type Boolean: BoolPerfect<Result=Self>;
    fn new(boolean: Self::Boolean, pid: PatternId) -> Self;
    fn fold_or(&mut self, other: Self);
    fn complete(&self) -> Option<PatternId>;
}
impl BorderPerfect for Option<PatternId> {
    type Boolean = bool;
    fn new(boolean: Self::Boolean, pid: PatternId) -> Self {
        boolean.then_some(pid)
    }
    fn fold_or(&mut self, other: Self) {
        *self = self.or(other);
    }
    fn complete(&self) -> Option<PatternId> {
        *self
    }
}
impl BorderPerfect for (Option<PatternId>, Option<PatternId>) {
    type Boolean = (bool, bool);
    fn new((a, b): Self::Boolean, pid: PatternId) -> Self {
        (
            a.then_some(pid),
            b.then_some(pid),
        )
    }
    fn fold_or(&mut self, other: Self) {
        self.0.fold_or(other.0);
        self.1.fold_or(other.1);
    }
    fn complete(&self) -> Option<PatternId> {
        self.0.complete().and_then(|pid|
            self.1.complete().filter(|i| *i == pid)
        )
    }
}
