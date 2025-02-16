use crate::{
    graph::vertex::child::Child,
    path::mutators::move_path::key::TokenPosition,
    traversal::cache::{
        entry::PositionCache,
        key::directed::DirectedPosition,
    },
    HashMap,
};

pub type DirectedPositions = HashMap<TokenPosition, PositionCache>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VertexCache {
    pub bottom_up: DirectedPositions,
    pub top_down: DirectedPositions,
    pub index: Child,
}

impl From<Child> for VertexCache {
    fn from(index: Child) -> Self {
        Self {
            index,
            bottom_up: Default::default(),
            top_down: Default::default(),
        }
    }
}

impl VertexCache {
    pub fn start(index: Child) -> Self {
        let bottom_up = HashMap::default();
        //bottom_up.insert(
        //    index.width().into(),
        //    PositionCache::start(index)
        //);
        Self {
            bottom_up,
            index,
            top_down: Default::default(),
        }
    }
    pub fn dir(
        &self,
        pos: &DirectedPosition,
    ) -> &DirectedPositions {
        match pos {
            DirectedPosition::BottomUp(_) => &self.bottom_up,
            DirectedPosition::TopDown(_) => &self.top_down,
        }
    }
    pub fn dir_mut(
        &mut self,
        pos: &DirectedPosition,
    ) -> &mut DirectedPositions {
        match pos {
            DirectedPosition::BottomUp(_) => &mut self.bottom_up,
            DirectedPosition::TopDown(_) => &mut self.top_down,
        }
    }
    pub fn get(
        &self,
        pos: &DirectedPosition,
    ) -> Option<&PositionCache> {
        self.dir(pos).get(pos.pos())
    }
    pub fn get_mut(
        &mut self,
        pos: &DirectedPosition,
    ) -> Option<&mut PositionCache> {
        self.dir_mut(pos).get_mut(pos.pos())
    }
    pub fn insert(
        &mut self,
        pos: &DirectedPosition,
        cache: PositionCache,
    ) {
        self.dir_mut(pos).insert(*pos.pos(), cache);
    }
}
