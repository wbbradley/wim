use crate::command::CommandBuilder;
use crate::prelude::*;

#[derive(Default)]
pub struct Bindings {
    map: HashMap<Vec<Key>, DK>,
}

pub struct BindingsBuilder {
    bindings: Bindings,
    default_vk: ViewKey,
}

pub trait KeysLike {
    fn parse_keys(self) -> Vec<Key>;
}

impl BindingsBuilder {
    pub fn new(vk: ViewKey) -> Self {
        Self {
            bindings: Default::default(),
            default_vk: vk,
        }
    }
    pub fn insert(&mut self, keys_like: impl KeysLike, dk_like: impl IntoDKBinding) {
        let keys = keys_like.parse_keys();
        assert!(!self.bindings.map.contains_key(&keys));
        self.bindings
            .map
            .insert(keys, dk_like.to_dk_with_default_vk(self.default_vk));
    }
    pub fn get_bindings(self) -> Bindings {
        self.bindings
    }
}

impl Bindings {
    pub fn get_map(self) -> HashMap<Vec<Key>, DK> {
        self.map
    }
}

impl KeysLike for &str {
    fn parse_keys(self) -> Vec<Key> {
        self.chars().map(Key::Utf8).collect()
    }
}

impl KeysLike for Key {
    fn parse_keys(self) -> Vec<Key> {
        vec![self]
    }
}

trait IntoDKBinding {
    fn to_dk_with_default_vk(self, vk: ViewKey) -> DK;
}

impl IntoDKBinding for DK {
    fn to_dk_with_default_vk(self, _: ViewKey) -> DK {
        self
    }
}

impl IntoDKBinding for CommandBuilder {
    fn to_dk_with_default_vk(self, vk: ViewKey) -> DK {
        self.at_view(vk)
    }
}
