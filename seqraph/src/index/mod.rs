use std::collections::HashMap;

use crate::{
    vertex::*,
    search::*,
    r#match::*,
    Hypergraph, ChildLocation,
};

mod indexer;
pub use indexer::*;
mod index_direction;
pub use index_direction::*;

#[derive(Clone, Debug)]
pub(crate) enum IndexingNode {
    Start(Child),
    Parent(ChildPath),
    Child(ChildPath, ChildPath, Child), // start path, root, end path, child
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedPath {
    pub(crate) indexed: IndexedChild,
    pub(crate) end_path: Option<ChildPath>,
    pub(crate) remainder: Option<Pattern>
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexedChild {
    pub(crate) location: ChildLocation,
    pub(crate) context: Option<Child>,
    pub(crate) inner: Child,
}
impl IndexedPath {
    pub fn new(indexed: IndexedChild, end_path: ChildPath, remainder: impl IntoPattern<Item=impl AsChild>) -> Self {
        Self {
            indexed,
            end_path: if end_path.is_empty() {
                None
            } else {
                Some(end_path)
            },
            remainder: if remainder.is_empty() {
                None
            } else {
                Some(remainder.into_pattern())
            },
        }
    }
    pub fn remainder(indexed: IndexedChild, remainder: impl IntoPattern<Item=impl AsChild>) -> Self {
        Self {
            indexed,
            end_path: None,
            remainder: if remainder.is_empty() {
                None
            } else {
                Some(remainder.into_pattern())
            },
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subgraph {
    graph: HashMap<VertexIndex, SubgraphVertex>,
}
impl Subgraph {
    pub fn new() -> Self {
        Self {
            graph: HashMap::new(),
        }
    }
    pub fn add_index_parent(&mut self, root: VertexIndex, parent: Child, pi: PatternIndex) {
        self.graph.entry(root).and_modify(|v|
            v.add_parent(parent, pi)
        )
        .or_insert_with(|| {
            let mut v = SubgraphVertex::new();
            v.add_parent(parent, pi);
            v
        });
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubgraphVertex {
    parents: VertexParents,
}
impl SubgraphVertex {
    fn new() -> Self {
        Self {
            parents: Default::default(),
        }
    }
    fn add_parent(&mut self, parent: Child, pi: PatternIndex) {
        self.parents.entry(parent.index)
            .and_modify(|p|
                p.add_pattern_index(pi.pattern_id, pi.sub_index)
            )
            .or_insert_with(|| {
                let mut p = Parent::new(parent.width);
                p.add_pattern_index(pi.pattern_id, pi.sub_index);
                p
            });
    }
}
type IndexingResult = Result<IndexedPath, NoMatch>;

impl<'t, 'g, T> Hypergraph<T>
where
    T: Tokenize + 't,
{
    //pub fn indexer() -> Indexer<T, Right> {
    //    Indexer::new(self)
    //}
    //pub(crate) fn index_found(
    //    &mut self,
    //    found_path: FoundPath,
    //) -> (Option<Child>, Child, Option<Child>, Pattern) {
    //    self.indexer().index_found(found_path)
    //}
    ///// does not include location
    //pub(crate) fn index_pre_context_at(
    //    &mut self,
    //    location: &ChildLocation,
    //) -> Result<Child, NoMatch> {
    //    self.indexer().index_pre_context_at(location)
    //}
    ///// does not include location
    //pub(crate) fn index_post_context_at(
    //    &mut self,
    //    location: &ChildLocation,
    //) -> Result<Child, NoMatch> {
    //    self.indexer().index_post_context_at(location)
    //}
}