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

pub enum View {
    Editor(crate::editor::Editor),
    DocView(crate::docview::DocView),
    VStack(crate::vstack::VStack),
    CommandLine(crate::commandline::CommandLine),
}

macro_rules! forward {
    ($self:expr, $($tt:tt)+) => {
        match $self {
            View::Editor(editor) => editor.$($tt)+,
            View::DocView(docview) => docview.$($tt)+,
            View::VStack(vstack) => vstack.$($tt)+,
            View::CommandLine(cmdline) => cmdline.$($tt)+,
        }
    };
}

impl ViewContext for View {
    fn get_property(&self, property: &str) -> Option<PropertyValue> {
        forward!(self, get_property(property))
    }
}

impl ViewImpl for View {
    fn get_parent(&self) -> Option<ViewKey> {
        forward!(self, get_parent())
    }
    fn install_plugins(&mut self, plugin: PluginRef) {
        forward!(self, install_plugins(plugin))
    }
    fn layout(&mut self, frame: Rect) {
        forward!(self, layout(frame))
    }
    fn display(&self, buf: &mut Buf, context: &dyn ViewContext) {
        forward!(self, display(buf, context))
    }
    fn get_view_key(&self) -> ViewKey {
        forward!(self, get_view_key())
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        forward!(self, get_cursor_pos())
    }
    fn execute_command(&mut self, name: String, args: Vec<CallArg>) -> Result<Status> {
        forward!(self, execute_command(name, args))
    }
    fn send_key(&mut self, key: Key) -> Result<Status> {
        forward!(self, send_key(key))
    }
    fn get_view_mode(&self) -> Mode {
        forward!(self, get_view_mode())
    }
    fn get_key_bindings(&self, root_view_key: ViewKey) -> Bindings {
        forward!(self, get_key_bindings(root_view_key))
    }
}

pub trait ViewImpl {
    fn get_parent(&self) -> Option<ViewKey>;
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

pub fn ancestor_path(view_mapper: &dyn ViewMapper, view_key: ViewKey) -> Vec<ViewKey> {
    let mut path: Vec<ViewKey> = Default::default();
    let mut view = view_mapper.get_view(view_key);
    loop {
        path.push(view.get_view_key());
        match view.get_parent().map(|vk| view_mapper.get_view(vk)) {
            Some(parent) => {
                view = parent;
            }
            None => break,
        }
    }
    path
}

pub trait ViewMapper {
    fn get_view(&self, vk: ViewKey) -> &View;
    fn get_view_mut(&mut self, vk: ViewKey) -> &mut View;
}
