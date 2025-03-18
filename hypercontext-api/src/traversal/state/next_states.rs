use super::{
    bottom_up::parent::ParentState,
    top_down::{
        child::ChildState,
        end::EndState,
    },
    traversal::TraversalState,
    InnerKind,
};
use itertools::Itertools;

use crate::traversal::cache::key::prev::PrevKey;
#[derive(Clone, Debug)]
pub struct StateNext<T> {
    pub prev: PrevKey,
    //pub new: Vec<NewEntry>,
    pub inner: T,
}

#[derive(Clone, Debug)]
pub enum NextStates {
    Parents(StateNext<Vec<ParentState>>),
    Prefixes(StateNext<Vec<ChildState>>),
    End(StateNext<EndState>),
    Child(StateNext<ChildState>),
    Empty,
}
impl IntoIterator for NextStates {
    type Item = TraversalState;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        self.into_states().into_iter()
    }
}

impl NextStates {
    pub fn into_states(self) -> Vec<TraversalState> {
        match self {
            Self::Parents(state) => state
                .inner
                .iter()
                .map(|s| TraversalState {
                    prev: state.prev,
                    kind: InnerKind::Parent(s.clone()),
                })
                .collect_vec(),
            Self::Prefixes(state) => state
                .inner
                .iter()
                .map(|s| TraversalState {
                    prev: state.prev,
                    kind: InnerKind::Child(s.clone()),
                })
                .collect_vec(),
            Self::Child(state) => vec![TraversalState {
                prev: state.prev,
                kind: InnerKind::Child(state.inner),
            }],
            Self::End(_) => vec![],
            Self::Empty => vec![],
        }
    }
}
