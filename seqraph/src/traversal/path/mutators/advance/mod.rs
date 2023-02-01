use crate::*;

pub mod exit;
pub mod from_advanced;
pub mod into_advanced;
pub mod into_primer;

pub use exit::*;
pub use from_advanced::*;
pub use into_advanced::*;
pub use into_primer::*;


/// advance path leaf position in graph
pub trait Advance:
    AdvanceExit
    + PathPop
    + PathAppend
    //+ AdvanceWidth
    + Sized
{
    fn advance<
        Trav: Traversable,
    >(&mut self, trav: &Trav) -> ControlFlow<()> {
        //if self.is_finished(trav) {
        //    ControlFlow::BREAK
        //} else {
            let graph = trav.graph();
            // skip path segments with no successors
            if let Some(location) = std::iter::from_fn(|| 
                self.pop_path()
            ).find_map(|mut location| {
                let pattern = graph.expect_pattern_at(&location);
                Trav::Direction::pattern_index_next(pattern.borrow(), location.sub_index)
                    .map(|next| {
                        location.sub_index = next;
                        location
                    })
            }) {
                self.path_append(trav, location);
                ControlFlow::CONTINUE
            } else {
                self.advance_exit_pos(trav)
            }
        //}
    }
}
impl<T: 
    AdvanceExit
    //+ AdvanceWidth
    + PathPop
    + PathAppend
    + Sized
> Advance for T {
}

//pub trait AdvanceWidth {
//    fn advance_width(&mut self, width: usize);
//}
//impl <T: WideMut> AdvanceWidth for T {
//    fn advance_width(&mut self, width: usize) {
//        *self.width_mut() += width;
//    }
//}
//
//pub trait AddMatchWidth: AdvanceWidth + LeafChild<End> {
//    fn add_match_width<
//        T: Tokenize,
//        D: MatchDirection,
//        Trav: Traversable<T>,
//    >(&mut self, trav: &Trav) {
//        let leaf = self.leaf_child(trav);
//        self.advance_width(leaf.width);
//    }
//}
//impl<T: AdvanceWidth + LeafChild<End>> AddMatchWidth for T {
//}
//impl AdvanceWidth for QueryRangePath {
//    fn advance_width(&mut self, _width: usize) {
//    }
//}