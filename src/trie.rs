use crate::bindings::Bindings;
use crate::dispatch::Dispatcher;
use crate::prelude::*;

#[derive(Debug, Default)]
pub struct TrieNode {
    dk: Option<DK>,
    children: HashMap<Key, TrieNode>,
}

impl TrieNode {
    pub fn from_ancestor_path(ancestor_path: Vec<Target>, dispatcher: &dyn Dispatcher) -> Self {
        ancestor_path
            .iter()
            .cloned()
            .map(|target| dispatcher.resolve(target).get_key_bindings())
            .fold(Self::default(), |node, b| node.with_bindings(b))
    }
    fn with_bindings(self, bindings: Bindings) -> Self {
        for (keys, dk) in bindings {
            self.insert(dk, &keys);
        }
        self
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
            } else {
                match cur.children.get(key) {
                    Some(next) => {
                        cur = next;
                    }
                    None => {
                        return PrefixMatch::None;
                    }
                }
            }
        }
        if cur.children.is_empty() {
            (&cur.dk).into()
        } else {
            PrefixMatch::Choices(&cur.children)
        }
    }

    pub(crate) fn longest_prefix<'a, 'b>(&'a self, input: &'b [Key]) -> Mapping<'a, 'b> {
        trace!("finding longest_prefix of input {:?}", input);
        let mut choices: Mapping<'a, 'b> = Mapping::None;
        for i in (0..input.len()).rev() {
            let prefix = &input[..=i];
            let prefix_match = self.match_prefix(prefix);
            trace!(
                "longest_prefix loop [i={},prefix={:?},prefix_match={:?}]",
                i,
                prefix,
                prefix_match
            );
            match prefix_match {
                PrefixMatch::DK(dk) => {
                    return Mapping::Bound {
                        dk,
                        remaining: &input[i + 1..],
                    };
                }
                PrefixMatch::Choices(children) => {
                    if i == input.len() - 1 {
                        /* user already typed all these keys, let's stash the possible next choices
                         * for them */
                        choices = Mapping::Choices(children);
                    }
                }
                PrefixMatch::None => continue,
            }
        }
        trace!(
            "longest_prefix found no prefix match: returning {:?}",
            choices
        );
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
