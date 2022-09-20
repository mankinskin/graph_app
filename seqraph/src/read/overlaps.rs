
use std::collections::BTreeMap;

use crate::*;
use super::*;

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
pub(crate) enum Overlap {
    Node(OverlapNode),
    Band(OverlapBand),
}
impl Overlap {
    pub fn into_band(self) -> OverlapBand {
        *self.band()
    }
    pub fn band(&self) -> &OverlapBand {
        match self {
            Self::Node(node) => &node.band,
            Self::Band(band) => &band,
        }
    }
    pub fn postfix_location(&self) -> Option<&ChildLocation> {
        match self {
            Self::Node(node) => Some(&node.postfix_location),
            Self::Band(_) => None,
        }
    }
    pub fn prefix_location(&self) -> Option<&ChildLocation> {
        match self {
            Self::Node(node) => Some(&node.prefix_location),
            Self::Band(_) => None,
        }
    }
    pub fn locations(&self) -> Option<(&ChildLocation, &ChildLocation)> {
        match self {
            Self::Node(node) => Some((&node.postfix_location, &node.prefix_location)),
            Self::Band(_) => None,
        }
    }
}
pub(crate) struct OverlapNode {
    postfix_location: ChildLocation, // location of postfix/overlap in first index
    prefix_location: ChildLocation, // location of prefix/overlap in second index
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
    pub fn postfix_context_path(&self) -> Vec<ChildLocation> {
        self.path.values()
            .flat_map(|n| n.postfix_location().cloned())
            .collect_vec()
    }
    pub fn prefix_context_path(&self) -> Vec<ChildLocation> {
        self.path.values()
            .flat_map(|n| n.postfix_location().cloned())
            .collect_vec()
    }
    pub fn take_last(&mut self) -> Option<Overlap> {
        self.path.pop_last().map(|(_, n)| n)
    }
    pub fn take_past_overlaps(&mut self, end_bound: usize) -> OverlapChain {
        let mut past = self.path;
        self.path = past.split_off(&end_bound);
        Self { path: past }
    }
}

impl<T: Tokenize, D: IndexDirection> Reader<T, D> {
    pub(crate) fn read_overlaps(
        &mut self,
        first: Child,
        context: &mut PrefixPath,
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
        overlaps: OverlapChain,
        context: &mut PrefixPath,
    ) -> Option<Child> {
        if let Some(next) = overlaps.take_last()
            .and_then(|last|
                self.find_next_overlap(
                    last,
                    &mut overlaps,
                    context,
                )
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
            self.close(overlaps)
        }
    }
    /// find largest expandable postfix
    fn find_next_overlap(
        &mut self,
        last: Overlap,
        overlaps: &mut OverlapChain,
        prefix_path: &mut PrefixPath,
    ) -> Option<OverlapNode> {

        let band@&OverlapBand {
            index,
            back_context,
            end_bound,
        } = last.band();

        let mut bundle = vec![]; // collect all finished bands, ending at current end_bound
        let mut postfix_path = Vec::new(); // remember path to current postfix
        let mut graph = self.indexer().graph_mut();

        let next = PostfixIterator::<_, D, _>::new(&self.indexer(), index)
            .find_map(|(postfix_location, postfix)| {
                let start_bound = end_bound - postfix.width();

                // take all bands ending before or at current start_bound
                let past = overlaps.take_past_overlaps(start_bound);

                let new_context = match past.path.len() {
                    0 => None, // need to generate from postfix_path
                    1 => Some(past.path.pop_last().unwrap().1.into_band().into_iter().collect_vec()),
                    // Todo: index past bands
                    _ => {
                        past.prefix_context_path();
                        None
                    },
                };

                let root = overlaps.path.iter()
                    .next()
                    .map(|(_, n)| n)
                    .unwrap_or_else(|| &last);

                // expand
                // todo: get prefix_location
                match self.graph.index_query(OverlapPrimer::new(postfix, prefix_path.clone())) {
                    Ok((expansion, advanced)) => {
                        *prefix_path = advanced.into_prefix_path();

                        let new_context = new_context
                            .map(|band| {
                                // use past band
                                graph.index_pattern(band)
                            })
                            .unwrap_or_else(|| {
                                // get from parent
                                let (ictx, l) = IndexContext::<T, D, IndexBack>::context_path(
                                    self,
                                    postfix_location,
                                    postfix_path,
                                    postfix,
                                );
                                let bctx = graph.index_pattern(back_context.borrow());
                                let new_ctx = graph.index_pattern([bctx, ictx]);
                                if root.postfix_location().is_none() {
                                    let root_inner = 
                                        IndexSplit::<T, D, IndexBack>::single_offset_split(
                                            self,
                                            index,
                                            NonZeroUsize::new(root.band().end_bound - start_bound).unwrap(),
                                        );
                                    graph.add_pattern_with_update(
                                        postfix_location,
                                        <IndexBack as IndexSide<D>>::concat_inner_and_context(root_inner.inner, new_ctx)
                                    );
                                }
                                new_ctx
                            });
                        Some(OverlapNode {
                            postfix_location,
                            prefix_location,
                            band: OverlapBand {
                                end_bound: start_bound + expansion.width(),
                                index: expansion,
                                back_context: vec![new_context],
                            },
                        })
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
            });
        let root = overlaps.path.iter()
            .next()
            .map(|(_, n)| n)
            .unwrap_or_else(|| &last);
        let overlap = if bundle.is_empty() {
            if let Some((&postfix_location, &prefix_location)) = root.locations() {
                Overlap::Node(OverlapNode {
                    postfix_location,
                    prefix_location,
                    band: *band
                })
            } else {
                Overlap::Band(*band)
            }
        } else {
            bundle.push(band.into_iter().collect_vec());
            let index = graph.index_patterns(bundle);
            Overlap::Band(OverlapBand {
                index,
                end_bound,
                back_context: vec![],
            })
        };
        overlaps.path.insert(end_bound, overlap);
        next
    }
}