use crate::{
    graph::vertex::{
        child::Child,
        pattern::IntoPattern,
    },
    path::{
        accessors::complete::PathComplete,
        structs::{
            query_range_path::FoldablePath,
            rooted::pattern_range::PatternRangePath,
        },
    },
};

use super::fold::state::FoldState;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum FoundRange {
    Complete(Child),
    Incomplete(FoldState),
}

impl FoundRange {
    pub fn unwrap_complete(self) -> Child {
        self.expect_complete("Unable to unwrap complete FoundRange")
    }
    pub fn unwrap_incomplete(self) -> FoldState {
        self.expect_incomplete("Unable to unwrap incomplete FoundRange")
    }
    pub fn expect_complete(
        self,
        msg: &str,
    ) -> Child {
        match self {
            Self::Complete(c) => c,
            _ => panic!("{}", msg),
        }
    }
    pub fn expect_incomplete(
        self,
        msg: &str,
    ) -> FoldState {
        match self {
            Self::Incomplete(s) => s,
            _ => panic!("{}", msg),
        }
    }
}
impl PathComplete for FoundRange {
    /// returns child if reduced to single child
    fn as_complete(&self) -> Option<Child> {
        match self {
            Self::Complete(c) => Some(*c),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FinishedState {
    pub result: FoundRange,
    //pub query: QueryState,
}

impl FinishedState {
    pub fn new(
        result: impl Into<FoundRange>,
        //query: impl Into<QueryState>,
    ) -> Self {
        Self {
            result: result.into(),
            //query: query.into(),
        }
    }
    #[track_caller]
    pub fn unwrap_complete(self) -> Child {
        self.result.unwrap_complete()
    }
    #[allow(unused)]
    #[track_caller]
    pub fn expect_complete(
        self,
        msg: &str,
    ) -> Child {
        self.result.expect_complete(msg)
    }
}

impl FinishedState {
    #[allow(unused)]
    pub fn new_complete(
        query: impl IntoPattern,
        index: impl crate::graph::vertex::has_vertex_index::ToChild,
    ) -> Self {
        let query = query.into_pattern();
        Self {
            result: FoundRange::Complete(index.to_child()),
        }
    }
}
