use crate::dk::{CallArg, DK};
use crate::prelude::*;
use crate::view::ViewKey;

pub fn command<T>(name: T) -> CommandBuilder
where
    T: Into<String>,
{
    CommandBuilder {
        name: name.into(),
        args: Default::default(),
    }
}

pub struct CommandBuilder {
    name: String,
    args: Vec<CallArg>,
}

impl CommandBuilder {
    pub fn arg<T>(mut self, t: T) -> Self
    where
        T: Into<CallArg>,
    {
        self.args.push(t.into());
        self
    }
    pub fn vk(self, view_key: ViewKey) -> DK {
        DK::Dispatch(
            Some(view_key),
            Message::Command {
                name: self.name,
                args: self.args,
            },
        )
    }
    pub fn no_vk(self) -> DK {
        DK::Dispatch(
            None,
            Message::Command {
                name: self.name,
                args: self.args,
            },
        )
    }
}
