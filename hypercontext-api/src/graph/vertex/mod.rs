
use parent::Parent;
use pattern::Pattern;

use crate::HashMap;
use crate::graph::vertex::data::VertexData;
use crate::graph::vertex::key::VertexKey;
use crate::graph::vertex::pattern::id::PatternId;

pub mod child;
pub mod has_vertex_index;
pub mod location;
pub mod parent;
pub mod pattern;
pub mod token;
pub mod has_vertex_data;
pub mod wide;
pub mod data;
pub mod key;
pub mod has_vertex_key;

pub type VertexEntry<'x> = indexmap::map::Entry<'x, VertexKey, VertexData>;
pub type IndexedVertexEntry<'x> = indexmap::map::IndexedEntry<'x, VertexKey, VertexData>;
pub type VertexIndex = usize;
pub type VertexParents = HashMap<VertexIndex, Parent>;
pub type ChildPatterns = HashMap<PatternId, Pattern>;
pub type TokenPosition = usize;
pub type IndexPosition = usize;
pub type IndexPattern = Vec<VertexIndex>;
pub type VertexPatternView<'a> = Vec<&'a VertexData>;

