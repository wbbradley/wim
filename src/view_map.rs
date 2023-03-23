use crate::error::Result;
use crate::keygen::ViewKeyGenerator;
use crate::prelude::*;
use crate::trie::{Mapping, TrieNode};

pub struct ViewMap {
    map: HashMap<ViewKey, ViewRef>,
    named_views: HashMap<String, ViewKey>,
    previous_views: Vec<ViewKey>,
    view_key_gen: ViewKeyGenerator,
    root_view_key: Option<ViewKey>,
}

impl ViewMap {
    pub fn new() -> Self {
        Self {
            map: Default::default(),
            named_views: Default::default(),
            previous_views: Default::default(),
            view_key_gen: ViewKeyGenerator::new(),
            root_view_key: None,
        }
    }
    pub fn set_focused_view(&mut self, view_key_to_focus: ViewKey) {
        assert!(self.map.contains_key(&view_key_to_focus));
        log::trace!("focusing view '{:?}'", view_key_to_focus);
        self.previous_views.retain(|vk| {
            // Keep the views that still exist and that aren't the intended one so we can move it
            // to the top of the stack..
            *vk != view_key_to_focus
        });

        self.previous_views.push(view_key_to_focus);
    }
    pub fn set_root_view_key(&mut self, vk: ViewKey) {
        self.root_view_key = Some(vk)
    }
    pub fn get_next_key(&mut self) -> ViewKey {
        self.view_key_gen.next_key()
    }
    pub fn get_root_view(&self) -> ViewRef {
        self.get_view(self.root_view_key.unwrap())
    }
    pub fn get_root_view_key(&self) -> ViewKey {
        self.root_view_key.unwrap()
    }
    pub fn insert(&mut self, vk: ViewKey, view: ViewRef, name: Option<String>) {
        self.map.insert(vk, view);
        if let Some(name) = name {
            assert!(!self.named_views.contains_key(&name));
            self.named_views.insert(name, vk);
        }
    }
    pub fn get_named_view(&self, name: &str) -> Option<ViewRef> {
        self.named_views.get(name).map(|&vk| self.get_view(vk))
    }
    pub fn get_view(&self, vk: ViewKey) -> ViewRef {
        match self.map.get(&vk) {
            Some(view) => view.clone(),
            None => panic!("oh no, no view!"),
        }
    }
    pub fn focused_view_key(&self) -> ViewKey {
        assert!(!self.previous_views.is_empty());
        *self.previous_views.last().unwrap()
    }
    pub fn focused_view(&self) -> ViewRef {
        self.get_view(self.focused_view_key())
    }

    /*
    pub fn get_view_or_focused_view(&self, view_key: Option<ViewKey>) -> ViewRef {
        match view_key {
            Some(view_key) => self.get_view(view_key),
            None => self.focused_view(),
        }
    }
    */
    fn ancestor_path(&self, view_key: ViewKey) -> Vec<Target> {
        let mut path: Vec<Target> = Default::default();
        let mut view = self.get_view(view_key);
        loop {
            path.push(Target::View(view.get_view_key()));
            match view.get_parent().map(|vk| self.get_view(vk)) {
                Some(parent) => {
                    view = parent;
                }
                None => break,
            }
        }
        path
    }
    pub(crate) fn handle_keys(&mut self, dks: &mut VecDeque<DK>) -> HandleKey {
        let path: Vec<Target> = self.ancestor_path(self.focused_view_key());
        let trie: TrieNode = TrieNode::from_ancestor_path(path, self);
        let inbound_keys: Vec<Key> = dks
            .iter()
            .take_while(|dk| matches!(dk, DK::Key(_)))
            .map(|dk| match dk {
                DK::Key(key) => *key,
                _ => {
                    panic!("foogoo");
                }
            })
            .collect();
        if inbound_keys.is_empty() {
            return HandleKey::DK(dks.pop_front().unwrap());
        }
        trace!("inbound_keys of dks === {:?} of {:?}", inbound_keys, dks);
        assert!(!inbound_keys.is_empty());
        match trie.longest_prefix(&inbound_keys) {
            Mapping::Bound { dk, remaining } => {
                trace!(
                    "keys {:?} translated into dk={:?}, leaving remaining={:?}",
                    inbound_keys,
                    dk,
                    remaining
                );
                (0..(inbound_keys.len() - remaining.len())).for_each(|_| {
                    dks.pop_front();
                });
                HandleKey::DK(dk)
            }
            Mapping::Choices(choices) => {
                trace!("found choices {:?}", choices);
                assert!(!choices.is_empty());
                HandleKey::Choices(choices.iter().map(|(key, _)| key).cloned().collect())
            }
            Mapping::None => {
                trace!("no mapping found, returning SendKey({:?})", inbound_keys[0]);
                dks.pop_front();
                HandleKey::DK(DK::Dispatch(
                    Target::View(self.focused_view_key()),
                    Message::SendKey(inbound_keys[0]),
                ))
            }
        }
    }

    pub fn goto_previous_view(&mut self) {
        self.previous_views.pop();
    }
}

impl DispatchTarget for ViewMap {
    fn execute_command(&mut self, name: String, _args: Vec<Variant>) -> Result<Status> {
        if name == "focus-previous" {
            self.goto_previous_view();
            Ok(Status::Cleared)
        } else {
            panic!("JKDFJKDJFK")
        }
    }
}

impl Dispatcher for ViewMap {
    fn resolve(&mut self, target: Target) -> DispatchRef {
        match target {
            Target::ViewMap => self.into(),
            Target::Focused => self.focused_view().into(),
            Target::View(vk) => self.get_view(vk).into(),
            Target::Root => self.get_view(self.root_view_key.unwrap()).into(),
        }
    }
}

#[derive(Debug)]
pub(crate) enum HandleKey {
    DK(DK),
    Choices(Vec<Key>),
}

impl ViewContext for ViewMap {
    fn get_property(&self, property: &str) -> Option<Variant> {
        panic!("is editor asked for {:?}", property);
        // self.focused_view().get_property(property)
    }
}
