use hypercontext_api::graph::vertex::{
    child::Child,
    pattern::Pattern,
};

//#[derive(Clone, Debug)]
//pub enum BandEnd {
//    Index(Child),
//    //Chain(OverlapChain),
//}

//impl<'p> BandEnd {
//    pub fn into_index(self) -> Child {
//        match self {
//            Self::Index(c) => c,
//            //Self::Chain(c) => c.close(reader).expect("Empty chain in BandEnd!"),
//        }
//    }
//    pub fn index(&self) -> Option<&Child> {
//        match self {
//            Self::Index(c) => Some(c),
//            //_ => None,
//        }
//    }
//}

#[derive(Clone, Debug)]
pub struct OverlapBand {
    pub end: Child,
    pub back_context: Pattern,
}

impl<'p> OverlapBand {
    pub fn append(
        &mut self,
        end: Child,
    ) {
        self.back_context.push(self.end.clone());
        self.end = end;
    }
    pub fn into_pattern(self) -> Pattern {
        self.back_context
            .into_iter()
            .chain(std::iter::once(self.end))
            .collect()
    }
    //pub fn appended<
    //    'a: 'g,
    //    'g,
    //    T: Tokenize,
    //    D: IndexDirection,
    //>(mut self, reader: &mut ReadContext<T, D>, end: BandEnd) -> Self {
    //    self.append(reader, end);
    //    self
    //}
}

impl From<Child> for OverlapBand {
    fn from(next: Child) -> Self {
        OverlapBand {
            end: next,
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
