
use std::collections::BTreeMap;

use crate::*;
use super::*;

#[derive(Clone)]
pub(crate) struct OverlapBand {
    index: Child,
    end_bound: usize,
    back_context: Pattern,
}
impl IntoIterator for OverlapBand {
    type Item = Child;
    type IntoIter = std::iter::Chain<std::vec::IntoIter<Child>, std::iter::Once<Child>>;
    fn into_iter(self) -> Self::IntoIter {
        self.back_context.into_iter().chain(std::iter::once(self.index))
    }
}
#[derive(Clone)]
pub(crate) enum Overlap {
    Node(OverlapNode),
    Band(OverlapBand),
}
impl Overlap {
    pub fn into_band(self) -> OverlapBand {
        match self {
            Self::Node(node) => node.band,
            Self::Band(band) => band,
        }
    }
    pub fn band(&self) -> &OverlapBand {
        match self {
            Self::Node(node) => &node.band,
            Self::Band(band) => &band,
        }
    }
    pub fn postfix_path(&self) -> Option<&ChildPath> {
        match self {
            Self::Node(node) => Some(&node.postfix_path),
            Self::Band(_) => None,
        }
    }
    pub fn prefix_location(&self) -> Option<&ChildPath> {
        match self {
            Self::Node(node) => Some(&node.prefix_path),
            Self::Band(_) => None,
        }
    }
    pub fn paths(self) -> Option<(ChildPath, ChildPath)> {
        match self {
            Self::Node(node) => Some((node.postfix_path, node.prefix_path)),
            Self::Band(_) => None,
        }
    }
}
#[derive(Clone)]
pub(crate) struct OverlapNode {
    postfix_path: ChildPath, // location of postfix/overlap in first index
    prefix_path: ChildPath, // location of prefix/overlap in second index
    band: OverlapBand,
}
pub(crate) struct OverlapChain {
    path: BTreeMap<usize, Overlap>,
}
impl From<Child> for OverlapChain {
    fn from(next: Child) -> Self {
        Self::from(OverlapBand {
            end_bound: next.width(),
            index: next,
            back_context: vec![],
        })
    }
}
impl From<OverlapBand> for OverlapChain {
    fn from(next: OverlapBand) -> Self {
        Self {
            path: BTreeMap::from([
                (next.end_bound, Overlap::Band(next))
            ]),
        }
    }
}
impl OverlapChain {
    pub fn postfix_context_path(&self) -> Vec<ChildPath> {
        self.path.values()
            .flat_map(|n| n.postfix_path().cloned())
            .collect_vec()
    }
    pub fn prefix_context_path(&self) -> Vec<ChildPath> {
        self.path.values()
            .flat_map(|n| n.postfix_path().cloned())
            .collect_vec()
    }
    pub fn take_last(&mut self) -> Option<Overlap> {
        self.path.pop_last().map(|(_, n)| n)
    }
    pub fn take_past_overlaps(&mut self, end_bound: usize) -> OverlapChain {
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
                    OverlapChain::from(first),
                    context,
                )
            )?
    }
    pub(crate) fn read_next_overlap(
        &mut self,
        mut overlaps: OverlapChain,
        context: &mut PrefixQuery,
    ) -> Option<Child> {
        if let Some(last) = overlaps.take_last() {
            if let Some(next) =
                self.find_next_overlap(
                    last.clone(),
                    &mut overlaps,
                    context,
                )
            {
                overlaps.path.insert(
                    next.band.end_bound,
                    Overlap::Node(next),
                );
                self.read_next_overlap(
                    overlaps,
                    context,
                )
            } else {
                let mut graph = self.graph_mut();
                Some(graph.index_pattern(last.into_band().into_iter().collect_vec()))
            }
        } else {
            None
        }
    }
    /// find largest expandable postfix
    fn find_next_overlap(
        &mut self,
        last: Overlap,
        overlaps: &mut OverlapChain,
        prefix_query: &mut PrefixQuery,
    ) -> Option<OverlapNode> {


        let root = overlaps.path.iter()
            .next()
            .map(|(_, n)| n)
            .unwrap_or_else(|| &last)
            .clone();

        let band = last.into_band();
        let mut bundle = vec![]; // collect all finished bands, ending at current end_bound
        let mut postfix_path = Vec::new(); // remember path to current postfix

        let next = match PostfixIterator::<_, D, _>::new(&self.indexer(), band.index)
            .find_map(|(postfix_location, postfix)| {
                let start_bound = band.end_bound - postfix.width();

                // take all bands ending before or at current start_bound
                let mut past = overlaps.take_past_overlaps(start_bound);

                let new_context = match past.path.len() {
                    0 => None, // need to generate from postfix_path
                    1 => Some(past.path.pop_last().unwrap().1.into_band().into_iter().collect_vec()),
                    // Todo: index past bands
                    _ => {
                        past.prefix_context_path();
                        None
                    },
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

                        Some((
                            prefix_path.into_context_path(),
                            start_bound,
                            expansion,
                            postfix,
                            postfix_location,
                            new_context
                        ))
                    },
                    Err(_) => {
                        postfix_path.push(postfix_location);
                        // if not expandable, at band boundary and no bundle exists, create bundle
                        if let Some(band) = new_context {
                            bundle.push(
                                band.tap_mut(|b| b.push(postfix)),
                            );
                        }
                        None
                    },
                }
            }) {
            Some((prefix_path, start_bound, expansion, postfix, postfix_location, new_context)) => {
                let new_context = if let Some(band) = new_context {
                    // use past band
                    self.graph_mut().index_pattern(band)
                } else {
                    // get from parent
                    let (ictx, l) = IndexContext::<T, D, IndexBack>::context_path(
                        &mut self.graph,
                        postfix_location,
                        postfix_path.clone(),
                        postfix,
                    );
                    let bctx = self.graph_mut().index_pattern(band.back_context.borrow());
                    let new_ctx = self.graph_mut().index_pattern([bctx, ictx]);
                    if root.postfix_path().is_none() {
                        let root_inner = 
                            IndexSplit::<T, D, IndexBack>::single_offset_split(
                                &mut self.graph,
                                band.index,
                                NonZeroUsize::new(root.band().end_bound - start_bound).unwrap(),
                            );
                        self.graph_mut().add_pattern_with_update(
                            postfix_location,
                            <IndexBack as IndexSide<D>>::concat_inner_and_context(root_inner.inner, new_ctx)
                        );
                    }
                    new_ctx
                };
                Some(OverlapNode {
                    postfix_path,
                    prefix_path,
                    band: OverlapBand {
                        end_bound: start_bound + expansion.width(),
                        index: expansion,
                        back_context: vec![new_context],
                    },
                })
            },
            None => None,
        };
        let end_bound = band.end_bound;
        let overlap = if bundle.is_empty() {
            if let Some((postfix_path, prefix_path)) = root.paths() {
                Overlap::Node(OverlapNode {
                    postfix_path,
                    prefix_path,
                    band
                })
            } else {
                Overlap::Band(band)
            }
        } else {
            bundle.push(band.into_iter().collect_vec());
            let index = self.graph_mut().index_patterns(bundle);
            Overlap::Band(OverlapBand {
                index,
                back_context: vec![],
                end_bound,
            })
        };
        overlaps.path.insert(end_bound, overlap);
        next
    }
}