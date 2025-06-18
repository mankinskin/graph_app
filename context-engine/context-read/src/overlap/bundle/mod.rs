use context_trace::{
    graph::vertex::{
        child::Child,
        pattern::Pattern,
    },
    trace::has_graph::HasGraphMut,
};

use super::bands::band::Band;

//pub mod iterator;

#[derive(Default, Clone, Debug)]
pub struct Bundle {
    bundle: Vec<Pattern>,
    end_bound: usize,
}
impl<'p> Bundle {
    pub fn new(band: Band) -> Self {
        Self {
            bundle: vec![band.pattern],
            end_bound: band.end_bound,
        }
    }
    pub fn add_pattern(
        &mut self,
        pattern: Pattern,
    ) {
        self.bundle.push(pattern)
    }
    pub fn wrap_into_band(
        mut self,
        trav: impl HasGraphMut,
    ) -> Band {
        let end_bound = self.end_bound;
        assert!(!self.bundle.is_empty());
        let pattern = if self.bundle.len() == 1 {
            self.bundle.pop().unwrap()
        } else {
            vec![self.wrap_into_child(trav)]
        };
        Band {
            pattern,
            start_bound: 0,
            end_bound,
        }
    }
    pub fn wrap_into_child(
        self,
        mut trav: impl HasGraphMut,
    ) -> Child {
        assert!(!self.bundle.is_empty());
        trav.graph_mut().insert_patterns(self.bundle)
    }
}
