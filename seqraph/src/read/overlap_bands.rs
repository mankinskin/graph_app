use crate::*;
use std::collections::BTreeMap;
pub struct OverlapBands {
    bands: BTreeMap<usize, Vec<Pattern>>,
}
pub enum PastBand {
    Before(usize, Pattern),
    At(usize, Pattern),
}
impl Deref for OverlapBands {
    type Target = BTreeMap<usize, Vec<Pattern>>;
    fn deref(&self) -> &Self::Target {
        &self.bands
    }
}
impl DerefMut for OverlapBands {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bands
    }
}
impl OverlapBands {
    pub fn new(first: Child) -> Self {
        let mut bands = BTreeMap::default();
        bands.insert(first.width(), vec![vec![first]]);
        Self {
            bands,
        }
    }
    pub(crate) fn take_min_past_band<
        'a: 'g,
        'g,
        T: Tokenize + 'a,
        Trav: TraversableMut<'a, 'g, T> + 'a,
    >(&mut self, trav: &'a mut Trav, start_bound: usize) -> Option<PastBand> {
        let &key = self.bands.keys().find(|k| **k <= start_bound)?;
        let bundle = self.bands.remove(&key).unwrap();
        let band = match bundle.len() {
            0 => panic!("Empty bundle in bands!"),
            1 => bundle.into_iter().next().unwrap(),
            _ => {
                let mut graph = trav.graph_mut();
                let symbol = graph.index_patterns(bundle);
                vec![symbol]
            }
        };
        match key.cmp(&start_bound) {
            Ordering::Equal => Some(PastBand::At(key, band)),
            Ordering::Less => Some(PastBand::Before(key, band)),
            Ordering::Greater => None,
        }
    }
    //pub fn append_index<
    //    'a: 'g,
    //    'g,
    //    T: Tokenize + 'a,
    //    Trav: TraversableMut<'a, 'g, T> + 'a,
    //>(&mut self, trav: &'a mut Trav, end_bound: usize, index: Child) -> usize {
    //    let (end_bound, bundle) = self.take_min_past_band(trav, end_bound)
    //        .map(|(key, mut bundle)|
    //            (
    //                key + index.width(),
    //                vec![{
    //                    bundle.push(index);
    //                    bundle
    //                }]
    //            )
    //        )
    //        .unwrap_or_else(|| (index.width(), vec![vec![index]]));
    //    self.bands.insert(end_bound, bundle);
    //    end_bound
    //}
    pub fn add_band(&mut self, end_bound: usize, band: Pattern) {
        self.bands.insert(end_bound, vec![band]);
    }
}