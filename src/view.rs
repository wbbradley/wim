use crate::plugin::PluginRef;
use crate::prelude::*;
use crate::types::{Pos, Rect};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ViewKey(usize);

impl From<usize> for ViewKey {
    fn from(u: usize) -> Self {
        Self(u)
    }
}

pub trait ViewContext {
    fn get_property(&self, _property: &str) -> Option<Variant> {
        None
    }
    fn get_property_bool(&self, property: &str, default: bool) -> bool {
        match self.get_property(property) {
            Some(Variant::Bool(b)) => b,
            _ => default,
        }
    }
    fn get_property_string(&self, property: &str, default: &str) -> String {
        match self.get_property(property) {
            Some(Variant::String(b)) => b,
            _ => default.to_string(),
        }
    }
    fn get_property_pos(&self, property: &str) -> Option<Pos> {
        match self.get_property(property) {
            Some(Variant::Pos(b)) => Some(b),
            _ => None,
        }
    }
}

pub trait View {
    fn get_parent(&self) -> Option<ViewKey>;
    fn install_plugins(&mut self, plugin: PluginRef);
    fn layout(&mut self, view_map: &ViewMap, frame: Rect);
    fn display(&self, view_map: &ViewMap, buf: &mut Buf, context: &dyn ViewContext);
    fn get_view_key(&self) -> ViewKey;
    fn get_cursor_pos(&self) -> Option<Pos>;
    fn get_view_mode(&self) -> Mode;
}
