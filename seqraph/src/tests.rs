#[test]
fn fuzz1() {
    if let Err(_) = crate::mock::gen_graph::gen_graph() {
        panic!("Encountered panics when generating graph!");
    }
}