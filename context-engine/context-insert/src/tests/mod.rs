use context_trace::*;

use crate::{
    build_trace_cache,
    insert_patterns,
};

use pretty_assertions::assert_eq;

pub mod insert;
pub mod interval;

#[macro_export]
macro_rules! insert_patterns {
    ($graph:ident,
        $(
            $name:ident => [
                $([$($pat:expr),*]),*$(,)?
            ]
        ),*$(,)?
    ) => {

        $(
            let $name = $graph.insert_patterns([$(vec![$($pat),*]),*]);
        )*
    };
    ($graph:ident,
        $(
            $name:ident =>
                [$($pat:expr),*]
        ),*$(,)?
    ) => {

        $(
            let $name = $graph.insert_pattern([$($pat),*]);
        )*
    };
    ($graph:ident,
        $(
            ($name:ident, $idname:ident) => [
                $([$($pat:expr),*]),*$(,)?
            ]
        ),*$(,)?
    ) => {

        $(
            let ($name, $idname) = context_trace::trace::has_graph::HasGraphMut::graph_mut(&mut $graph).insert_patterns_with_ids([$(vec![$($pat),*]),*]);
        )*
    };
    ($graph:ident,
        $(
            ($name:ident, $idname:ident) =>
                [$($pat:expr),*]
        ),*$(,)?
    ) => {

        $(
            let ($name, $idname) = context_trace::trace::has_graph::HasGraphMut::graph_mut(&mut $graph).insert_pattern_with_id([$($pat),*]);
            let $idname = $idname.unwrap();
        )*
    };
}

//#[macro_export]
//macro_rules! insert_patterns2 {
//    ($graph:ident,
//        $(
//            $name1:ident => [
//                $($pat1:ident),*
//                $([$($pat2:expr),*]),*
//                $(,)?
//            ]
//            $(
//                ($name2:ident, $idname:ident) => [
//                    $($pat3:ident),*
//                    $([$($pat4:ident),*]),*
//                    $(,)?
//                ]
//            )?
//        ),*
//        $(,)?
//    ) => {
//
//        $(
//            let $name1: Child = $graph.insert_pattern([$($pat1),*]);
//            let $name1: Child = $graph.insert_patterns(vec![$(vec![$($pat2),*]),*] as Vec<context_trace::graph::vertex::pattern::Pattern>);
//            $(
//                let ($name2, $idname): (Child, _) = $graph.graph_mut().insert_pattern_with_id([$($pat3),*]);
//                let $idname = $idname.unwrap();
//            )?
//            $(let ($name2, $idname): (Child, _) = $graph.graph_mut().insert_patterns_with_ids([$(vec![$($pat4),*]),*]))?
//        )*
//    };
//}
//

#[macro_export]
macro_rules! build_trace_cache {
    (
        $(
            $entry_root:ident => (BU {
                $(
                    $bu_pos:expr $(=> $($bu_child:ident -> ($bu_pid:expr, $bu_sub:expr)),*)?
                ),* $(,)?
            },
            TD {
                $(
                    $td_pos:expr $(=> $($td_child:ident -> ($td_pid:expr, $td_sub:expr)),*)?
                ),* $(,)?
            }
            $(,)?
        )
        ),*
            $(,)?
    ) => {
        context_trace::trace::cache::TraceCache {
            entries: HashMap::from_iter([
                $(
                    ($entry_root.vertex_index(), VertexCache {
                        index: $entry_root,
                        bottom_up: DirectedPositions::from_iter([
                            $(
                                (
                                    $bu_pos.into(),
                                    PositionCache {
                                        top: Default::default(),
                                        bottom: HashMap::from_iter([
                                            $($(
                                                (
                                                    DirectedKey {
                                                        index: $bu_child,
                                                        pos: DirectedPosition::BottomUp($bu_pos.into()),
                                                    },
                                                    SubLocation::new($bu_pid, $bu_sub),
                                                )
                                            ),*)?
                                        ]),
                                    },
                                ),
                            )*
                        ]),
                        top_down: DirectedPositions::from_iter([
                            $(
                                (
                                    $td_pos.into(),
                                    PositionCache {
                                        top: Default::default(),
                                        bottom: HashMap::from_iter([
                                            $($(
                                                (
                                                    DirectedKey {
                                                        index: $td_child,
                                                        pos: DirectedPosition::TopDown($td_pos.into()),
                                                    },
                                                    SubLocation::new($td_pid, $td_sub),
                                                ),
                                            ),*)?
                                        ]),
                                    },
                                ),
                            )*
                        ]),
                    }),
                )*
            ]),
        }
    };
}
#[macro_export]
macro_rules! nz {
    ($x:expr) => {
        std::num::NonZeroUsize::new($x).unwrap()
    };
}
#[macro_export]
macro_rules! build_split_cache {
    (
        $root_mode:expr,
        $(
            $entry_root:ident => {
                $(
                    {
                        $($top:ident: $top_pos:expr),*$(,)?
                    } -> $pos:expr => {
                        $($pid:expr => ($sub:expr, $inner:expr)),*$(,)?
                    }
                ),*$(,)?
            }
        ),*
        $(,)?
    ) => {
        SplitCache {
            root_mode: $root_mode,
            entries: HashMap::from_iter([
                $(
                    (
                        $entry_root.index,
                        SplitVertexCache {
                            positions: BTreeMap::from_iter([
                                $(
                                    (
                                        nz!($pos),
                                        SplitPositionCache {
                                            top: HashSet::from_iter([
                                                $(
                                                    PosKey {
                                                        index: $top.to_owned(),
                                                        pos: nz!($top_pos),
                                                    }
                                                ),*
                                            ]),
                                            pattern_splits: HashMap::from_iter([
                                                $(
                                                    (
                                                        $pid.to_owned(),
                                                        ChildTracePos {
                                                            inner_offset: $inner,
                                                            sub_index: $sub,
                                                        }
                                                    )
                                                ),*
                                            ])
                                        }
                                    )
                                ),*
                            ])
                        }
                    )
                ),*
            ])
        }
    };
}

#[test]
fn test_build_trace_cache1() {
    let mut graph = HypergraphRef::default();
    insert_tokens!(graph, {h, e, l, d});
    insert_patterns!(graph,
        (ld, ld_id) => [l, d],
        (heldld, heldld_id) => [h, e, ld, ld]
    );
    let cache = build_trace_cache!(
        heldld => (
            BU {},
            TD {2 => ld -> (heldld_id, 2) },
        ),
        ld => (
            BU {},
            TD { 2 => l -> (ld_id, 0) },
        ),
        h => (
            BU {},
            TD {},
        ),
        l => (
            BU {},
            TD { 2 },
        ),
    );
    assert_eq!(cache, TraceCache {
        entries: HashMap::from_iter([
            (heldld.vertex_index(), VertexCache {
                index: heldld,
                bottom_up: DirectedPositions::from_iter([]),
                top_down: DirectedPositions::from_iter([(
                    2.into(),
                    PositionCache {
                        top: Default::default(),
                        bottom: HashMap::from_iter([(
                            DirectedKey {
                                index: ld,
                                pos: DirectedPosition::TopDown(2.into(),),
                            },
                            SubLocation::new(heldld_id, 2),
                        )]),
                    },
                )]),
            }),
            (ld.vertex_index(), VertexCache {
                index: ld,
                bottom_up: DirectedPositions::from_iter([]),
                top_down: DirectedPositions::from_iter([(
                    2.into(),
                    PositionCache {
                        top: Default::default(),
                        bottom: HashMap::from_iter([(
                            DirectedKey {
                                index: l,
                                pos: DirectedPosition::TopDown(2.into(),),
                            },
                            SubLocation::new(ld_id, 0),
                        )]),
                    },
                )]),
            }),
            (h.vertex_index(), VertexCache {
                index: h,
                bottom_up: DirectedPositions::from_iter([]),
                top_down: DirectedPositions::from_iter([]),
            }),
            (l.vertex_index(), VertexCache {
                index: l,
                bottom_up: DirectedPositions::from_iter([]),
                top_down: DirectedPositions::from_iter([(
                    2.into(),
                    PositionCache {
                        top: Default::default(),
                        bottom: Default::default(),
                    },
                )]),
            }),
        ]),
    });
}

#[test]
fn test_build_trace_cache2() {
    let mut graph = HypergraphRef::default();
    insert_tokens!(graph, {a, b, c, d});

    insert_patterns!(graph,
        (ab, ab_id) => [a, b],
        (ababcd, ababcd_id) => [ab, ab, c, d]
    );
    let cache = build_trace_cache!(
        ababcd => (
            BU { 1 => ab -> (ababcd_id, 1) },
            TD {},
        ),
        ab => (
            BU { 1 => b -> (ab_id, 1) },
            TD {},
        ),
        b => (
            BU {},
            TD {},
        ),
    );
    assert_eq!(cache, TraceCache {
        entries: HashMap::from_iter([
            (ababcd.vertex_index(), VertexCache {
                index: ababcd,
                bottom_up: DirectedPositions::from_iter([(
                    1.into(),
                    PositionCache {
                        top: Default::default(),
                        bottom: HashMap::from_iter([(
                            DirectedKey {
                                index: ab,
                                pos: DirectedPosition::BottomUp(1.into(),),
                            },
                            SubLocation::new(ababcd_id, 1),
                        )]),
                    },
                )]),
                top_down: DirectedPositions::from_iter([]),
            }),
            (ab.vertex_index(), VertexCache {
                index: ab,
                bottom_up: DirectedPositions::from_iter([(
                    1.into(),
                    PositionCache {
                        top: Default::default(),
                        bottom: HashMap::from_iter([(
                            DirectedKey {
                                index: b,
                                pos: DirectedPosition::BottomUp(1.into(),),
                            },
                            SubLocation::new(ab_id, 1),
                        )]),
                    },
                )]),
                top_down: DirectedPositions::from_iter([]),
            }),
            (b.vertex_index(), VertexCache {
                index: b,
                bottom_up: DirectedPositions::from_iter([]),
                top_down: DirectedPositions::from_iter([]),
            }),
        ]),
    });
}
