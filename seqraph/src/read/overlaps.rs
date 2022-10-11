
use std::collections::BTreeMap;

use crate::*;
use super::*;

#[derive(Clone)]
pub(crate) struct OverlapBand {
    //end_bound: usize,
    index: Child,
    back_context: Pattern,
}
impl IntoIterator for OverlapBand {
    type Item = Child;
    type IntoIter = std::iter::Chain<std::vec::IntoIter<Child>, std::iter::Once<Child>>;
    fn into_iter(self) -> Self::IntoIter {
        self.back_context.into_iter().chain(std::iter::once(self.index))
    }
}
impl<'a> IntoIterator for &'a OverlapBand {
    type Item = &'a Child;
    type IntoIter = std::iter::Chain<std::slice::Iter<'a, Child>, std::iter::Once<&'a Child>>;
    fn into_iter(self) -> Self::IntoIter {
        self.back_context.iter().chain(std::iter::once(&self.index))
    }
}
impl OverlapBand {
    pub fn appended(mut self, postfix: Child) -> Self {
        self.append(postfix);
        self
    }
    pub fn append(&mut self, postfix: Child) {
        self.back_context.push(self.index);
        self.index = postfix;
    }
}
#[derive(Clone)]
pub(crate) struct OverlapNode {
    postfix_path: StartPath, // location of postfix/overlap in first index
    prefix_path: MatchEnd<StartPath>, // location of prefix/overlap in second index
    overlap: Child,
}
#[derive(Clone)]
pub(crate) struct Overlap {
    node: Option<OverlapNode>,
    band: OverlapBand,
}
impl Overlap {
    pub fn appended_index(mut self, index: Child) -> Self {
        self.append_index(index);
        self
    }
    pub fn append_index(&mut self, index: Child) {
        self.band.append(index);
        self.node = None;
    }
}
#[derive(Clone, Default)]
pub(crate) struct OverlapBundle {
    bundle: Vec<Overlap>,
}
impl From<Overlap> for OverlapBundle {
    fn from(overlap: Overlap) -> Self {
        Self {
            bundle: vec![overlap],
        }
    }
}
impl From<Vec<Overlap>> for OverlapBundle {
    fn from(bundle: Vec<Overlap>) -> Self {
        Self {
            bundle,
        }
    }
}
impl OverlapBundle {
    fn add_overlap(&mut self, overlap: Overlap) {
        self.bundle.push(overlap)
    }
    pub fn append_index<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(&mut self, reader: &mut Reader<T, D>, index: Child) {
        if self.bundle.len() > 1 {
            self.bundle.first_mut()
                .expect("Empty bundle in overlap chain!")
                .append_index(index);
        } else {
            self.bundle = vec![self.clone().into_overlap(reader).appended_index(index)];
        }
    }
    pub fn into_overlap<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(self, reader: &mut Reader<T, D>) -> Overlap {
        assert!(!self.bundle.is_empty());
        Overlap {
            node: None,
            band: OverlapBand {
                index: reader.graph_mut().index_patterns(
                    self.bundle.into_iter().map(|overlap| overlap.band.into_iter().collect_vec())
                ),
                back_context: vec![],
            },
        }
    }
    pub fn appended_index<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(mut self, reader: &mut Reader<T, D>, index: Child) -> OverlapBundle {
        self.append_index(reader, index);
        self
    }
}
#[derive(Default)]
pub(crate) struct OverlapChain {
    path: BTreeMap<usize, Overlap>,
}
impl From<Child> for OverlapBand {
    fn from(next: Child) -> Self {
        OverlapBand {
            index: next,
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
    >(&mut self, reader: &mut Reader<T, D>, start_bound: usize, index: Child) -> Option<Overlap> {
        // postfixes should always be first in the chain
        self.path.remove(&start_bound).map(|band| {
            let end_bound = start_bound + index.width();
            // might want to pass postfix_path
            band.appended_index(index)
        })
    }
    pub fn append_index<
        'a: 'g,
        'g,
        T: Tokenize,
        D: IndexDirection,
    >(&mut self, reader: &mut Reader<T, D>, start_bound: usize, index: Child) {
        self.take_appended(reader, start_bound, index)
            .map(|overlap| 
                self.add_overlap(
                    start_bound + index.width(), // end_bound
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
    >(mut self, reader: &'a mut Reader<T, D>) -> Option<Child> {
        let mut path = self.path.into_iter();
        let first_band: Overlap = path.next()?.1;
        let (mut bundle, prev_band, prev_ctx) =
            path.fold(
                (vec![], first_band, None),
                |(mut bundle, prev_band, prev_ctx), (end_bound, overlap)| {
                    let ctx = overlap.node.as_ref().and_then(|node| 
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
                        prev_ctx.map(|prev|
                            ctx.map(|ctx|
                                reader.read_pattern(vec![prev, ctx])
                            ).unwrap_or(prev)
                        ).or(ctx)
                    )
                }
            );

        bundle.push(prev_band);
        Some(reader.graph_mut().index_patterns(bundle.into_iter().map(|overlap| overlap.band.into_iter().collect_vec())))
    }
    pub fn take_past(&mut self, end_bound: usize) -> OverlapChain {
        let mut past = self.path.split_off(&end_bound);
        std::mem::swap(&mut self.path, &mut past);
        Self { path: past }
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
                    first.width(),
                    Overlap {
                        node: None,
                        band: OverlapBand::from(first),
                    }.into(),
                    OverlapChain::default(),
                    context,
                )
            )?
    }
    pub(crate) fn read_next_overlap(
        &mut self,
        end_bound: usize,
        last: OverlapBundle,
        mut overlaps: OverlapChain,
        context: &mut PrefixQuery,
    ) -> Option<Child> {
        // find expandable postfix, may append postfixes in overlap chain
        match self.find_next_overlap(
                end_bound,
                last,
                &mut overlaps,
                context,
            ) {
            Ok((start_bound, next, expansion, bundle)) => {
                    overlaps.path.insert(
                        end_bound,
                        bundle.into_overlap(self),
                    );
                    let ctx = self.take_past_context_pattern(
                        start_bound,
                        &next,
                        &mut overlaps,
                    )
                    .map(|(end_bound, ctx)|
                        if end_bound == start_bound {
                            ctx
                        } else {
                            assert!(end_bound < start_bound);
                            // use postfix_path of next to index inner context in previous
                            // index overlap between previous and new coming index
                            // create new bundle to start with
                            ctx
                        }
                    )
                    .unwrap_or_else(||
                        self.back_context_from_path(&mut overlaps, &next)
                    );
                    let band = OverlapBand {
                        index: expansion,
                        back_context: ctx,
                    };
                    let next_bound = start_bound + expansion.width();
                    self.read_next_overlap(
                        next_bound,
                        Overlap {
                            node: Some(next),
                            band
                        }.into(),
                        overlaps,
                        context,
                    )
                },
            Err(bundle) => {
                overlaps.path.insert(
                    end_bound,
                    bundle.into_overlap(self),
                );
                overlaps.close(self)
            }
        }
    }
    fn back_context_from_path(
        &mut self,
        overlaps: &mut OverlapChain,
        node: &OverlapNode,
    ) -> Pattern {
        let (inner_back_ctx, _loc) = IndexContext::<T, D, IndexBack>::try_context_path(
            self,
            node.postfix_path.clone().into_context_path(),
            node.overlap,
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
        node: &OverlapNode,
        overlaps: &mut OverlapChain,
    ) -> Option<(usize, Pattern)> {
        let mut past = overlaps.take_past(start_bound);
        match past.path.len() {
            0 => None,
            1 => {
                let (end_bound, past) = past.path.pop_last().unwrap();
                Some((end_bound, past.band.into_iter().collect()))
            },
            _ => Some((*past.path.keys().last().unwrap(), vec![past.close(self).unwrap()])),
        }
    }
    /// find largest expandable postfix
    fn find_next_overlap(
        &mut self,
        end_bound: usize,
        last: OverlapBundle,
        overlaps: &mut OverlapChain,
        prefix_query: &mut PrefixQuery,
    ) -> Result<(usize, OverlapNode, Child, OverlapBundle), OverlapBundle> {
        // determine first overlap
        //let root = overlaps.path.iter()
        //    .next()
        //    .map(|(_, n)| n)
        //    .unwrap_or_else(|| &last)
        //    .clone();
        //let band = last.into_band();
        
        //let mut postfix_path = None; // remember path to current postfix

        match PostfixIterator::<_, D, _>::new(&self.indexer(), last)
            .try_fold((None as Option<StartPath>, last), |(path, mut bundle), (postfix_location, postfix)| {
                let start_bound = end_bound - postfix.width();

                //// all bands ending before or at current start_bound
                //let (next_overlaps, new_context) = self.get_past_context(start_bound, overlaps);

                let postfix_path = if let Some(path) = path {
                    path.append::<_, D, _>(&self.graph, postfix_location)
                } else {
                    StartPath::from(StartLeaf::new(postfix, postfix_location))
                };
                // expand
                // todo: get prefix_location
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
                            OverlapNode {
                                postfix_path,
                                prefix_path,
                                overlap: postfix,
                            },
                            expansion,
                            bundle,
                        ))
                    },
                    Err(_) => {
                        // if not expandable, at band boundary and no bundle exists, create bundle
                        if let Some(overlap) = overlaps.take_appended(self, start_bound, postfix) {
                            bundle.add_overlap(overlap)
                        }
                        ControlFlow::Continue((Some(postfix_path), bundle))
                    },
                }
            }) {
                ControlFlow::Break(b) => Ok(b),
                ControlFlow::Continue((_, bundle)) => Err(bundle.into()),
            }
    }
        //{
        //        ControlFlow::Break((
        //            postfix_path,
        //            prefix_path,
        //            start_bound,
        //            expansion,
        //            postfix,
        //            new_context,
        //        )) => {
        //            let postfix_path = postfix_path.unwrap();
        //            let new_context = if let Some((end_bound, band)) = new_context {
        //                // use past band
        //                self.graph_mut().index_pattern(band)
        //            } else {
        //                // get from parent
        //                let (ictx, l) = IndexContext::<T, D, IndexBack>::context_path(
        //                    &mut self.graph,
        //                    postfix_path.get_entry_location(),
        //                    postfix_path.into_path(),
        //                    postfix,
        //                );
        //                let bctx = self.graph_mut().index_pattern(band.back_context.borrow());
        //                let new_ctx = self.graph_mut().index_pattern([bctx, ictx]);
        //                if root.postfix_path().is_none() {
        //                    let root_inner = 
        //                        IndexSplit::<T, D, IndexBack>::single_offset_split(
        //                            &mut self.graph,
        //                            band.index,
        //                            NonZeroUsize::new(root.band().end_bound - start_bound).unwrap(),
        //                        );
        //                    self.graph_mut().add_pattern_with_update(
        //                        postfix_path.get_entry_location(),
        //                        <IndexBack as IndexSide<D>>::concat_inner_and_context(root_inner.inner, new_ctx)
        //                    );
        //                }
        //                new_ctx
        //            };
        //            Some(OverlapNode::Node {
        //                overlap: postfix,
        //                postfix_path,
        //                prefix_path,
        //                band: OverlapBand {
        //                    //end_bound: start_bound + expansion.width(),
        //                    index: expansion,
        //                    back_context: vec![new_context],
        //                },
        //            })
        //        },
        //        _ => None,
        //};
        //let overlap = if bundle.is_empty() {
        //    if let Some((postfix_path, prefix_path)) = root.paths() {
        //        OverlapNode::Node {
        //            overlap: postfix,
        //            postfix_path,
        //            prefix_path,
        //            band
        //        }
        //    } else {
        //        OverlapNode::Band(band)
        //    }
        //} else {
        //    bundle.push(band.into_iter().collect_vec());
        //    let index = self.graph_mut().index_patterns(bundle);
        //    OverlapNode::Band(OverlapBand {
        //        index,
        //        back_context: vec![],
        //        //end_bound,
        //    })
        //};
        //overlaps.path.insert(end_bound, overlap);
        //next.map(|n| (end_bound, n))
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