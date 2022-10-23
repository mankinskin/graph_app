
use std::collections::BTreeMap;

use crate::*;
use super::*;

#[derive(Clone, Debug)]
pub(crate) enum BandEnd {
    Index(Child),
    Chain(OverlapChain),
}
impl BandEnd {
    pub fn into_index<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(self, reader: &mut Reader<T, D>) -> Child {
        match self {
            Self::Index(c) => c,
            Self::Chain(c) => c.close(reader).expect("Empty chain in BandEnd!"),
        }
    }
    fn index(&self) -> Option<&Child> {
        match self {
            Self::Index(c) => Some(c),
            _ => None,
        }
    }
}
#[derive(Clone, Debug)]
pub(crate) struct OverlapBand {
    end: BandEnd,
    back_context: Pattern,
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
impl OverlapBand {
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
        self.back_context.push(self.end.clone().into_index(reader));
        self.end = end;
    }
    fn into_pattern<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(self, reader: &mut Reader<T, D>) -> Pattern {
        self.back_context.into_iter()
            .chain(std::iter::once(self.end.into_index(reader)))
            .collect()
    }
}
#[derive(Clone, Debug)]
pub(crate) struct OverlapLink {
    postfix_path: StartPath, // location of postfix/overlap in first index
    prefix_path: MatchEnd<StartPath>, // location of prefix/overlap in second index
    overlap: Child,
}
#[derive(Clone, Debug)]
pub(crate) struct Overlap {
    link: Option<OverlapLink>,
    band: OverlapBand,
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
#[derive(Default, Clone, Debug)]
pub(crate) struct OverlapBundle {
    bundle: Vec<OverlapBand>,
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
        Self {
            bundle,
        }
    }
}
impl OverlapBundle {
    fn add_band(&mut self, overlap: OverlapBand) {
        self.bundle.push(overlap)
    }
    pub fn append<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(&mut self, reader: &mut Reader<T, D>, end: BandEnd) {
        if self.bundle.len() > 1 {
            self.bundle.first_mut()
                .expect("Empty bundle in overlap chain!")
                .append(reader, end);
        } else {
            self.bundle = vec![self.clone().into_band(reader).appended(reader, end)];
        }
    }
    pub fn appended<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(mut self, reader: &mut Reader<T, D>, end: BandEnd) -> OverlapBundle {
        self.append(reader, end);
        self
    }
    pub fn into_band<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(self, reader: &mut Reader<T, D>) -> OverlapBand {
        assert!(!self.bundle.is_empty());

        let bundle = self.bundle.into_iter().map(|band| band.into_pattern(reader)).collect_vec();
        OverlapBand {
            end: BandEnd::Index(reader.graph_mut().index_patterns(bundle)),
            back_context: vec![],
        }
    }
}
#[derive(Default, Clone, Debug)]
pub(crate) struct OverlapChain {
    path: BTreeMap<usize, Overlap>,
}
impl From<Child> for OverlapBand {
    fn from(next: Child) -> Self {
        OverlapBand {
            end: BandEnd::Index(next),
            back_context: vec![],
        }
    }
}
impl OverlapChain {
    pub fn take_appended<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(&mut self, reader: &mut Reader<T, D>, start_bound: usize, end: BandEnd) -> Option<Overlap> {
        // postfixes should always be first in the chain
        self.path.remove(&start_bound).map(|band| {
            let index = end.into_index(reader);
            let end_bound = start_bound + index.width();
            // might want to pass postfix_path
            band.appended(reader, BandEnd::Index(index))
        })
    }
    pub fn append_index<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(&mut self, reader: &mut Reader<T, D>, start_bound: usize, end: BandEnd) {
        self.take_appended(reader, start_bound, end)
            .map(|overlap|
                self.add_overlap(
                    start_bound + overlap.band.end.index().unwrap().width(), // end_bound
                    overlap,
                )
            );
    }
    fn add_overlap(&mut self, end_bound: usize, overlap: Overlap) -> Result<(), Overlap> {
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
        println!("closing {:#?}", self);
        let mut path = self.path.into_iter();
        let first_band: Overlap = path.next()?.1;
        let (mut bundle, prev_band, _) =
            path.fold(
                (vec![], first_band, None),
                |(mut bundle, prev_band, prev_ctx), (_end_bound, overlap)| {
                    // index context of prefix
                    let ctx = overlap.link.as_ref().and_then(|node| 
                        IndexContext::<T, D, IndexFront>::try_context_path(
                            reader,
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
        let index = reader.graph_mut().index_patterns(bundle);
        println!("close result: {:?}", index);
        Some(index)
    }
    pub fn take_past(&mut self, end_bound: usize) -> OverlapChain {
        let mut past = self.path.split_off(&end_bound);
        std::mem::swap(&mut self.path, &mut past);
        Self { path: past }
    }
}
#[derive(Default, Clone, Debug)]
pub(crate) struct OverlapCache {
    chain: OverlapChain,
    end_bound: usize,
    last: Option<Overlap>,
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
    fn add_last_bundle<
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
impl<T: Tokenize, D: IndexDirection> Reader<T, D> {
    pub(crate) fn read_overlaps(
        &mut self,
        first: Child,
        context: &mut PrefixQuery,
    ) -> Option<Child> {
        (first.width() > 1)
            .then(||
                self.read_next_overlap(
                    OverlapCache::new(first),
                    context,
                )
            )?
    }
    /// next bands generated when next overlap starts after a past bundle with a gap
    pub(crate) fn odd_overlap_next_bands(
        &mut self,
        cache: &mut OverlapCache,
        past_end_bound: usize,
        next_link: &OverlapLink,
        expansion: Child,
        past_ctx: Pattern,
    ) -> Overlap {
        println!("odd overlap");
        let last = cache.last.as_mut().unwrap();
        let prev = last.band.end.clone().into_index(self);
        last.band.end = BandEnd::Index(prev);
        // split last band to get overlap with past bundle
        let IndexSplitResult {
            inner,
            location,
            path,
        } = IndexSplit::<_, D, IndexFront>::single_offset_split(
            &mut *self.graph.graph_mut(),
            prev,
            NonZeroUsize::new(cache.end_bound - past_end_bound).unwrap(),
        );
        assert!(path.is_empty());

        // build new context path (to overlap)
        let context_path = {
            // entry in last band (could be handled by IndexSplit
            let inner_entry = {
                let graph = self.graph.graph();
                let (pid, pattern) = graph.expect_vertex_data(inner).expect_any_pattern();
                ChildLocation {
                    parent: inner,
                    pattern_id: *pid,
                    sub_index: D::last_index(pattern.borrow()),
                }
            };
            let postfix_path = next_link.postfix_path.clone().into_context_path();
            Vec::with_capacity(postfix_path.len() + 2).tap_mut(|v| {
                v.push(location);
                v.push(inner_entry);
                v.extend(postfix_path.into_iter().skip(1));
            })
        };
        // get index between past and next overlap
        let (inner_back_ctx, _loc) = IndexContext::<T, D, IndexBack>::try_context_path(
            self,
            context_path,
            next_link.overlap,
        ).unwrap();

        let past = self.graph.graph_mut().index_pattern(past_ctx);
        let past_inner = self.graph.graph_mut().index_pattern([past, inner_back_ctx]);
        let inner_expansion = self.graph.graph_mut().index_pattern([inner_back_ctx, expansion]);
        let index = self.graph.graph_mut().index_patterns([
            [past_inner, expansion],
            [past, inner_expansion],
        ]);
        Overlap {
            band: OverlapBand {
                end: BandEnd::Index(index),
                back_context: vec![],
            },
            link: None, // todo
        }
    }
    pub(crate) fn read_next_overlap(
        &mut self,
        cache: OverlapCache,
        context: &mut PrefixQuery,
    ) -> Option<Child> {
        // find expandable postfix, may append postfixes in overlap chain
        println!("read next overlap with {:#?}", cache.last);
        match self.find_next_overlap(
                cache,
                context,
            ) {
            Ok((start_bound, next_link, expansion, mut cache)) => {
                    println!("found overlap at {}: {:?}", start_bound, expansion);
                    if let Some(ctx) =
                        if let Some((past_end_bound, past_ctx)) = self.take_past_context_pattern(
                            start_bound,
                            &mut cache.chain,
                        ) {
                            println!("reusing back context {past_end_bound}: {:#?}", past_ctx);
                            if past_end_bound == start_bound {
                                Some(past_ctx)
                            } else {
                                assert!(past_end_bound < start_bound);
                                let next = self.odd_overlap_next_bands(
                                    &mut cache,
                                    past_end_bound,
                                    &next_link,
                                    expansion,
                                    past_ctx,
                                );
                                cache.append(
                                    self,
                                    start_bound,
                                    next,
                                );

                                //OverlapBand {
                                //    end: BandEnd::Chain(expansion),
                                //    back_context: vec![inner_back_ctx],
                                //}
                                // use postfix_path of next to index inner context in previous
                                // index overlap between previous and new coming index
                                // create new bundle to start with

                                // - remember to bundle latest bands 
                                // - until after band where odd overlap ocurred
                                // - add extra band with postfix of previous band
                                // - if overlap after previous band 
                                None
                            }
                        } else {
                            println!("building back context from path");
                            Some(self.back_context_from_path(&mut cache.chain, &next_link))
                        }
                {
                    cache.append(
                        self,
                        start_bound,
                        Overlap {
                            band: OverlapBand {
                                end: BandEnd::Index(expansion),
                                back_context: ctx,
                            },
                            link: Some(next_link), // todo
                        }
                    );
                }

                    self.read_next_overlap(
                        cache,
                        context,
                    )
                },
            Err(cache) => {
                println!("No overlap found");
                cache.chain.close(self)
            }
        }
    }
    fn back_context_from_path(
        &mut self,
        overlaps: &mut OverlapChain,
        link: &OverlapLink,
    ) -> Pattern {
        let (inner_back_ctx, _loc) = IndexContext::<T, D, IndexBack>::try_context_path(
            self,
            link.postfix_path.clone().into_context_path(),
            link.overlap,
        ).unwrap();
        D::context_then_inner(
            overlaps.path.iter_mut().last()
                .and_then(|(_, last)|
                    self.graph.index_pattern(last.band.back_context.borrow())
                        .map(move |(back_ctx, _)| {
                            last.band.back_context = vec![back_ctx];
                            last.band.back_context.borrow()
                        })
                        .ok()
                ).unwrap_or_default(),
            inner_back_ctx,
        )
    }
    fn take_past_context_pattern(
        &mut self,
        start_bound: usize,
        overlaps: &mut OverlapChain,
    ) -> Option<(usize, Pattern)> {
        let mut past = overlaps.take_past(start_bound);
        match past.path.len() {
            0 => None,
            1 => {
                let (end_bound, past) = past.path.pop_last().unwrap();
                Some((end_bound, past.band.into_pattern(self)))
            },
            _ => Some((*past.path.keys().last().unwrap(), vec![past.close(self).unwrap()])),
        }
    }
    /// find largest expandable postfix
    fn find_next_overlap(
        &mut self,
        mut cache: OverlapCache,
        prefix_query: &mut PrefixQuery,
    ) -> Result<(usize, OverlapLink, Child, OverlapCache), OverlapCache> {
        let last = cache.last.take().expect("No last overlap to take!");
        match PostfixIterator::<_, D, _>::new(&self.indexer(), *last.band.end.index().unwrap())
            .try_fold((None as Option<StartPath>, OverlapBundle::from(last.band)), |(path, mut bundle), (postfix_location, postfix)| {
                let start_bound = cache.end_bound - postfix.width();

                let postfix_path = if let Some(path) = path {
                    path.append::<_, D, _>(&self.graph, postfix_location)
                } else {
                    StartPath::from(StartLeaf::new(postfix, postfix_location))
                };
                // try expand
                match self.graph.index_query_with_origin(OverlapPrimer::new(postfix, prefix_query.clone())) {
                    Ok((
                        OriginPath {
                            postfix: expansion,
                            origin: prefix_path,
                        },
                        advanced,
                    )) => {
                        *prefix_query = advanced.into_prefix_path();

                        ControlFlow::Break((
                            start_bound,
                            OverlapLink {
                                postfix_path,
                                prefix_path,
                                overlap: postfix,
                            },
                            expansion,
                            bundle,
                        ))
                    },
                    Err(_) => {
                        // if not expandable, at band boundary -> add to bundle
                        if let Some(overlap) = cache.chain.take_appended(self, start_bound, BandEnd::Index(postfix)) {
                            bundle.add_band(overlap.band)
                        }
                        ControlFlow::Continue((Some(postfix_path), bundle))
                    },
                }
            }) {
                ControlFlow::Break((start_bound, next, expansion, bundle)) => {
                    cache.add_last_bundle(self, bundle);
                    Ok((start_bound, next, expansion, cache))
                },
                ControlFlow::Continue((_, bundle)) => {
                    cache.add_last_bundle(self, bundle);
                    Err(cache)
                }
            }
    }
        
    //pub fn postfix_context_path(&self) -> Vec<StartPath> {
    //    self.path.values()
    //        .flat_map(|n| n.postfix_path().cloned())
    //        .collect_vec()
    //}
    //pub fn prefix_context_path(&self) -> Vec<StartPath> {
    //    self.path.values()
    //        .flat_map(|n| n.postfix_path().cloned())
    //        .collect_vec()
    //}
}