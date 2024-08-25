use std::borrow::Borrow;
use serde::{
    Deserialize,
    Serialize,
};

use parent::Parent;
use pattern::Pattern;

use crate::{
    graph::kind::{
        BaseGraphKind,
        GraphKind,
        VertexKeyOf,
    },
    HashMap,
};
use crate::graph::vertex::{
    has_vertex_index::{
        HasVertexIndex,
        ToChild,
    },
    pattern::{
        IntoPattern,
        pattern_range::PatternRangeIndex,
    },
    token::Tokenize,
    wide::Wide
};
use crate::graph::vertex::data::VertexData;

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

pub type VertexEntry<'x, G = BaseGraphKind> = indexmap::map::Entry<'x, VertexKeyOf<G>, VertexData<G>>;
pub type IndexedVertexEntry<'x, G = BaseGraphKind> = indexmap::map::IndexedEntry<'x, VertexKeyOf<G>, VertexData<G>>;
pub type VertexIndex = usize;
pub type VertexParents = HashMap<VertexIndex, Parent>;
pub type ChildPatterns = HashMap<PatternId, Pattern>;
pub type PatternId = usize;
pub type TokenPosition = usize;
pub type IndexPosition = usize;
pub type IndexPattern = Vec<VertexIndex>;
pub type VertexPatternView<'a, G> = Vec<&'a VertexData<G>>;

