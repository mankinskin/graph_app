use crate::*;

pub trait Wide {
    fn width(&self) -> usize;
}
impl Wide for Pattern {
    fn width(&self) -> TokenPosition {
        pattern::pattern_width(self)
    }
}
impl Wide for [Child] {
    fn width(&self) -> TokenPosition {
        pattern::pattern_width(self)
    }
}

impl Wide for char {
    fn width(&self) -> usize {
        1
    }
}

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

pub trait WideMut {
    fn width_mut(&mut self) -> &mut usize;
}