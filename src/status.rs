use std::time::Instant;

#[derive(Debug)]
pub enum Status {
    Message { message: String, expiry: Instant },
    Ok,
}

impl From<()> for Status {
    fn from(_: ()) -> Self {
        Status::Ok
    }
}

macro_rules! status {
    ($($args:expr),+) => {{
        Status::Message{message:format!($($args),+),expiry:Instant::now()+Duration::from_secs(2)}
    }};
}
pub(crate) use status;
