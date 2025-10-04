use std::borrow::Borrow;

use context_trace::*;
use derivative::Derivative;
use derive_more::Deref;

pub struct BandCtx<'a> {
    pub band: &'a Band,
    //pub back_link: Option<&'a OverlapLink>,
    //pub front_link: Option<&'a OverlapLink>,
}
impl From<BandCtx<'_>> for Band {
    fn from(band: BandCtx<'_>) -> Self {
        band.band.clone()
    }
}

#[derive(Clone, Debug, Derivative)]
#[derivative(Ord, Eq, PartialEq, PartialOrd)]
pub struct Band {
    pub pattern: Pattern,
    pub start_bound: usize,
    pub end_bound: usize, // key for ordering
}
impl Borrow<usize> for Band {
    fn borrow(&self) -> &usize {
        &self.end_bound
    }
}
impl Band {
    pub fn postfix(&self) -> Child {
        *self.pattern.last().unwrap()
    }
    pub fn append(
        &mut self,
        postfix: Child,
    ) {
        let width = self.postfix().width();
        self.start_bound += width;
        self.end_bound += width;
        self.pattern.push(postfix);
    }
}
//impl From<(usize, Band)> for Band {
//    fn from((_, band): (usize, Band)) -> Self {
//        band
//    }
//}
impl From<Child> for Band {
    fn from(first: Child) -> Self {
        Self {
            start_bound: 0,
            end_bound: first.width(),
            pattern: vec![first],
        }
    }
}
impl From<(usize, Pattern)> for Band {
    fn from((start_bound, pattern): (usize, Pattern)) -> Self {
        let end_bound = start_bound + pattern_width(&pattern);
        Self {
            pattern,
            start_bound,
            end_bound,
        }
    }
}

#[derive(Clone, Debug, Eq, Derivative, Deref)]
#[derivative(Ord, PartialOrd, PartialEq)]
pub struct Overlap {
    #[deref]
    pub index: Child,
    pub start_bound: usize, // key for ordering
}
impl Overlap {
    pub fn end_bound(&self) -> usize {
        self.start_bound + self.width()
    }
}
