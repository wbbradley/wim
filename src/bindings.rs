use crate::prelude::*;

#[derive(Debug, Default)]
pub struct Bindings {
    map: HashMap<(ViewKey, Vec<Key>), DK>,
}

impl Bindings {
    pub fn add(&mut self, view_key: &ViewKey, keys: Vec<Key>, dk: DK) {
        self.map.insert((view_key.clone(), keys), dk);
    }
    pub fn translate(&self, key: Key) -> Option<DK> {
        let mut matches = vec![];
        for ((view_key, keys), dk) in &self.map {
            if let Some(k) = keys.get(0) {
                if *k == key {
                    matches.push((view_key.clone(), &keys[1..], dk.clone()))
                }
            }
        }
        if matches.is_empty() {
            return Some(DK::SendKey(None, key));
        }
        log::trace!("matches: {:?}", matches);
        let mut choices: HashMap<Key, (ViewKey, DK)> = Default::default();
        for (view_key, keys, dk) in matches {
            let next_key: Key = if !keys.is_empty() { keys[0] } else { Key::None };
            choices.insert(next_key, (view_key, dk));
        }
        if choices.len() == 1 {
            for (key, (_view_key, dk)) in choices.iter() {
                if *key == Key::None {
                    return Some(dk.clone());
                }
            }
        }
        Some(DK::Trie { choices })
    }
    pub fn add_bindings(&mut self, rhs: Bindings) {
        self.map.extend(rhs.map)
    }
}
