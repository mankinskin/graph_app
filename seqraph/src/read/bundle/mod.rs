use band::OverlapBand;
use itertools::Itertools;

use hypercontext_api::traversal::traversable::TraversableMut;

pub mod band;
//pub struct BandsContext {
//    pub graph: HypergraphRef,
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
    pub fn wrap_into_band(
        self,
        mut trav: impl TraversableMut,
    ) -> OverlapBand {
        assert!(!self.bundle.is_empty());

        let bundle = self
            .bundle
            .into_iter()
            .map(|band| band.into_pattern())
            .collect_vec();
        OverlapBand {
            end: trav.graph_mut().insert_patterns(bundle),
            back_context: vec![],
        }
    }
    //pub fn append<
    //    'a: 'g,
    //    'g,
    //    T: Tokenize,
    //    D: IndexDirection,
    //>(&mut self, reader: &mut ReadContext<T, D>, end: BandEnd) {
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
    //>(mut self, reader: &mut ReadContext<T, D>, end: BandEnd) -> OverlapBundle {
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
