use crate::buf::Buf;
use crate::command::Command;
use crate::dk::DK;
use crate::error::{Error, Result};
use crate::keygen::KeyGenerator;
use crate::read::Key;
use crate::status::Status;
use crate::types::{Pos, Rect};

pub type ViewKey = String;
pub type ViewKeyGenerator = KeyGenerator;

#[allow(dead_code)]
pub enum PropertyValue {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Pos(Pos),
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

pub trait View: ViewContext {
    fn layout(&mut self, frame: Rect);
    fn display(&self, buf: &mut Buf, context: &dyn ViewContext);
    fn get_cursor_pos(&self) -> Option<Pos>;
    fn execute_command(&mut self, command: Command) -> Result<Status> {
        Err(Error::not_impl(format!(
            "{}::execute_command does not yet exist. Needs to handle {:?}.",
            std::any::type_name::<Self>(),
            command
        )))
    }
    fn dispatch_key(&mut self, key: Key) -> Result<DK> {
        Err(Error::new(format!(
            "{} does not (yet?) handle dispatch_key [key={:?}]",
            std::any::type_name::<Self>(),
            key
        )))
    }
}
