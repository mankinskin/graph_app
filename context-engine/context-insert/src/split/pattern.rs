use context_trace::*;

use super::vertex::VertexSplits;
use std::fmt::Debug;

pub trait PatternSplits: Debug + Clone {
    type Pos;
    type Offsets;
    fn get(
        &self,
        pid: &PatternId,
    ) -> Option<Self::Pos>;
    fn ids<'a>(&'a self) -> Box<dyn Iterator<Item = &'a PatternId> + 'a>;
    fn offsets(&self) -> Self::Offsets;
}

impl PatternSplits for VertexSplits {
    type Pos = ChildTracePos;
    type Offsets = usize;
    fn get(
        &self,
        pid: &PatternId,
    ) -> Option<Self::Pos> {
        self.splits.get(pid).cloned()
    }
    fn ids<'a>(&'a self) -> Box<dyn Iterator<Item = &'a PatternId> + 'a> {
        Box::new(self.splits.keys())
    }
    fn offsets(&self) -> Self::Offsets {
        self.pos.get()
    }
}

impl PatternSplits for &VertexSplits {
    type Pos = ChildTracePos;
    type Offsets = usize;
    fn get(
        &self,
        pid: &PatternId,
    ) -> Option<Self::Pos> {
        self.splits.get(pid).cloned()
    }
    fn ids<'a>(&'a self) -> Box<dyn Iterator<Item = &'a PatternId> + 'a> {
        Box::new(self.splits.keys())
    }
    fn offsets(&self) -> Self::Offsets {
        self.pos.get()
    }
}

//impl<'a> PatternSplitsRef<'a> for &'a VertexSplits {
//    type Ref<'t> = Self where Self: 't;
//    fn as_ref<'t>(&'t self) -> Self::Ref<'t> where Self: 't {
//        *self
//    }
//}
impl<A: PatternSplits, B: PatternSplits> PatternSplits for (A, B) {
    type Pos = (A::Pos, B::Pos);
    type Offsets = (A::Offsets, B::Offsets);
    fn get(
        &self,
        pid: &PatternId,
    ) -> Option<Self::Pos> {
        self.0.get(pid).map(|a| {
            let b = self.1.get(pid).unwrap();
            (a, b)
        })
    }
    fn ids<'a>(&'a self) -> Box<dyn Iterator<Item = &'a PatternId> + 'a> {
        self.0.ids()
    }
    fn offsets(&self) -> Self::Offsets {
        (self.0.offsets(), self.1.offsets())
    }
}
//impl<
//    'a,
//    A: PatternSplitsRef<'a, Ref<'a> = PosSplitRef<'a>> + 'a,
//    B: PatternSplitsRef<'a, Ref<'a> = PosSplitRef<'a>> + 'a,
//> PatternSplitsRef<'a> for (A, B) {
//    type Ref<'t> = (PosSplitRef<'t>, PosSplitRef<'t>) where Self: 't;
//    fn as_ref<'t>(&'t self) -> Self::Ref<'t> where Self: 't {
//        (
//            self.0.as_ref(),
//            self.1.as_ref(),
//        )
//    }
//}
