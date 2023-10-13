pub mod bands;
pub use bands::*;
pub mod traverser;
pub use traverser::*;

use crate::*;

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