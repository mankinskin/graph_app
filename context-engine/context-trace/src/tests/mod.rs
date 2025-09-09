use tracing::Level;

use crate::{
    HashMap,
    graph::{
        Hypergraph,
        vertex::{
            has_vertex_index::ToChild,
            parent::{
                Parent,
                PatternIndex,
            },
        },
    },
};
pub mod mock;

pub(crate) mod grammar;
#[macro_use]
pub mod graph;

pub mod env;
pub mod trace;

pub fn init_tracing() {
    tracing_subscriber::fmt()
        .with_max_level(Level::DEBUG)
        .with_target(false)
        .init();
}

#[macro_export]
macro_rules! assert_patterns {
    ($graph:ident,
        $(
            $name:ident => [
                $([$($pat:expr),*]),*$(,)?
            ]
        ),*$(,)?
    ) => {

        let g = $graph.graph();
        $(
            let pats: HashSet<_> =
                $crate::graph::vertex::has_vertex_data::HasVertexData::vertex(&$name, &g).get_child_pattern_set().into_iter().collect();
            assert_eq!(pats, hashset![$(vec![$($pat),*]),*]);
        )*
        #[allow(dropping_references)]
        drop(g);
    };
}
#[macro_export]
macro_rules! assert_not_indices {
    ($graph:ident, $($name:ident),*) => {
        $(
        assert_matches!(
            $graph
            .find_sequence(stringify!($name).chars()),
            Err(_) | Ok(FinishedState { kind: FinishedKind::Incomplete(_), .. })
        );
        )*
    };
}
#[macro_export]
macro_rules! assert_indices {
    ($graph:ident, $($name:ident),*) => {
        $(
        let $name = $graph
            .find_sequence(stringify!($name).chars())
            .unwrap()
            .expect_complete(stringify!($name));
        )*
    };
}
#[macro_export]
macro_rules! expect_tokens {
    ($graph:ident, {$($name:ident),*}) => {

        let g = $graph.graph();
        $(let $name = g.expect_token_child($crate::charify::charify!($name));)*
        #[allow(dropping_references)]
        drop(g);
    };
}
#[macro_export]
macro_rules! insert_tokens {
    ($graph:ident, {$($name:ident),*}) => {
        use itertools::Itertools;
        let ($($name),*) = $crate::trace::has_graph::HasGraphMut::graph_mut(&mut $graph)
            .insert_tokens([
                $(
                    $crate::graph::vertex::token::Token::Element($crate::charify::charify!($name))
                ),*
            ])
            .into_iter()
            .next_tuple()
            .unwrap();
    };
}
pub fn assert_parents(
    graph: &Hypergraph,
    child: impl ToChild,
    parent: impl ToChild,
    pattern_indices: impl IntoIterator<Item = PatternIndex>,
) {
    assert_eq!(
        graph
            .expect_parents(child)
            .clone()
            .into_iter()
            .collect::<HashMap<_, _>>(),
        HashMap::from_iter([(parent.vertex_index(), Parent {
            pattern_indices: pattern_indices.into_iter().collect(),
            width: parent.width(),
        })])
    );
}
