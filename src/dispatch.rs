use crate::bindings::Bindings;
use crate::error::{Error, Result};
use crate::prelude::*;

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

pub enum DispatchRef<'a> {
    ViewMap(&'a mut ViewMap),
    ViewRef(ViewRef),
}

impl<'a> From<&'a mut ViewMap> for DispatchRef<'a> {
    fn from(vm: &'a mut ViewMap) -> Self {
        Self::ViewMap(vm)
    }
}

impl<'a> From<ViewRef> for DispatchRef<'a> {
    fn from(vr: ViewRef) -> Self {
        Self::ViewRef(vr)
    }
}

impl<'a> DispatchTarget for DispatchRef<'a> {
    fn get_key_bindings(&self) -> Bindings {
        match self {
            Self::ViewMap(view_map) => view_map.get_key_bindings(),
            Self::ViewRef(view_ref) => view_ref.get_key_bindings(),
        }
    }
    fn execute_command(&mut self, name: String, args: Vec<Variant>) -> Result<Status> {
        match self {
            Self::ViewMap(view_map) => view_map.execute_command(name, args),
            Self::ViewRef(view_ref) => view_ref.execute_command(name, args),
        }
    }
    fn send_key(&mut self, key: Key) -> Result<Status> {
        match self {
            Self::ViewMap(view_map) => view_map.send_key(key),
            Self::ViewRef(view_ref) => view_ref.send_key(key),
        }
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
    fn resolve(&mut self, target: Target) -> DispatchRef;
}
