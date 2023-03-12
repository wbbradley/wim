use std::time::Instant;

#[derive(Debug)]
pub enum Status {
    Message { message: String, expiry: Instant },
    NothingToSay,
    Cleared,
}

impl From<()> for Status {
    fn from(_: ()) -> Self {
        Status::NothingToSay
    }
}
