use crate::bindings::Bindings;
use crate::error::{Error, Result};
use crate::prelude::*;

pub trait DispatchClient {
    fn get_key_bindings(&self) -> Bindings;
    fn execute_command(&self, name: String, args: Vec<Variant>) -> Result<Status>;
    fn send_key(&self, key: Key) -> Result<Status>;
}

pub trait DispatchTarget {
    fn get_key_bindings(&self) -> Bindings {
        Default::default()
    }
    fn execute_command(&mut self, name: String, args: Vec<Variant>) -> Result<Status> {
        Err(Error::not_impl(format!(
            "{}::execute_command does not yet exist. Needs to handle {:?} {:?}.",
            std::any::type_name::<Self>(),
            name,
            args,
        )))
    }
    fn send_key(&mut self, key: Key) -> Result<Status> {
        Err(Error::not_impl(format!(
            "{}::send_key does not yet exist. Needs to handle '{:?}'",
            std::any::type_name::<Self>(),
            key,
        )))
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Target {
    View(ViewKey),
    Focused,
    ViewMap,
    Root, // Should be the editor.
}

pub trait Dispatcher {
    fn resolve_mut(&mut self, target: Target) -> &mut dyn DispatchTarget;
}
