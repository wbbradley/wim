use crate::prelude::*;

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
    args: Vec<Variant>,
}

impl CommandBuilder {
    pub fn arg<T>(mut self, t: T) -> Self
    where
        T: Into<Variant>,
    {
        self.args.push(t.into());
        self
    }
    pub fn at_target(self, target: Target) -> DK {
        DK::Dispatch(
            target,
            Message::Command {
                name: self.name,
                args: self.args,
            },
        )
    }
    pub fn at_view(self, view_key: ViewKey) -> DK {
        self.at_target(Target::View(view_key))
    }
    pub fn at_root(self) -> DK {
        self.at_target(Target::Root)
    }
    pub fn at_view_map(self) -> DK {
        self.at_target(Target::ViewMap)
    }
    pub fn at_focused(self) -> DK {
        self.at_target(Target::Focused)
    }
}
