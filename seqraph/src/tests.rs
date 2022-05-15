#[test]
fn fuzz1() {
    if crate::gen_graph::gen_graph().is_err() {
        panic!();
    }
}