use std::iter::FromIterator;

use crate::search::Searchable;
use itertools::*;
use pretty_assertions::{
    assert_eq,
    assert_matches,
};

#[cfg(test)]
use context_trace::tests::env::Env1;

use crate::{
    fold::result::{
        FinishedKind,
        FinishedState,
    },
    traversal::state::{
        cursor::{
            PatternCursor,
            PatternRangeCursor,
        },
        end::{
            postfix::PostfixEnd,
            range::RangeEnd,
            EndKind,
            EndReason,
            EndState,
        },
    },
};
use context_trace::{
    graph::{
        getters::ErrorReason,
        kind::BaseGraphKind,
        vertex::{
            child::Child,
            location::{
                child::ChildLocation,
                pattern::PatternLocation,
                SubLocation,
            },
            token::Token,
        },
        Hypergraph,
        HypergraphRef,
    },
    lab,
    path::structs::{
        role_path::RolePath,
        rooted::{
            role_path::RootedRolePath,
            root::IndexRoot,
            RootedRangePath,
        },
        sub_path::SubPath,
    },
    tests::env::TestEnv,
    trace::{
        cache::{
            key::directed::{
                down::DownKey,
                DirectedKey,
            },
            position::{
                Edges,
                PositionCache,
            },
            vertex::VertexCache,
            TraceCache,
        },
        has_graph::HasGraph,
    },
    HashMap,
    HashSet,
};
use tracing::{
    info,
    Level,
};

#[test]
fn find_parent1() {
    let Env1 {
        graph,
        a,
        b,
        c,
        d,
        ab,
        bc,
        abc,
        ..
    } = &*Env1::get_expected();
    let a_bc_pattern = vec![Child::new(a, 1), Child::new(bc, 2)];
    let ab_c_pattern = vec![Child::new(ab, 2), Child::new(c, 1)];
    let a_bc_d_pattern =
        vec![Child::new(a, 1), Child::new(bc, 2), Child::new(d, 1)];
    let b_c_pattern = vec![Child::new(b, 1), Child::new(c, 1)];
    let bc_pattern = vec![Child::new(bc, 2)];
    let a_b_c_pattern =
        vec![Child::new(a, 1), Child::new(b, 1), Child::new(c, 1)];

    let query = bc_pattern;
    assert_eq!(
        graph.find_parent(&query),
        Err(ErrorReason::SingleIndex(*bc)),
        "bc"
    );
    let query = b_c_pattern;
    assert_matches!(
        graph.find_parent(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *bc,
        "b_c"
    );
    let query = ab_c_pattern;
    assert_matches!(
        graph.find_parent(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *abc,
        "ab_c"
    );
    // enable when bfs for parent-child batches is implemented
    //let query = a_bc_pattern;
    //assert_matches!(
    //    graph.find_parent(&query),
    //    Ok(FinishedState {
    //        kind: FinishedKind::Complete(x),
    //        ..
    //    }) if x == *abc,
    //    "a_bc"
    //);
    //let query = a_bc_d_pattern;
    //assert_matches!(
    //    graph.find_parent(&query),
    //    Ok(FinishedState {
    //        kind: FinishedKind::Complete(x),
    //        ..
    //    }) if x == *abc,
    //    "a_bc_d"
    //);
    //let query = a_b_c_pattern.clone();
    //assert_matches!(
    //    graph.find_parent(&query),
    //    Ok(FinishedState {
    //        kind: FinishedKind::Complete(x),
    //        ..
    //    }) if x == *abc,
    //    "a_b_c"
    //);
    //let query = [&a_b_c_pattern[..], &[Child::new(c, 1)]].concat();
    //assert_matches!(
    //    graph.find_parent(&query),
    //    Ok(FinishedState {
    //        kind: FinishedKind::Complete(x),
    //        ..
    //    }) if x == *abc,
    //    "a_b_c_c"
    //);
}
