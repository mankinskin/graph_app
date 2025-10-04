pub mod block_iter;

use context_trace::*;

use std::{
    fmt::Debug,
    str::Chars,
};

pub trait ToNewTokenIndices: Debug {
    fn to_new_token_indices<'a: 'g, 'g, G: HasGraphMut<Kind = BaseGraphKind>>(
        self,
        graph: &'a mut G,
    ) -> NewTokenIndices;
}

impl ToNewTokenIndices for NewTokenIndices {
    fn to_new_token_indices<
        'a: 'g,
        'g,
        G: HasGraphMut<Kind = BaseGraphKind>,
    >(
        self,
        _graph: &'a mut G,
    ) -> NewTokenIndices {
        self
    }
}
impl ToNewTokenIndices for Chars<'_> {
    fn to_new_token_indices<
        'a: 'g,
        'g,
        G: HasGraphMut<Kind = BaseGraphKind>,
    >(
        self,
        graph: &'a mut G,
    ) -> NewTokenIndices {
        graph.graph_mut().new_token_indices(self)
    }
}
//impl<T: Tokenize> ToNewTokenIndices<T> for Vec<T> {
//    fn to_new_token_indices<'a: 'g, 'g, G: HasGraphMut>(
//        self,
//        graph: &'a mut G,
//    ) -> NewTokenIndices {
//        graph.graph_mut().new_token_indices(self)
//    }
//}

//impl<Iter: IntoIterator<Item = DefaultToken> + Debug + Send + Sync> ToNewTokenIndices<DefaultToken>
//    for Iter
//{
//    fn to_new_token_indices<'a: 'g, 'g, G: HasGraphMut<Kind = BaseGraphKind>>(
//        self,
//        graph: &'a mut G,
//    ) -> NewTokenIndices {
//        graph.graph_mut().new_token_indices(self)
//    }
//}
