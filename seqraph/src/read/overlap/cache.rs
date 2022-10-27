use crate::*;
use super::*;

#[derive(Default, Clone, Debug)]
pub(crate) struct OverlapCache {
    pub(crate) chain: OverlapChain,
    pub(crate) end_bound: usize,
    pub(crate) last: Option<Overlap>,
}
impl OverlapCache {
    pub fn new(first: Child) -> Self {
        Self {
            end_bound: first.width(),
            last: Overlap {
                link: None,
                band: OverlapBand::from(first),
            }.into(),
            chain: OverlapChain::default(),
        }
    }
    pub fn add_bundle<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(
        &mut self,
        reader: &mut Reader<T, D>,
        bundle: OverlapBundle
    ) {
        self.chain.path.insert(
            self.end_bound,
            Overlap {
                link: None,
                band: bundle.into_band(reader),
            }
        );
    }
    pub fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(&mut self,
        _reader: &mut Reader<T, D>,
        start_bound: usize,
        overlap: Overlap,
    ) {
        let width = overlap.band.end.index().unwrap().width();
        if let Some(last) = self.last.replace(overlap) {
            self.chain.add_overlap(self.end_bound, last).unwrap()
        }
        self.end_bound = start_bound + width;
    }
}