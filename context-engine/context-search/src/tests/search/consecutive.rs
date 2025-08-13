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
fn find_consecutive1() {
    let Env1 {
        graph,
        a,
        b,
        c,
        d,
        e,
        f,
        g,
        h,
        i,
        ab,
        bc,
        abc,
        abcd,
        ghi,
        ababababcdefghi,
        ..
    } = &*Env1::get_expected();
    let a_bc_pattern = [Child::new(a, 1), Child::new(bc, 2)];
    let ab_c_pattern = [Child::new(ab, 2), Child::new(c, 1)];
    let g_h_i_a_b_c_pattern = vec![
        Child::new(g, 1),
        Child::new(h, 1),
        Child::new(i, 1),
        Child::new(a, 1),
        Child::new(b, 1),
        Child::new(c, 1),
    ];

    let query = g_h_i_a_b_c_pattern;
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *ghi,
        "+g_h_i_a_b_c"
    );
    assert_matches!(
        graph.find_ancestor(&query),
        Ok(FinishedState {
            kind: FinishedKind::Complete(x),
            ..
        }) if x == *abc,
        "g_h_i_+a_b_c"
    );
}
