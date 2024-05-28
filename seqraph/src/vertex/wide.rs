use crate::{
    graph::kind::GraphKind,
    traversal::cache::key::{
        DirectedKey,
        DownKey,
        UpKey,
    },
};

use super::{
    child::Child,
    pattern::{
        Pattern,
        pattern_width,
    },
    TokenPosition,
    VertexData,
};

pub trait Wide {
    fn width(&self) -> usize;
}

impl Wide for Pattern {
    fn width(&self) -> TokenPosition {
        pattern_width(self)
    }
}

impl Wide for [Child] {
    fn width(&self) -> TokenPosition {
        pattern_width(self)
    }
}

impl Wide for char {
    fn width(&self) -> usize {
        1
    }
}
//impl<R> Wide for RolePath<R> {
//    fn width(&self) -> usize {
//        self.width
//    }
//}

impl<T: Wide> Wide for &'_ T {
    fn width(&self) -> usize {
        (**self).width()
    }
}

impl<T: Wide> Wide for &'_ mut T {
    fn width(&self) -> usize {
        (**self).width()
    }
}

//impl Wide for OverlapPrimer {
//    fn width(&self) -> usize {
//        self.width
//    }
//}
impl<G: GraphKind> Wide for VertexData<G> {
    fn width(&self) -> usize {
        self.width
    }
}

impl Wide for DirectedKey {
    fn width(&self) -> usize {
        self.index.width()
    }
}

impl Wide for UpKey {
    fn width(&self) -> usize {
        self.index.width()
    }
}

impl Wide for DownKey {
    fn width(&self) -> usize {
        self.index.width()
    }
}

pub trait WideMut: Wide {
    fn width_mut(&mut self) -> &mut usize;
}
//impl<P: WideMut> WideMut for OriginPath<P> {
//    fn width_mut(&mut self) -> &mut usize {
//        self.postfix.width_mut()
//    }
//}
//impl WideMut for OverlapPrimer {
//    fn width_mut(&mut self) -> &mut usize {
//        &mut self.width
//    }
//}
