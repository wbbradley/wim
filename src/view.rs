use crate::bindings::Bindings;
use crate::error::{Error, Result};
use crate::plugin::PluginRef;
use crate::prelude::*;
use crate::propvalue::PropertyValue;
use crate::status::Status;
use crate::types::{Pos, Rect};

#[derive(Any, Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct ViewKey(usize);

impl From<usize> for ViewKey {
    fn from(u: usize) -> Self {
        Self(u)
    }
}

pub type ViewRef = Rc<RefCell<dyn View>>;

pub trait ViewContext {
    fn get_property(&self, _property: &str) -> Option<PropertyValue> {
        None
    }
    fn get_property_bool(&self, property: &str, default: bool) -> bool {
        match self.get_property(property) {
            Some(PropertyValue::Bool(b)) => b,
            _ => default,
        }
    }
    fn get_property_string(&self, property: &str, default: &str) -> String {
        match self.get_property(property) {
            Some(PropertyValue::String(b)) => b,
            _ => default.to_string(),
        }
    }
    fn get_property_pos(&self, property: &str) -> Option<Pos> {
        match self.get_property(property) {
            Some(PropertyValue::Pos(b)) => Some(b),
            _ => None,
        }
    }
}

pub trait View: ViewContext {
    fn get_parent(&self) -> Option<Weak<RefCell<dyn View>>>;
    fn install_plugins(&mut self, plugin: PluginRef);
    fn layout(&mut self, frame: Rect);
    fn display(&self, buf: &mut Buf, context: &dyn ViewContext);
    fn get_view_key(&self) -> ViewKey;
    fn get_cursor_pos(&self) -> Option<Pos>;
    fn execute_command(&mut self, name: String, args: Vec<CallArg>) -> Result<Status> {
        Err(Error::not_impl(format!(
            "{}::execute_command does not yet exist. Needs to handle {:?} {:?}.",
            std::any::type_name::<Self>(),
            name,
            args,
        )))
    }
    fn send_key(&mut self, key: Key) -> Result<Status>;
    fn get_view_mode(&self) -> Mode;
    fn get_key_bindings(&self, root_view_key: ViewKey) -> Bindings;
}

impl dyn View {
    pub fn ancestor_path(&self, path: &mut Vec<ViewKey>) {
        path.push(self.get_view_key());
        if let Some(next) = self.get_parent() {
            if let Some(parent) = next.upgrade() {
                parent.borrow().ancestor_path(path);
            }
        }
    }
}

pub fn to_view<T>(v: &Rc<RefCell<T>>) -> Rc<RefCell<dyn View>>
where
    T: View + 'static,
{
    v.clone() as Rc<RefCell<dyn View>>
}

pub fn to_weak_view(v: ViewRef) -> Weak<RefCell<dyn View>> {
    let v = v as Rc<RefCell<dyn View>>;
    Rc::downgrade(&v)
}
