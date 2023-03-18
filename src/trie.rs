use crate::bindings::Bindings;
use crate::prelude::*;

#[derive(Default)]
pub struct TrieNode {
    pair: Option<DK>,
    children: HashMap<Key, TrieNode>,
}

impl TrieNode {
    pub fn from_ancestor_path(
        ancestor_path: Vec<ViewKey>,
        view_map: &HashMap<ViewKey, ViewRef>,
    ) -> Self {
        let mut slf = Self::default();
        ancestor_path
            .iter()
            .map(|view_key| view_map.get(view_key).unwrap().borrow().get_key_bindings())
            .for_each(|b| slf.add_bindings(b));
        slf
    }
    fn add_bindings(&mut self, bindings: Bindings) {
        for (keys, dk) in bindings.get_map() {
            self.insert(dk, &keys);
        }
    }
    fn insert(&mut self, dk: DK, keys: &[Key]) {
        let mut cur = self;
        for key in keys {
            cur = cur.children.entry(*key).or_insert(TrieNode::default());
        }
        cur.pair = Some(dk);
    }

    fn match_prefix(&self, prefix: &[Key]) -> Option<DK> {
        let mut cur = self;
        for key in prefix {
            if let Some(next) = cur.children.get(key) {
                cur = next;
            } else {
                return None;
            }
        }
        Some(DK::Noop)
    }

    pub fn longest_prefix<'a>(&self, input: &'a [Key]) -> Option<(DK, &'a [Key])> {
        for i in (0..input.len()).rev() {
            let prefix = &input[..=i];
            if let Some(res) = self.match_prefix(prefix) {
                assert!(input.len() >= input[i + 1..].len());
                return Some((res, &input[i + 1..]));
            }
        }
        None
    }
}
