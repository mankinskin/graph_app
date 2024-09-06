use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Hash, Debug, PartialEq, Eq, From, Serialize, Deserialize, Clone, Copy, Display)]
pub struct PatternId(Uuid);
impl PatternId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}