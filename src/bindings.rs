use crate::prelude::*;

#[derive(Debug, Default)]
pub struct Bindings {
    map: HashMap<Vec<Key>, DK>,
}

impl Bindings {
    pub fn get_map(&self) -> HashMap<Vec<Key>, DK> {
        self.map
    }
    pub fn add(&mut self, keys: Vec<Key>, dk: DK) {
        self.map.insert(keys, dk);
    }
    pub fn add_bindings(&mut self, rhs: Bindings) {
        self.map.extend(rhs.map)
    }
}
