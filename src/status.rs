use std::time::Instant;

#[derive(Debug)]
pub enum Status {
    Message { message: String, expiry: Instant },
    None,
}
