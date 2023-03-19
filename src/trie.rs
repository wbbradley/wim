use crate::bindings::Bindings;
use crate::prelude::*;

#[derive(Debug, Default)]
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

    fn match_prefix<'a>(&'a self, prefix: &[Key]) -> PrefixMatch<'a> {
        let mut cur = self;
        for key in prefix {
            if key == &Key::None {
                return (&cur.dk).into();
            } else if let Some(next) = cur.children.get(key) {
                cur = next;
            } else {
                return PrefixMatch::Choices(&self.children);
            }
        }
        if cur.children.is_empty() {
            (&cur.dk).into()
        } else {
            PrefixMatch::Choices(&self.children)
        }
    }

    pub(crate) fn longest_prefix<'a, 'b>(&'a self, input: &'b [Key]) -> Mapping<'a, 'b> {
        trace!("finding longest_prefix of input {:?}", input);
        let mut choices: Mapping<'a, 'b> = Mapping::None;
        for i in (0..input.len()).rev() {
            trace!("longest_prefix loop {}", i);
            let prefix = &input[..=i];
            let prefix_match = self.match_prefix(prefix);
            match prefix_match {
                PrefixMatch::DK(dk) => {
                    return Mapping::Bound {
                        dk,
                        remaining: &input[i + 1..],
                    };
                }
                PrefixMatch::Choices(children) => {
                    if i == 0 {
                        /* user already typed all these keys, let's stash the possible next choices
                         * for them */
                        choices = Mapping::Choices(children);
                    }
                }
                PrefixMatch::None => continue,
            }
        }
        choices
    }
}

impl<'a> From<&Option<DK>> for PrefixMatch<'a> {
    fn from(dk: &Option<DK>) -> Self {
        match dk {
            Some(dk) => Self::DK(dk.clone()),
            None => Self::None,
        }
    }
}

#[derive(Debug)]
enum PrefixMatch<'a> {
    DK(DK),
    Choices(&'a HashMap<Key, TrieNode>),
    None,
}

#[derive(Debug)]
pub(crate) enum Mapping<'a, 'b> {
    Bound { dk: DK, remaining: &'b [Key] },
    Choices(&'a HashMap<Key, TrieNode>),
    None,
}
