use std::{
    borrow::Borrow,
    num::NonZeroUsize,
    sync::RwLockWriteGuard,
};

use derive_more::{
    Deref,
    DerefMut,
};

use builder::*;
use ctx::*;
use position::*;

use crate::{
    index::side::{
        IndexBack,
        IndexSide,
    },
    join::partition::splits::offset::OffsetSplits,
    split::{
        cache::vertex::SplitVertexCache,
        PatternSplitPos,
    },
    traversal::{
        cache::{
            entry::position::SubSplitLocation,
            key::SplitKey,
            labelled_key::vkey::VertexCacheKey,
        },
        folder::state::{
            FoldState,
            RootMode,
        },
        traversable::TraversableMut,
    },
    vertex::{
        child::Child,
        indexed::Indexed,
        location::SubLocation,
        pattern::Pattern,
        PatternId,
    },
    HashMap,
};

pub mod vertex;

pub mod builder;
pub mod ctx;
pub mod leaves;
pub mod position;
pub mod split;

#[derive(Debug, Clone)]
pub struct TraceState {
    pub index: Child,
    pub offset: NonZeroUsize,
    pub prev: SplitKey,
}

#[derive(Debug, Deref, DerefMut)]
pub struct SplitCache {
    pub entries: HashMap<VertexCacheKey, SplitVertexCache>,
    #[deref]
    #[deref_mut]
    pub context: CacheContext,
    pub root_mode: RootMode,
}

impl SplitCache {
    pub fn new<
        'a,
        Trav: TraversableMut<GuardMut<'a> = RwLockWriteGuard<'a, crate::graph::Hypergraph>> + 'a,
    >(
        trav: &'a mut Trav,
        fold_state: FoldState,
    ) -> Self {
        SplitCacheBuilder::new(trav, fold_state).build()
    }
    pub fn get(
        &self,
        key: &SplitKey,
    ) -> Option<&SplitPositionCache> {
        self.entries
            .get(&key.index.vertex_index())
            .and_then(|ve| ve.positions.get(&key.pos))
    }
    pub fn get_mut(
        &mut self,
        key: &SplitKey,
    ) -> Option<&mut SplitPositionCache> {
        self.entries
            .get_mut(&key.index.vertex_index())
            .and_then(|ve| ve.positions.get_mut(&key.pos))
    }
    pub fn expect(
        &self,
        key: &SplitKey,
    ) -> &SplitPositionCache {
        self.get(key).unwrap()
    }
    pub fn expect_mut(
        &mut self,
        key: &SplitKey,
    ) -> &mut SplitPositionCache {
        self.get_mut(key).unwrap()
    }
}

pub fn range_splits<'a>(
    patterns: impl Iterator<Item = (&'a PatternId, &'a Pattern)>,
    parent_range: (NonZeroUsize, NonZeroUsize),
) -> (OffsetSplits, OffsetSplits) {
    let (ls, rs) = patterns
        .map(|(pid, pat)| {
            let (li, lo) = IndexBack::token_offset_split(pat.borrow(), parent_range.0).unwrap();
            let (ri, ro) = IndexBack::token_offset_split(pat.borrow(), parent_range.1).unwrap();
            (
                (
                    *pid,
                    PatternSplitPos {
                        sub_index: li,
                        inner_offset: lo,
                    },
                ),
                (
                    *pid,
                    PatternSplitPos {
                        sub_index: ri,
                        inner_offset: ro,
                    },
                ),
            )
        })
        .unzip();
    (
        OffsetSplits {
            offset: parent_range.0,
            splits: ls,
        },
        OffsetSplits {
            offset: parent_range.1,
            splits: rs,
        },
    )
}

pub fn cleaned_position_splits<'a>(
    patterns: impl Iterator<Item = (&'a PatternId, &'a Pattern)>,
    parent_offset: NonZeroUsize,
) -> Result<Vec<SubSplitLocation>, SubLocation> {
    patterns
        .map(|(pid, pat)| {
            let (sub_index, inner_offset) =
                IndexBack::token_offset_split(pat.borrow(), parent_offset).unwrap();
            let location = SubLocation::new(*pid, sub_index);
            if inner_offset.is_some() || pat.len() > 2 {
                // can't be clean
                Ok(SubSplitLocation {
                    location,
                    inner_offset,
                })
            } else {
                // must be clean
                Err(location)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        collections::BTreeMap,
        iter::FromIterator,
    };

    use pretty_assertions::assert_eq;

    use crate::{
        graph::tests::{
            context_mut,
            Context,
        },
        split::{
            cache::{
                position::SplitPositionCache,
                vertex::SplitVertexCache,
            },
            PatternSplitPos,
            SplitCache,
        },
        traversal::cache::{
            key::SplitKey,
            labelled_key::vkey::lab,
            trace,
        },
        HashMap,
        HashSet,
    };

    macro_rules! nz {
        ($x:expr) => {
            NonZeroUsize::new($x).unwrap()
        };
    }
    #[test]
    fn split_graph1() {
        let res = trace::tests::build_trace1();
        let Context {
            graph,
            def,
            d_ef_id,
            c_def_id,
            cdef,
            abcdef,
            abcd_ef_id,
            ab_cdef_id,
            abc_def_id,
            ef,
            e_f_id,
            ..
        } = &mut *context_mut();
        let splits = SplitCache::new(&mut *graph, res);
        assert_eq!(
            splits.entries[&lab!(ef)],
            SplitVertexCache {
                positions: BTreeMap::from_iter([(
                    nz!(1),
                    SplitPositionCache {
                        top: HashSet::from_iter([
                            SplitKey {
                                index: *abcdef,
                                pos: nz!(5),
                            },
                            SplitKey {
                                index: *def,
                                pos: nz!(2),
                            },
                        ]),
                        pattern_splits: HashMap::from_iter([(
                            *e_f_id,
                            PatternSplitPos {
                                inner_offset: None,
                                sub_index: 1,
                            }
                        )])
                    }
                )]),
            },
        );
        assert_eq!(
            splits.entries[&lab!(def)],
            SplitVertexCache {
                positions: BTreeMap::from_iter([(
                    nz!(2),
                    SplitPositionCache {
                        top: HashSet::from_iter([
                            SplitKey {
                                index: *abcdef,
                                pos: nz!(5),
                            },
                            SplitKey {
                                index: *cdef,
                                pos: nz!(3),
                            },
                        ]),
                        pattern_splits: HashMap::from_iter([(
                            *d_ef_id,
                            PatternSplitPos {
                                inner_offset: Some(nz!(1)),
                                sub_index: 1,
                            }
                        )])
                    }
                )]),
            },
        );
        assert_eq!(
            splits.entries[&lab!(cdef)],
            SplitVertexCache {
                positions: BTreeMap::from_iter([(
                    nz!(3),
                    SplitPositionCache {
                        top: HashSet::from_iter([SplitKey {
                            index: *abcdef,
                            pos: nz!(5),
                        },]),
                        pattern_splits: HashMap::from_iter([(
                            *c_def_id,
                            PatternSplitPos {
                                inner_offset: Some(nz!(2)),
                                sub_index: 1,
                            }
                        )])
                    }
                )]),
            },
        );
        assert_eq!(
            splits.entries[&lab!(abcdef)],
            SplitVertexCache {
                positions: BTreeMap::from_iter([(
                    nz!(5),
                    SplitPositionCache {
                        top: HashSet::from_iter([]),
                        pattern_splits: HashMap::from_iter([
                            (
                                *abcd_ef_id,
                                PatternSplitPos {
                                    inner_offset: Some(nz!(1)),
                                    sub_index: 1,
                                }
                            ),
                            (
                                *abc_def_id,
                                PatternSplitPos {
                                    inner_offset: Some(nz!(2)),
                                    sub_index: 1,
                                }
                            ),
                            (
                                *ab_cdef_id,
                                PatternSplitPos {
                                    inner_offset: Some(nz!(3)),
                                    sub_index: 1,
                                }
                            ),
                        ])
                    }
                )]),
            },
        );
        assert_eq!(splits.entries.len(), 4);
    }
}
