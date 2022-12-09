use super::*;

use std::collections::BTreeMap;

#[derive(Default, Clone, Debug)]
pub struct OverlapChain {
    pub path: BTreeMap<usize, Overlap>,
}
impl OverlapChain {
    pub fn add_overlap(&mut self, end_bound: usize, overlap: Overlap) -> Result<(), Overlap> {
        // postfixes should always start at first end bounds in the chain
        match self.path.insert(end_bound, overlap) {
            Some(other) => Err(other),
            None => Ok(()),
        }
    }
    #[instrument(skip(self, reader))]
    pub fn close<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(self, reader: &'a mut Reader<T, D>) -> Option<Child> {
        //println!("closing {:#?}", self);
        let mut path = self.path.into_iter();
        let first_band: Overlap = path.next()?.1;
        let (mut bundle, prev_band, _) = {
            let reader = &reader.clone();
            path.fold(
                (vec![], first_band, None),
                |(mut bundle, prev_band, prev_ctx), (_end_bound, overlap)| {
                    let mut reader = reader.clone();
                    // index context of prefix
                    let ctx = if let Some(node) = overlap.link.as_ref() {
                        reader.contexter::<IndexFront>().try_context_path(
                            node.prefix_path.get_path().unwrap().clone().into_context_path(),
                            //node.overlap,
                        )
                        
                        .map(|(ctx, _)| ctx)
                    } else {
                        None
                    };
                    bundle.push(prev_band);
                    (
                        bundle,
                        overlap,
                        // join previous and current context into 
                        if let Some(prev) = prev_ctx {
                            Some(if let Some(ctx) = ctx {
                                reader.read_pattern(vec![prev, ctx]).unwrap()
                            } else {
                                prev
                            })
                        } else {
                            ctx
                        }
                    )
                }
            )
        };
        bundle.push(prev_band);
        let bundle = bundle.into_iter()
            .map(|overlap| overlap.band.into_pattern(reader))
            .collect_vec();
        let index = reader.graph_mut().insert_patterns(bundle);
        //println!("close result: {:?}", index);
        Some(index)
    }
    #[instrument(skip(self, end_bound))]
    pub fn take_past(&mut self, end_bound: usize) -> OverlapChain {
        let mut past = self.path.split_off(&end_bound);
        std::mem::swap(&mut self.path, &mut past);
        Self { path: past }
    }
}
#[derive(Clone, Debug)]
pub struct OverlapLink {
    pub postfix_path: StartPath, // location of postfix/overlap in first index
    pub prefix_path: MatchEnd<StartPath>, // location of prefix/overlap in second index
}
#[derive(Clone, Debug)]
pub struct Overlap {
    pub link: Option<OverlapLink>,
    pub band: OverlapBand,
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