use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum Message {
    SendKey(Key),
    Command { name: String, args: Vec<Variant> },
}

impl crate::dk::ToDK for Message {
    fn vk(self, view_key: ViewKey) -> DK {
        DK::Dispatch(Target::View(view_key), self)
    }
}
