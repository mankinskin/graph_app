use std::collections::VecDeque;

use context_trace::graph::vertex::{
    child::Child,
    pattern::Pattern,
    wide::Wide,
};

use crate::expansion::chain::link::StartBound;

#[derive(Debug, Clone)]
pub enum StackLocation {
    Head,
    Nested {
        nested_index: usize,
        inner_location: Box<StackLocation>,
    },
}
//#[derive(Debug, Clone)]
//pub struct NestedStack {
//    pub stack: OverlapStack,
//    pub back_context: Pattern,
//    pub start_bound: usize,
//}

#[derive(Debug, Clone)]
pub struct OverlapStack {
    pub head: Pattern,
    pub overlaps: VecDeque<StackBand>,
}

#[derive(Debug, Clone)]
pub enum StackBandEnd {
    Single(Child),
    Stack(OverlapStack),
}
#[derive(Debug, Clone)]
pub struct StackBand {
    pub back_context: Child,
    pub expansion: StackBandEnd,
}
impl StartBound for StackBand {
    fn start_bound(&self) -> usize {
        self.back_context.width()
    }
}

impl OverlapStack {
    pub fn new(head_index: Child) -> Self {
        Self {
            head: vec![head_index],
            overlaps: VecDeque::default(),
        }
    }

    ///// Find if an expansion can be appended to any band in this stack
    //pub fn find_appendable_band(
    //    &self,
    //    expansion: &BandExpansion,
    //) -> Option<StackLocation> {
    //    // Check if expansion can be appended to head band
    //    if self.head.pattern_width() == expansion.start_bound {
    //        return Some(StackLocation::Head);
    //    }

    //    // Recursively check nested stacks
    //    for (nested_index, nested) in self.nested_stacks.iter().enumerate() {
    //        if let Some(location) = nested.stack.find_appendable_band(expansion)
    //        {
    //            return Some(StackLocation::Nested {
    //                nested_index,
    //                inner_location: Box::new(location),
    //            });
    //        }
    //    }
    //    None
    //}
}
