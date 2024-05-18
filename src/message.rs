use crate::prelude::*;

#[derive(Clone, Debug)]
pub enum Message {
    SendKey(Key),
    Command { name: String, args: Vec<Variant> },
}
