use context_trace::{
    graph::{
        vertex::atom::Atom,
        Hypergraph,
    },
    insert_atoms,
    insert_patterns,
};

pub fn build_graph1() -> Hypergraph {
    let graph = Hypergraph::default();
    insert_atoms!(graph, {a, b, w, x, y, z});
    // Single patterns
    insert_patterns!(graph,
        ab => [a, b],
        by => [b, y],
        yz => [y, z],
        xa => [x, a]
    );
    // Multiple patterns (all with same element count per inner array)
    insert_patterns!(graph,
        xab => [[x, ab], [xa, b]],
        xaby => [[xab, y], [xa, by]],
        xabyz => [[xaby, z], [xab, yz]]
    );
    // Single pattern with 5 elements
    insert_patterns!(graph,
        _wxabyzabbyxabyz => [w, xabyz, ab, by, xabyz]
    );
    graph
}
pub fn build_graph2() -> Hypergraph {
    let graph = Hypergraph::default();
    insert_atoms!(graph, {a, b, c, d, e, f, g, h, i});
    // Single patterns (2 elements)
    insert_patterns!(graph,
        ab => [a, b],
        bc => [b, c],
        ef => [e, f],
        gh => [g, h],
        cd => [c, d]
    );
    // Single patterns (various sizes)
    insert_patterns!(graph,
        def => [d, ef],
        cdef => [c, def],
        efgh => [ef, gh],
        ghi => [gh, i],
        aba => [ab, a]
    );
    // Multiple patterns (2 elements each)
    insert_patterns!(graph,
        abc => [[ab, c], [a, bc]],
        bcd => [[bc, d], [b, cd]],
        abcd => [[abc, d], [a, bcd]],
        efghi => [[efgh, i], [ef, ghi]],
        abab => [[aba, b], [ab, ab]],
        ababab => [[abab, ab], [ab, abab]]
    );
    // Use graph methods directly for patterns with mixed element counts
    let abcdefghi =
        graph.insert_patterns([vec![abcd, efghi], vec![ab, cdef, ghi]]);
    // Multiple patterns (3 elements each)
    insert_patterns!(graph,
        ababcd => [[ab, abcd], [aba, bcd], [abab, cd]],
        ababababcd => [[ababab, abcd], [abab, ababcd], [ab, ababab, cd]]
    );
    // Patterns depending on abcdefghi
    insert_patterns!(graph,
        ababcdefghi => [[ab, abcdefghi], [ababcd, efghi]],
        _ababababcdefghi => [[ababababcd, efghi], [abab, ababcdefghi], [ababab, abcdefghi]]
    );
    graph
}
pub fn build_graph3() -> Hypergraph {
    let graph = Hypergraph::default();
    // Insert atoms manually to avoid tuple size limits (max 12 elements)
    // Note: 'space' must be inserted manually as it's not a single-character identifier
    let [d, i, e, k, a, t, z, m, c, u, r, h, n, w, f, sp] =
        graph.insert_atoms([
            Atom::Element('d'),
            Atom::Element('i'),
            Atom::Element('e'),
            Atom::Element('k'),
            Atom::Element('a'),
            Atom::Element('t'),
            Atom::Element('z'),
            Atom::Element('m'),
            Atom::Element('c'),
            Atom::Element('u'),
            Atom::Element('r'),
            Atom::Element('h'),
            Atom::Element('n'),
            Atom::Element('w'),
            Atom::Element('f'),
            Atom::Element(' '),
        ])[..]
    else {
        panic!()
    };

    insert_patterns!(graph,
        _mach => [sp, m, a, c, h],
        en => [e, n]
    );
    insert_patterns!(graph,
        _macht => [_mach, t]
    );
    insert_patterns!(graph,
        t_ => [t, sp],
        _machen => [_mach, en],
        e_mach => [e, _mach]
    );
    insert_patterns!(graph,
        _macht_ => [[_macht, sp], [_mach, t_]]
    );
    insert_patterns!(graph,
        e_macht_ => [[e_mach, t_], [e, _macht_]],
        e_machen => [[e_mach, en], [e, _machen]]
    );
    insert_patterns!(graph,
        die => [d, i, e],
        hund => [h, u, n, d],
        wuff => [w, u, f, f]
    );
    insert_patterns!(graph,
        die_ => [die, sp],
        _wuff => [sp, wuff],
        _hund => [sp, hund]
    );
    insert_patterns!(graph,
        _macht_wuff => [[_macht_, wuff], [_macht, _wuff]],
        die_hund => [[die, _hund], [die_, hund]]
    );
    insert_patterns!(graph,
        _s1 => [die_, k, a, t, z, e_macht_, m, i, a, u],
        _s2a => [d, e, r, _hund, _macht_wuff],
        _s2b => [die_hund, e_machen, _wuff]
    );
    graph
}
