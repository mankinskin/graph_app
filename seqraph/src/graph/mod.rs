use crate::*;
use itertools::Itertools;
use petgraph::graph::DiGraph;
use std::{
    collections::HashMap,
    fmt::Debug,
};

mod child_strings;
mod getters;
mod insert;
pub use {
    child_strings::*,
    getters::*,
    insert::*,
};

#[derive(Debug, Default)]
pub struct Hypergraph<T: Tokenize> {
    graph: indexmap::IndexMap<VertexKey<T>, VertexData>,
}

impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    pub fn index_width(
        &self,
        index: &impl Indexed,
    ) -> TokenPosition {
        self.expect_vertex_data(index.index()).width
    }
    pub fn vertex_count(&self) -> usize {
        self.graph.len()
    }
    //pub fn index_sequence<N: Into<T>, I: IntoIterator<Item = N>>(&mut self, seq: I) -> VertexIndex {
    //    let seq = seq.into_iter();
    //    let tokens = T::tokenize(seq);
    //    let pattern = self.to_token_children(tokens);
    //    self.index_pattern(&pattern[..])
    //}
}
impl<'t, 'a, T> Hypergraph<T>
where
    T: Tokenize + 't + std::fmt::Display,
{
    pub fn to_petgraph(&self) -> DiGraph<(VertexKey<T>, VertexData), Parent> {
        let mut pg = DiGraph::new();
        // id refers to index in Hypergraph
        // idx refers to index in petgraph
        let nodes: HashMap<_, _> = self
            .vertex_iter()
            .map(|(key, node)| {
                let idx = pg.add_node((*key, node.clone()));
                let id = self.expect_index_by_key(key);
                (id, (idx, node))
            })
            .collect();
        nodes.values().for_each(|(idx, node)| {
            let parents = node.get_parents();
            for (p_id, rel) in parents {
                let (p_idx, _p_data) = nodes
                    .get(p_id)
                    .expect("Parent not mapped to node in petgraph!");
                pg.add_edge(*p_idx, *idx, rel.clone());
            }
        });
        pg
    }

    pub fn to_node_child_strings(&self) -> ChildStrings {
        let nodes = self.graph.iter().map(|(key, data)| {
            (
                self.key_data_string(key, data),
                data.to_pattern_strings(self),
            )
        });
        ChildStrings::from_nodes(nodes)
    }
    pub fn pattern_child_strings(
        &self,
        pattern: impl IntoPattern<Item = Child>,
    ) -> ChildStrings {
        let nodes = pattern.into_iter().map(|child| {
            (
                self.index_string(child.index),
                self.expect_vertex_data(child.index)
                    .to_pattern_strings(self),
            )
        });
        ChildStrings::from_nodes(nodes)
    }

    pub(crate) fn pattern_string_with_separator(
        &'a self,
        pattern: impl IntoIterator<Item = impl Indexed>,
        separator: &'static str,
    ) -> String {
        pattern
            .into_iter()
            .map(|child| self.index_string(child.index()))
            .join(separator)
    }
    pub fn separated_pattern_string(
        &'a self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> String {
        self.pattern_string_with_separator(pattern, "_")
    }
    pub fn pattern_string(
        &'a self,
        pattern: impl IntoIterator<Item = impl Indexed>,
    ) -> String {
        self.pattern_string_with_separator(pattern, "")
    }
    pub fn key_data_string(
        &self,
        key: &VertexKey<T>,
        data: &VertexData,
    ) -> String {
        self.key_data_string_impl(key, data, |t| t.to_string())
    }
    pub fn key_data_string_impl(
        &self,
        key: &VertexKey<T>,
        data: &VertexData,
        f: impl Fn(&Token<T>) -> String,
    ) -> String {
        match key {
            VertexKey::Token(token) => f(token),
            VertexKey::Pattern(_) => self.pattern_string(data.expect_any_pattern().1),
        }
    }
    pub fn index_string(
        &self,
        index: impl Indexed,
    ) -> String {
        let (key, data) = self.expect_vertex(index);
        self.key_data_string(key, data)
    }
    pub fn key_string(
        &self,
        key: &VertexKey<T>,
    ) -> String {
        let data = self.expect_vertex_data_by_key(key);
        self.key_data_string(key, data)
    }
}

#[cfg(test)]
#[macro_use]
pub(crate) mod tests {
    use super::*;
    use std::sync::{
        Arc,
        RwLock,
        RwLockReadGuard,
        RwLockWriteGuard,
    };
    pub struct Context {
        pub graph: Hypergraph<char>,
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
                let ababababcd = graph.insert_patterns([
                    [ababab, abcd],
                    [abab, ababcd],
                ]);
                let ababcdefghi = graph.insert_patterns([
                    [ab, abcdefghi],
                    [ababcd, efghi],
                ]);
                let ababababcdefghi = graph.insert_patterns([
                    [ababababcd, efghi],
                    [abab, ababcdefghi],
                    [ababab, abcdefghi],
                ]);
                Context {
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
        let mut graph = Hypergraph::default();
        if let [a, b, c, d] = graph.insert_tokens([
            Token::Element('a'),
            Token::Element('b'),
            Token::Element('c'),
            Token::Element('d'),
        ])[..]
        {
            // ab cd
            // abc d
            // a bcd

            let ab = graph.insert_pattern([a, b]);
            let bc = graph.insert_pattern([b, c]);
            let abc = graph.insert_patterns([[ab, c], [a, bc]]);
            let cd = graph.insert_pattern([c, d]);
            let bcd = graph.insert_patterns([[bc, d], [b, cd]]);
            let _abcd = graph.insert_patterns([[abc, d], [a, bcd]]);
        } else {
            panic!();
        }
        let pg = graph.to_petgraph();
        pg.write_to_file("assets/test_graph1.dot")
            .expect("Failed to write assets/test_graph1.dot file!");
    }
}
