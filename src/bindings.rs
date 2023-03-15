use crate::dk::DK;
use crate::key::Key;
use crate::mode::Mode;

#[derive(Default)]
pub struct Bindings {
    map: Vec<(Mode, Vec<Key>, DK)>,
    unmatched_key_passthrough: bool,
}

impl Bindings {
    pub fn add(&mut self, mode: Mode, keys: Vec<Key>, dk: DK) {
        self.map.push((mode, keys, dk));
    }
    pub fn enable_unmatched_key_passthrough(&mut self) {
        self.unmatched_key_passthrough = true;
    }
}
