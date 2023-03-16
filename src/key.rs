use rune::Any;

#[derive(Any, Hash, Copy, Clone, Eq, PartialEq)]
pub enum Key {
    #[rune(constructor)]
    Esc,
    #[rune(constructor)]
    EscSeq1(#[rune(get)] u8),
    #[rune(constructor)]
    EscSeq2(#[rune(get)] u8, #[rune(get)] u8),
    #[rune(constructor)]
    Up,
    #[rune(constructor)]
    Down,
    #[rune(constructor)]
    Left,
    #[rune(constructor)]
    Right,
    #[rune(constructor)]
    Home,
    #[rune(constructor)]
    End,
    #[rune(constructor)]
    Del,
    #[rune(constructor)]
    PageUp,
    #[rune(constructor)]
    PageDown,
    #[rune(constructor)]
    PrintScreen,
    #[rune(constructor)]
    Backspace,
    #[rune(constructor)]
    Enter,
    #[rune(constructor)]
    Ctrl(#[rune(get)] char),
    #[rune(constructor)]
    Ascii(#[rune(get)] char),
    #[rune(constructor)]
    Function(#[rune(get)] u8),
    #[rune(constructor)]
    None,
}

impl std::fmt::Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Key::PageUp => write!(f, "<PageUp>"),
            Key::PageDown => write!(f, "<PageDown>"),
            Key::Home => write!(f, "<Home>"),
            Key::End => write!(f, "<End>"),
            Key::Del => write!(f, "<Del>"),
            Key::Up => write!(f, "<Up>"),
            Key::Down => write!(f, "<Down>"),
            Key::Left => write!(f, "<Left>"),
            Key::Right => write!(f, "<Right>"),
            Key::Ctrl(ch) => write!(f, "<C-{}>", ch),
            Key::Ascii(ch) => write!(f, "{}", ch),
            Key::Enter => write!(f, "<Enter>"),
            Key::Esc => write!(f, "<Esc>"),
            Key::EscSeq1(a) => write!(f, "<Esc-{}>", *a as char),
            Key::EscSeq2(a, b) => write!(f, "<Esc-{}-{}>", *a as char, *b as char),
            Key::Function(a) => write!(f, "<F{}>", *a),
            Key::PrintScreen => write!(f, "<PrintScreen>"),
            Key::Backspace => write!(f, "<Backspace>"),
            Key::None => write!(f, "<>"),
        }
    }
}

impl std::fmt::Debug for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}
