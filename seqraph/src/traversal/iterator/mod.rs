pub mod bands;
pub use bands::*;
pub mod traverser;
pub use traverser::*;

use crate::*;

#[derive(Clone, Debug)]
pub struct StateNext<T> {
    pub prev: DirectedKey,
    pub new: Vec<NewEntry>,
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
impl NextStates {
    pub fn into_states(self) -> Vec<TraversalState> {
        match self {
            Self::Parents(state) =>
                state.inner.iter()
                    .map(|s| TraversalState {
                        prev: state.prev,
                        new: state.new.clone(),
                        kind: InnerKind::Parent(s.clone())
                    })
                    .collect_vec(),
            Self::Prefixes(state) =>
                state.inner.iter()
                    .map(|s|
                        TraversalState {
                            prev: state.prev,
                            new: state.new.clone(),
                            kind: InnerKind::Child(s.clone()),
                        }
                    )
                    .collect_vec(),
            Self::Child(state) =>
                vec![TraversalState {
                    prev: state.prev,
                    new: state.new,
                    kind: InnerKind::Child(state.inner),
                }],
            Self::End(_) => vec![],
            Self::Empty => vec![],
        }
    }
}

pub type IterTrav<'a, It> = <It as TraversalIterator<'a>>::Trav;
pub type IterKind<'a, It> = TravKind<IterTrav<'a, It>>;

pub trait TraversalIterator<
    'a, 
>: Iterator<Item = (usize, TraversalState)> + Sized + ExtendStates + PruneStates + Debug {
    type Trav: TraversalFolder + 'a;
    type Policy: DirectedTraversalPolicy<Trav=Self::Trav>;
    type NodeVisitor: NodeVisitor;

    fn trav(&self) -> &'a Self::Trav;
}
impl<'a, Trav, S, O> TraversalIterator<'a> for OrderedTraverser<'a, Trav, S, O>
    where
        Trav: TraversalFolder + 'a,
        S: DirectedTraversalPolicy<Trav=Trav>,
        O: NodeVisitor,
{
    type Trav = Trav;
    type Policy = S;
    type NodeVisitor = O;
    fn trav(&self) -> &'a Self::Trav {
        self.trav
    }
}
impl<'a, 'b: 'a, I: TraversalIterator<'b>> TraversalIterator<'b> for TraversalContext<'a, 'b, I> {
    type Trav = I::Trav;
    type Policy = I::Policy;
    type NodeVisitor = I::NodeVisitor;
    fn trav(&self) -> &'b Self::Trav {
        self.iter.trav()
    }
}