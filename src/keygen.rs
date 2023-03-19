use crate::view::ViewKey;

pub struct ViewKeyGenerator {
    iter: std::ops::RangeFrom<usize>,
}

impl ViewKeyGenerator {
    pub fn new() -> Self {
        Self {
            iter: (0..),
        }
    }
    pub fn next_key(&mut self) -> ViewKey {
        self.iter.next().unwrap().into()
    }
}
