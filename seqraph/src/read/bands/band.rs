use itertools::Itertools;

use hypercontext_api::graph::vertex::{
    child::Child,
    pattern::Pattern,
};
use crate::read::reader::context::ReadContext;

#[derive(Clone, Debug)]
pub enum BandEnd {
    Index(Child),
    //Chain(OverlapChain),
}

impl<'p> BandEnd {
    pub fn into_index(
        self,
        _reader: &mut ReadContext<'p>,
    ) -> Child {
        match self {
            Self::Index(c) => c,
            //Self::Chain(c) => c.close(reader).expect("Empty chain in BandEnd!"),
        }
    }
    pub fn index(&self) -> Option<&Child> {
        match self {
            Self::Index(c) => Some(c),
            //_ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct OverlapBand {
    pub end: BandEnd,
    pub back_context: Pattern,
}

impl<'p> OverlapBand {
    pub fn append(
        &mut self,
        reader: &mut ReadContext<'p>,
        end: BandEnd,
    ) {
        self.back_context.push(self.end.clone().into_index(reader));
        self.end = end;
    }
    pub fn into_pattern(
        self,
        reader: &mut ReadContext<'p>,
    ) -> Pattern {
        self.back_context
            .into_iter()
            .chain(std::iter::once(self.end.into_index(reader)))
            .collect()
    }
    //pub fn appended<
    //    'a: 'g,
    //    'g,
    //    T: Tokenize,
    //    D: IndexDirection,
    //>(mut self, reader: &mut ReadContext<'p><T, D>, end: BandEnd) -> Self {
    //    self.append(reader, end);
    //    self
    //}
}

impl From<Child> for OverlapBand {
    fn from(next: Child) -> Self {
        OverlapBand {
            end: BandEnd::Index(next),
            back_context: vec![],
        }
    }
}
//impl IntoIterator for OverlapBand {
//    type Item = Child;
//    type IntoIter = std::iter::Chain<std::vec::IntoIter<Child>, std::iter::Once<Child>>;
//    fn into_iter(self) -> Self::IntoIter {
//        self.back_context.into_iter().chain(std::iter::once(self.index))
//    }
//}
//impl<'a> IntoIterator for &'a OverlapBand {
//    type Item = &'a Child;
//    type IntoIter = std::iter::Chain<std::slice::Iter<'a, Child>, std::iter::Once<&'a Child>>;
//    fn into_iter(self) -> Self::IntoIter {
//        self.back_context.iter().chain(std::iter::once(&self.index))
//    }
//}

#[derive(Default, Clone, Debug)]
pub struct OverlapBundle {
    bundle: Vec<OverlapBand>,
}

impl<'p> OverlapBundle {
    pub fn add_band(
        &mut self,
        overlap: OverlapBand,
    ) {
        self.bundle.push(overlap)
    }
    pub fn into_band(
        self,
        reader: &mut ReadContext<'p>,
    ) -> OverlapBand {
        assert!(!self.bundle.is_empty());

        let bundle = self
            .bundle
            .into_iter()
            .map(|band| band.into_pattern(reader))
            .collect_vec();
        OverlapBand {
            end: BandEnd::Index(reader.graph.insert_patterns(bundle)),
            back_context: vec![],
        }
    }
    //pub fn append<
    //    'a: 'g,
    //    'g,
    //    T: Tokenize,
    //    D: IndexDirection,
    //>(&mut self, reader: &mut ReadContext<'p><T, D>, end: BandEnd) {
    //    if self.bundle.len() > 1 {
    //        self.bundle.first_mut()
    //            .expect("Empty bundle in overlap chain!")
    //            .append(reader, end);
    //    } else {
    //        self.bundle = vec![self.clone().into_band(reader).appended(reader, end)];
    //    }
    //}
    //pub fn appended<
    //    'a: 'g,
    //    'g,
    //    T: Tokenize,
    //    D: IndexDirection,
    //>(mut self, reader: &mut ReadContext<'p><T, D>, end: BandEnd) -> OverlapBundle {
    //    self.append(reader, end);
    //    self
    //}
}

impl From<OverlapBand> for OverlapBundle {
    fn from(overlap: OverlapBand) -> Self {
        Self {
            bundle: vec![overlap],
        }
    }
}

impl From<Vec<OverlapBand>> for OverlapBundle {
    fn from(bundle: Vec<OverlapBand>) -> Self {
        Self { bundle }
    }
}
