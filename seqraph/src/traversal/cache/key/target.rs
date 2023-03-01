use crate::*;

pub trait TargetKey {
    fn target_key(&self) -> CacheKey;
}
impl TargetKey for TraversalState {
    fn target_key(&self) -> CacheKey {
        match &self.kind {
            InnerKind::Parent(state) => state.root_key(),
            InnerKind::Child(state) => state.leaf_key(),
            InnerKind::End(state) => state.target_key(),
        }
    }
}
impl TargetKey for EndState {
    fn target_key(&self) -> CacheKey {
        match &self.kind {
            EndKind::Range(_) => self.target,//CacheKey::new(state.path.role_leaf_child::<End, _>(trav), *self.query_pos()),
            EndKind::Postfix(_) => self.root_key(),
            EndKind::Prefix(_) => self.target,
            EndKind::Complete(c) => CacheKey::new(*c, *self.query_pos()),
        }
    }
}