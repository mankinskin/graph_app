use derive_more::{Display, From};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Hash, Debug, PartialEq, Eq, From, Serialize, Deserialize, Clone, Copy, Display)]
pub struct VertexKey(Uuid);
impl VertexKey {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
//#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, new, Serialize, Deserialize)]
//pub enum VertexKey<T: Tokenize = TokenOf<BaseGraphKind>> {
//    Pattern(Child),
//    Token(Token<T>, VertexIndex)
//}
//impl<T: Tokenize> HasVertexIndex for VertexKey<T> {
//    fn vertex_index(&self) -> VertexIndex {
//        match self {
//            Self::Token(_token, index) => *index,
//            Self::Pattern(child) => child.vertex_index(),
//        }
//    }
//}
//impl<T: Tokenize> Borrow<VertexIndex> for VertexKey<T> {
//    fn borrow(&self) -> &VertexIndex {
//        match self {
//            Self::Token(_token, index) => index,
//            Self::Pattern(child) => &child.index,
//        }
//    }
//}

