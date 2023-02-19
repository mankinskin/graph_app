use super::*;
pub struct Context {
    pub graph: HypergraphRef,
    pub a: Child,
    pub b: Child,
    pub c: Child,
    pub d: Child,
    pub e: Child,
    pub f: Child,
    pub g: Child,
    pub h: Child,
    pub i: Child,
    pub ab: Child,
    pub bc: Child,
    pub bc_id: PatternId,
    pub cd: Child,
    pub cd_id: PatternId,
    pub bcd: Child,
    pub b_cd_id: PatternId,
    pub abc: Child,
    pub a_bc_id: PatternId,
    pub abcd: Child,
    pub a_bcd_id: PatternId,
    pub abc_d_id: PatternId,
    pub ef: Child,
    pub cdef: Child,
    pub efghi: Child,
    pub abab: Child,
    pub ababab: Child,
    pub ababababcdefghi: Child,
}
lazy_static::lazy_static! {
    pub static ref
        CONTEXT: Arc<RwLock<Context>> = Arc::new(RwLock::new({
        let mut graph = Hypergraph::default();
        if let [a, b, c, d, e, f, g, h, i] = graph.insert_tokens(
            [
                Token::Element('a'),
                Token::Element('b'),
                Token::Element('c'),
                Token::Element('d'),
                Token::Element('e'),
                Token::Element('f'),
                Token::Element('g'),
                Token::Element('h'),
                Token::Element('i'),
            ])[..] {
            // abcdefghi
            // ababababcdbcdefdefcdefefghefghghi
            // ->
            // abab ab abcdbcdefdefcdefefghefghghi
            // ab abab abcdbcdefdefcdefefghefghghi

            // abcdbcdef def cdef efgh efgh ghi

            // abcd b cdef
            // abcd bcd ef

            // ab cd
            // abc d
            // a bcd
            // index: 9
            let ab = graph.insert_pattern([a, b]);
            let (bc, bc_id) = graph.insert_pattern_with_id([b, c]);
            let (abc, abc_ids) = graph.insert_patterns_with_ids([
                [ab, c],
                [a, bc],
            ]);
            let (cd, cd_id) = graph.insert_pattern_with_id([c, d]);
            let (bcd, bcd_ids) = graph.insert_patterns_with_ids([
                [bc, d],
                [b, cd],
            ]);
            //let abcd = graph.insert_pattern(&[abc, d]);
            //graph.insert_to_pattern(abcd, &[a, bcd]);
            let (abcd, abcd_ids) = graph.insert_patterns_with_ids([
                [abc, d],
                [a, bcd],
            ]);
            // index 15
            let ef = graph.insert_pattern([e, f]);
            let gh = graph.insert_pattern([g, h]);
            let ghi = graph.insert_pattern([gh, i]);
            let efgh = graph.insert_pattern([ef, gh]);
            let efghi = graph.insert_patterns([
                [efgh, i],
                [ef, ghi],
            ]);
            let def = graph.insert_pattern([d, ef]);
            let cdef = graph.insert_pattern([c, def]);
            // index 22
            let abcdef = graph.insert_patterns([
                [abc, def],
                [ab, cdef]
            ]);
            let abcdefghi = graph.insert_patterns([
                [abcd, efghi],
                [abcdef, ghi]
            ]);
            let aba = graph.insert_pattern([ab, a]);
            // 5
            let abab = graph.insert_patterns([
                [aba, b],
                [ab, ab],
            ]);
            let ababab = graph.insert_patterns([
                [abab, ab],
                [ab, abab],
            ]);
            let ababcd = graph.insert_patterns([
                [ab, abcd],
                [aba, bcd],
                [abab, cd],
            ]);
            // 28
            let ababababcd = graph.insert_patterns([
                [ababab, abcd],
                [abab, ababcd],
            ]);
            let ababcdefghi = graph.insert_patterns([
                [ab, abcdefghi],
                [ababcd, efghi],
            ]);
            // 30
            let ababababcdefghi = graph.insert_patterns([
                [ababababcd, efghi],
                [abab, ababcdefghi],
                [ababab, abcdefghi],
            ]);
            Context {
                graph: HypergraphRef::from(graph),
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
                bc_id: bc_id.unwrap(),
                cd,
                cd_id: cd_id.unwrap(),
                bcd,
                b_cd_id: bcd_ids[1],
                abc,
                a_bc_id: abc_ids[1],
                abcd,
                abc_d_id: abcd_ids[0],
                a_bcd_id: abcd_ids[1],
                ef,
                cdef,
                efghi,
                abab,
                ababab,
                ababababcdefghi,
            }
        } else {
            panic!();
        }
    }));
}
pub fn context() -> RwLockReadGuard<'static, Context> {
    CONTEXT.read().unwrap()
}
pub fn context_mut() -> RwLockWriteGuard<'static, Context> {
    CONTEXT.write().unwrap()
}

#[test]
fn test_to_petgraph() {
    let mut graph = Hypergraph::<BaseGraphKind>::default();
    let (a, b, c, d) = graph.insert_tokens([
        Token::Element('a'),
        Token::Element('b'),
        Token::Element('c'),
        Token::Element('d'),
    ]).into_iter().next_tuple().unwrap();
    // ab cd
    // abc d
    // a bcd

    let ab = graph.insert_pattern([a, b]);
    let bc = graph.insert_pattern([b, c]);
    let abc = graph.insert_patterns([[ab, c], [a, bc]]);
    let cd = graph.insert_pattern([c, d]);
    let bcd = graph.insert_patterns([[bc, d], [b, cd]]);
    let _abcd = graph.insert_patterns([[abc, d], [a, bcd]]);
    let pg = graph.to_petgraph();
    pg.write_to_file("assets/test_graph1.dot")
        .expect("Failed to write assets/test_graph1.dot file!");
}