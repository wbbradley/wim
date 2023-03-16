use crate::dk::DK;
use crate::key::Key;
use crate::view::ViewKey;
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct Bindings {
    map: HashMap<(ViewKey, Vec<Key>), DK>,
}

impl Bindings {
    pub fn add(&mut self, view_key: &ViewKey, keys: Vec<Key>, dk: DK) {
        self.map.insert((view_key.clone(), keys), dk);
    }
    pub fn translate(&self, key: Key) -> Option<DK> {
        panic!("translate not impl for {}", key)
    }
    pub fn add_bindings(&mut self, rhs: Bindings) {
        self.map.extend(rhs.map)
    }
}
