use crate::*;

pub trait BoolPerfect: Default + Debug + Clone {
    type Result: BorderPerfect<Boolean = Self>;
    fn then_some(self, pid: PatternId) -> Self::Result;
    fn all_perfect(&self) -> bool;
}
impl BoolPerfect for bool {
    type Result = SinglePerfect;
    fn then_some(self, pid: PatternId) -> Self::Result {
        self.then_some(pid).into()
    }
    fn all_perfect(&self) -> bool {
        *self
    }
}
impl BoolPerfect for (bool, bool) {
    type Result = DoublePerfect;
    fn then_some(self, pid: PatternId) -> Self::Result {
        (
            self.0.then_some(pid),
            self.1.then_some(pid),
        ).into()
    }
    fn all_perfect(&self) -> bool {
        self.0 && self.1
    }
}
pub trait BorderPerfect: Default + Debug + Clone + Extend<Self> {
    type Boolean: BoolPerfect<Result=Self>;
    fn new(boolean: Self::Boolean, pid: PatternId) -> Self;
    //fn fold_or(&mut self, other: Self);
    fn complete(&self) -> SinglePerfect;
}
#[derive(Debug, Default, Clone, Copy, From, Into)]
pub struct SinglePerfect(pub Option<PatternId>);

#[derive(Debug, Default, Clone, Copy, From, Into)]
pub struct DoublePerfect(pub Option<PatternId>, pub Option<PatternId>);

impl std::ops::Add for SinglePerfect {
    type Output = Self;
    fn add(self, x: Self) -> Self::Output {
        self.0.or(x.0).into()
    }
}
impl std::ops::Add for DoublePerfect {
    type Output = Self;
    fn add(self, x: Self) -> Self::Output {
        (
            self.0.or(x.0),
            self.1.or(x.1),
        ).into()
    }
}
impl std::ops::AddAssign for SinglePerfect {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;        
    }
}
impl std::ops::AddAssign for DoublePerfect {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;        
    }
}


impl FromIterator<Self> for SinglePerfect {
    fn from_iter<T: IntoIterator<Item = Self>>(iter: T) -> Self {
        iter.into_iter().fold(Self::default(), |acc, x|
            acc + x
        )
    }
}
impl FromIterator<Self> for DoublePerfect {
    fn from_iter<T: IntoIterator<Item = Self>>(iter: T) -> Self {
        iter.into_iter().fold(Self::default(), |acc, x|
            acc + x
        )
    }
}
impl Extend<Self> for SinglePerfect {
    fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
        *self += Self::from_iter(iter);        
    }
}
impl Extend<Self> for DoublePerfect {
    fn extend<T: IntoIterator<Item = Self>>(&mut self, iter: T) {
        *self += Self::from_iter(iter);        
    }
}
impl BorderPerfect for SinglePerfect {
    type Boolean = bool;
    fn new(boolean: Self::Boolean, pid: PatternId) -> Self {
        (boolean.then_some(pid)).into()
    }
    //fn fold_or(&mut self, other: Self) {
    //    *self = self.or(other);
    //}
    fn complete(&self) -> SinglePerfect {
        *self
    }
}
impl BorderPerfect for DoublePerfect {
    type Boolean = (bool, bool);
    fn new((a, b): Self::Boolean, pid: PatternId) -> Self {
        (
            a.then_some(pid),
            b.then_some(pid),
        ).into()
    }
    //fn fold_or(&mut self, other: Self) {
    //    self.0.fold_or(other.0);
    //    self.1.fold_or(other.1);
    //}
    fn complete(&self) -> SinglePerfect {
        SinglePerfect(self.0).complete().0.and_then(|pid|
            SinglePerfect(self.1).complete().0.filter(|i| *i == pid)
        ).into()
    }
}
