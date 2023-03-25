#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CharType {
    Text,
    Punct,
    Space,
}

pub fn classify(ch: char) -> CharType {
    if unsafe { libc::ispunct(ch as libc::c_int) } != 0 {
        CharType::Punct
    } else if unsafe { libc::isspace(ch as libc::c_int) } != 0 {
        CharType::Space
    } else {
        CharType::Text
    }
}
