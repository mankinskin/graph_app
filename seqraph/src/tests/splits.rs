use std::{
    collections::BTreeMap,
    iter::FromIterator,
};

use pretty_assertions::assert_eq;

use crate::tests::trace::build_trace1;
use hypercontext_api::{
    lab, partition::splits::PosSplits, split::{
        cache::{
            position::SplitPositionCache,
            vertex::SplitVertexCache,
            SplitCache,
        },
        PatternSplitPos,
    }, tests::graph::{
        context_mut,
        Context,
    }, traversal::cache::key::SplitKey, HashMap, HashSet
};

macro_rules! nz {
    ($x:expr) => {
        std::num::NonZeroUsize::new($x).unwrap()
    };
}
#[test]
fn split_graph1() {
    let mut res = build_trace1();
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
    let splits = SplitCache::new(&mut *graph, &mut res);
    assert_eq!(
        splits.entries[&lab!(ef)],
        SplitVertexCache {
            positions: PosSplits { splits: BTreeMap::from_iter([(
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
            )]) },
        },
    );
    assert_eq!(
        splits.entries[&lab!(def)],
        SplitVertexCache {
            positions: PosSplits { splits: BTreeMap::from_iter([(
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
            )]) },
        },
    );
    assert_eq!(
        splits.entries[&lab!(cdef)],
        SplitVertexCache {
            positions: PosSplits { splits: BTreeMap::from_iter([(
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
            )]) },
        },
    );
    assert_eq!(
        splits.entries[&lab!(abcdef)],
        SplitVertexCache {
            positions: PosSplits { splits: BTreeMap::from_iter([(
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
            )]) },
        },
    );
    assert_eq!(splits.entries.len(), 4);
}