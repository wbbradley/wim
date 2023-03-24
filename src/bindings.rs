use crate::prelude::*;

#[derive(Default)]
pub struct Bindings {
    map: HashMap<Vec<Key>, DK>,
}

pub trait KeysLike {
    fn parse_keys(self) -> Vec<Key>;
}

impl Bindings {
    pub fn insert<T>(&mut self, keys_like: T, dk: DK)
    where
        T: KeysLike,
    {
        self.map.insert(keys_like.parse_keys(), dk);
    }
    pub fn get_map(self) -> HashMap<Vec<Key>, DK> {
        self.map
    }
}

impl KeysLike for &str {
    fn parse_keys(self) -> Vec<Key> {
        self.chars()
            .into_iter()
            .map(|x| Key::Ascii(x.into()))
            .collect()
    }
}

impl KeysLike for Key {
    fn parse_keys(self) -> Vec<Key> {
        vec![self]
    }
}
