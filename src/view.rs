use crate::buf::Buf;
use crate::command::Command;
use crate::error::{Error, Result};
use crate::read::Key;
use crate::status::Status;
use crate::types::{Pos, Rect};

pub trait View {
    fn layout(&mut self, frame: Rect);
    fn display(&self, buf: &mut Buf);
    fn get_cursor_pos(&self) -> Option<Pos>;
    fn execute_command(&mut self, command: Command) -> Result<Status> {
        Err(Error::not_impl(format!(
            "{} does not yet implement {:?}",
            std::any::type_name::<Self>(),
            command
        )))
    }
    fn dispatch_key(&mut self, key: Key) -> DK {
        DK::Err(Error::new(format!(
            "{} does not (yet?) handle dispatch_key [key={:?}]",
            std::any::type_name::<Self>(),
            key
        )))
    }
}

#[allow(dead_code)]
pub enum DK {
    Mapping(Vec<Key>),
    Err(Error),
    CloseView,
}
