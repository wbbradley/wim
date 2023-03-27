#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CharType {
    Text,
    Punct,
    Space,
}

pub fn classify(ch: char) -> CharType {
    if ch.is_ascii_punctuation() {
        CharType::Punct
    } else if ch.is_whitespace() {
        CharType::Space
    } else {
        CharType::Text
    }
}
