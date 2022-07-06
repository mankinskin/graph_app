#[test]
fn fuzz1() {
    if crate::mock::gen_graph::gen_graph().is_err() {
        panic!();
    }
}