use seqraph::*;

pub fn build_graph1() -> Hypergraph<char> {
    let mut graph = Hypergraph::default();
    if let [a, b, w, x, y, z] = graph.insert_tokens([
        Token::Element('a'),
        Token::Element('b'),
        Token::Element('w'),
        Token::Element('x'),
        Token::Element('y'),
        Token::Element('z'),
    ])[..]
    {
        let ab = graph.insert_pattern([a, b]);
        let by = graph.insert_pattern([b, y]);
        let yz = graph.insert_pattern([y, z]);
        let xa = graph.insert_pattern([x, a]);
        let xab = graph.insert_patterns([vec![x, ab], vec![xa, b]]);
        let xaby = graph.insert_patterns([vec![xab, y], vec![xa, by]]);
        let xabyz = graph.insert_patterns([vec![xaby, z], vec![xab, yz]]);
        let _wxabyzabbyxabyz = graph.insert_pattern([w, xabyz, ab, by, xabyz]);
    } else {
        panic!("Inserting tokens failed!");
    }
    graph
}
pub fn build_graph2() -> Hypergraph<char> {
    let mut graph = Hypergraph::default();
    if let [a, b, c, d, e, f, g, h, i] = graph.insert_tokens([
        Token::Element('a'),
        Token::Element('b'),
        Token::Element('c'),
        Token::Element('d'),
        Token::Element('e'),
        Token::Element('f'),
        Token::Element('g'),
        Token::Element('h'),
        Token::Element('i'),
    ])[..]
    {
        let ab = graph.insert_pattern([a, b]);
        let bc = graph.insert_pattern([b, c]);
        let ef = graph.insert_pattern([e, f]);
        let def = graph.insert_pattern([d, ef]);
        let cdef = graph.insert_pattern([c, def]);
        let gh = graph.insert_pattern([g, h]);
        let efgh = graph.insert_pattern([ef, gh]);
        let ghi = graph.insert_pattern([gh, i]);
        let abc = graph.insert_patterns([[ab, c], [a, bc]]);
        let cd = graph.insert_pattern([c, d]);
        let bcd = graph.insert_patterns([[bc, d], [b, cd]]);
        let abcd = graph.insert_patterns([[abc, d], [a, bcd]]);
        let efghi = graph.insert_patterns([[efgh, i], [ef, ghi]]);
        let abcdefghi = graph.insert_patterns([vec![abcd, efghi], vec![ab, cdef, ghi]]);
        let aba = graph.insert_pattern([ab, a]);
        let abab = graph.insert_patterns([[aba, b], [ab, ab]]);
        let ababab = graph.insert_patterns([[abab, ab], [ab, abab]]);
        let ababcd = graph.insert_patterns([[ab, abcd], [aba, bcd], [abab, cd]]);
        let ababababcd =
            graph.insert_patterns([vec![ababab, abcd], vec![abab, ababcd], vec![ab, ababab, cd]]);
        let ababcdefghi = graph.insert_patterns([[ab, abcdefghi], [ababcd, efghi]]);
        let _ababababcdefghi = graph.insert_patterns([
            [ababababcd, efghi],
            [abab, ababcdefghi],
            [ababab, abcdefghi],
        ]);
    } else {
        panic!("Inserting tokens failed!");
    }
    graph
}
pub fn build_graph3() -> Hypergraph<char> {
    let mut graph = Hypergraph::default();
    if let [
        d, i, e, space,
        k, a, t, z,
        m, c, u, r,
        h, n, w, f,
    ] = graph.insert_tokens([
        Token::Element('d'),
        Token::Element('i'),
        Token::Element('e'),
        Token::Element(' '),
        Token::Element('k'),
        Token::Element('a'),
        Token::Element('t'),
        Token::Element('z'),
        Token::Element('m'),
        Token::Element('c'),
        Token::Element('u'),
        Token::Element('r'),
        Token::Element('h'),
        Token::Element('n'),
        Token::Element('w'),
        Token::Element('f'),
    ])[..]
    {
        let _mach = graph.insert_pattern([space, m, a, c, h]);
        let _macht = graph.insert_pattern([_mach, t]);
        let t_ = graph.insert_pattern([t, space]);
        let _macht_ = graph.insert_patterns([
            [_macht, space],
            [_mach, t_],
        ]);
        let en = graph.insert_pattern([e, n]);
        let _machen = graph.insert_pattern([_mach, en]);
        let e_mach = graph.insert_pattern([e, _mach]);
        let e_macht_ = graph.insert_patterns([
            [e_mach, t_],
            [e, _macht_],
        ]);
        let e_machen = graph.insert_patterns([
            [e_mach, en],
            [e, _machen],
        ]);

        let die = graph.insert_pattern([d, i, e]);
        let die_ = graph.insert_pattern([die, space]);

        let hund = graph.insert_pattern([h, u, n, d]);

        let wuff = graph.insert_pattern([w, u, f, f]);
        let _wuff = graph.insert_pattern([space, wuff]);
        let _macht_wuff = graph.insert_patterns([
            [_macht_, wuff],
            [_macht, _wuff],
        ]);

        let _hund = graph.insert_pattern([space, hund]);
        let die_hund = graph.insert_patterns([
            [die, _hund],
            [die_, hund],
        ]);
        let _s1 = graph.insert_pattern([die_, k, a, t, z, e_macht_, m, i, a, u]);
        let _s2 = graph.insert_pattern([d, e, r, _hund, _macht_wuff]);
        let _s2 = graph.insert_pattern([die_hund, e_machen, _wuff]);
    } else {
        panic!("Inserting tokens failed!");
    }
    graph
}