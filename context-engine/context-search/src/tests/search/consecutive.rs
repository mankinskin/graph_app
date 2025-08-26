#[cfg(test)]
use {
    crate::fold::result::{
        FinishedKind,
        FinishedState,
    },
    crate::search::Searchable,
    context_trace::tests::env::Env1,
    context_trace::{
        graph::vertex::child::Child,

        tests::env::TestEnv,
    },
    pretty_assertions::assert_matches,
};

#[test]
fn find_consecutive1() {
    let Env1 {
        graph,
        a,
        b,
        c,
        g,
        h,
        i,
        abc,
        ghi,
        ..
    } = &*Env1::get_expected();
    //let a_bc_pattern = [Child::new(a, 1), Child::new(bc, 2)];
    //let ab_c_pattern = [Child::new(ab, 2), Child::new(c, 1)];
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
