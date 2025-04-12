use crate::graph::vertex::child::Child;
use std::fmt::Debug;

pub trait PathComplete: Sized + Debug {
    fn as_complete(&self) -> Option<Child>;

    fn is_complete(&self) -> bool {
        self.as_complete().is_some()
    }
    #[track_caller]
    fn unwrap_complete(self) -> Child {
        self.as_complete()
            .unwrap_or_else(|| panic!("Unable to unwrap {:?} as complete.", self))
    }
    #[track_caller]
    fn expect_complete(
        self,
        msg: &str,
    ) -> Child {
        self.as_complete()
            .unwrap_or_else(|| panic!("Unable to unwrap {:?} as complete: {}", self, msg))
    }
}
