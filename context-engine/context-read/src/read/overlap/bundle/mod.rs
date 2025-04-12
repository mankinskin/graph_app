use context_search::{
    graph::vertex::pattern::Pattern,
    traversal::traversable::TraversableMut,
};

use super::chain::band::Band;

pub mod iterator;

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
        mut trav: impl TraversableMut,
    ) -> Band {
        assert!(!self.bundle.is_empty());
        let pattern = if self.bundle.len() == 1 {
            self.bundle.pop().unwrap()
        } else {
            vec![trav.graph_mut().insert_patterns(self.bundle)]
        };
        Band {
            pattern,
            start_bound: 0,
            end_bound: self.end_bound,
        }
    }
}
