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
    fn get_property(&self, _property: &str) -> Option<Variant>;
    fn get_property_bool(&self, property: &str, default: bool) -> bool {
        match self.get_property(property) {
            Some(Variant::Bool(b)) => b,
            _ => default,
        }
    }
    fn get_property_string(&self, property: &str) -> Option<String> {
        match self.get_property(property) {
            Some(Variant::String(b)) => Some(b),
            _ => None,
        }
    }
    #[allow(dead_code)]
    fn get_property_pos(&self, property: &str) -> Option<Pos> {
        match self.get_property(property) {
            Some(Variant::Pos(b)) => Some(b),
            _ => None,
        }
    }
}

pub trait View: DispatchTarget + ViewContext {
    #[allow(dead_code)]
    fn install_plugins(&mut self, plugin: PluginRef);
    /// layout returns a vec of views that also need layout.
    #[must_use]
    fn layout(&mut self, view_map: &ViewMap, size: Size) -> Vec<(ViewKey, Rect)>;
    fn display(&self, view_map: &ViewMap, bmp: &mut BitmapView);
    fn get_view_key(&self) -> ViewKey;
    fn get_cursor_pos(&self) -> Option<Pos>;
    fn set_status(&mut self, status: Status) {
        log::warn!(
            "View '{}' is ignoring set_status calls. [status={:?}]",
            std::any::type_name::<Self>(),
            status
        );
    }
    #[allow(dead_code)]
    fn get_doc_text(&self, _view_map: &ViewMap) -> Option<String>;
}
