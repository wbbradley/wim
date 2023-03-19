use crate::bindings::Bindings;
use crate::prelude::*;

#[derive(Default)]
pub struct TrieNode {
    dk: Option<DK>,
    children: HashMap<Key, TrieNode>,
}

impl TrieNode {
    pub fn from_ancestor_path(
        ancestor_path: Vec<ViewKey>,
        view_map: &HashMap<ViewKey, ViewRef>,
        root_view_key: ViewKey,
    ) -> Self {
        let mut slf = Self::default();
        ancestor_path
            .iter()
            .map(|view_key| {
                view_map
                    .get(view_key)
                    .unwrap()
                    .borrow()
                    .get_key_bindings(root_view_key)
            })
            .for_each(|b| slf.add_bindings(b));
        slf
    }
    fn add_bindings(&mut self, bindings: Bindings) {
        for (keys, dk) in bindings {
            self.insert(dk, &keys);
        }
    }
    fn insert(&mut self, dk: DK, keys: &[Key]) {
        let mut cur = self;
        for key in keys {
            cur = cur.children.entry(*key).or_insert(TrieNode::default());
        }
        cur.dk = Some(dk);
    }

    fn match_prefix(&self, prefix: &[Key]) -> Option<DK> {
        let mut cur = self;
        for key in prefix {
            if key == &Key::None {
                return cur.dk.clone();
            }
            if let Some(next) = cur.children.get(key) {
                cur = next;
            } else {
                return None;
            }
        }
        if cur.children.is_empty() {
            cur.dk.clone()
        } else {
            // TODO: return the possible choices...
            None
        }
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
