use crate::*;
use super::*;

use std::collections::BTreeMap;

#[derive(Default, Clone, Debug)]
pub(crate) struct OverlapChain {
    pub(crate) path: BTreeMap<usize, Overlap>,
}
impl OverlapChain {
    pub fn add_overlap(&mut self, end_bound: usize, overlap: Overlap) -> Result<(), Overlap> {
        // postfixes should always start at first end bounds in the chain
        match self.path.insert(end_bound, overlap) {
            Some(other) => Err(other),
            None => Ok(()),
        }
    }
    pub fn close<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(self, reader: &'a mut Reader<T, D>) -> Option<Child> {
        //println!("closing {:#?}", self);
        let mut path = self.path.into_iter();
        let first_band: Overlap = path.next()?.1;
        let (mut bundle, prev_band, _) =
            path.fold(
                (vec![], first_band, None),
                |(mut bundle, prev_band, prev_ctx), (_end_bound, overlap)| {
                    // index context of prefix
                    let ctx = overlap.link.as_ref().and_then(|node| 
                        reader.contexter::<IndexFront>().try_context_path(
                            node.prefix_path.get_path().unwrap().clone().into_context_path(),
                            node.overlap,
                        )
                    ).map(|(ctx, _)| ctx);
                    bundle.push(prev_band);
                    (
                        bundle,
                        overlap,
                        // join previous and current context into 
                        prev_ctx.map(|prev|
                            ctx.map(|ctx|
                                reader.read_pattern(vec![prev, ctx])
                            ).unwrap_or(prev)
                        ).or(ctx)
                    )
                }
            );

        bundle.push(prev_band);
        let bundle = bundle.into_iter()
            .map(|overlap| overlap.band.into_pattern(reader))
            .collect_vec();
        let index = reader.graph_mut().insert_patterns(bundle);
        //println!("close result: {:?}", index);
        Some(index)
    }
    pub fn take_past(&mut self, end_bound: usize) -> OverlapChain {
        let mut past = self.path.split_off(&end_bound);
        std::mem::swap(&mut self.path, &mut past);
        Self { path: past }
    }
    //pub fn append_index<
    //    'a: 'g,
    //    'g,
    //    T: Tokenize,
    //    D: IndexDirection,
    //>(&mut self, reader: &mut Reader<T, D>, start_bound: usize, end: BandEnd) {
    //    self.take_appended(reader, start_bound, end)
    //        .map(|overlap|
    //            self.add_overlap(
    //                start_bound + overlap.band.end.index().unwrap().width(), // end_bound
    //                overlap,
    //            )
    //        );
    //}
}
#[derive(Clone, Debug)]
pub(crate) struct OverlapLink {
    pub(crate) postfix_path: StartPath, // location of postfix/overlap in first index
    pub(crate) prefix_path: MatchEnd<StartPath>, // location of prefix/overlap in second index
    pub(crate) overlap: Child,
}
#[derive(Clone, Debug)]
pub(crate) struct Overlap {
    pub(crate) link: Option<OverlapLink>,
    pub(crate) band: OverlapBand,
}
impl Overlap {
    pub fn appended<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(mut self, reader: &mut Reader<T, D>, end: BandEnd) -> Self {
        self.append(reader, end);
        self
    }
    pub fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(&mut self, reader: &mut Reader<T, D>, end: BandEnd) {
        self.band.append(reader, end);
        self.link = None;
    }
}