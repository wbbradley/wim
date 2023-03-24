use crate::buf::{Buf, ToBufBytes, TAB_STOP_SIZE};
use crate::types::{Coord, SafeCoordCast};
use std::ops::Range;

#[allow(dead_code)]
#[derive(Clone, Default, Debug)]
pub struct Row {
    buf: Buf,
    render: Buf,
}

#[allow(dead_code)]
impl Row {
    #[inline]
    pub fn append(&mut self, text: &str) {
        panic!("not used ({})", text);
        // self.buf.append(text)
    }
    pub fn render_buf(&self) -> &Buf {
        &self.render
    }
    pub fn from_line(line: &str) -> Self {
        Self {
            buf: Buf::from_bytes(line),
            render: Buf::render_from_bytes(line),
        }
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }
    pub fn col_len(&self) -> usize {
        self.render.len()
    }

    /// Adjust the render column to account for tabs.
    pub fn cursor_to_render_col(&self, cursor: Coord) -> Coord {
        let cursor = cursor;
        let mut render_x: usize = 0;
        for (i, &ch) in self.buf.to_bytes().iter().enumerate() {
            if i == cursor {
                break;
            }
            if ch == b'\t' {
                render_x += (TAB_STOP_SIZE - 1) - render_x % TAB_STOP_SIZE;
            }
            render_x += 1;
        }
        render_x.as_coord()
    }

    pub fn char_at(&self, x: Coord) -> Option<char> {
        match std::str::from_utf8(self.buf.to_bytes()) {
            Ok(s) => s.chars().nth(x),
            Err(_) => None,
        }
    }
    pub fn insert_char(&mut self, x: Coord, ch: char) {
        let mut tmp = [0u8; 4];
        let ch_text = ch.encode_utf8(&mut tmp);
        self.buf.splice(x..x, ch_text.as_bytes());
        self.render = Buf::render_from_bytes(self.buf.to_bytes());
    }

    pub fn splice<T>(&mut self, range: Range<Coord>, text: T)
    where
        T: ToBufBytes,
    {
        self.buf.splice(range, text.to_bytes());
        self.render = Buf::render_from_bytes(self.buf.to_bytes());
    }
    pub fn append_row(&mut self, row: &Self) {
        self.buf.append(row);
        self.render = Buf::render_from_bytes(self.buf.to_bytes());
    }
    pub fn next_word_break(&self, x: Coord) -> Coord {
        if self.buf.len() <= x + 1 {
            self.buf.len()
        } else {
            let bytes = self.buf.to_bytes();
            let start_class = classify(bytes[x] as char);
            for (i, &ch) in bytes[x + 1..].iter().enumerate() {
                let next_class = classify(ch as char);
                if next_class != start_class {
                    return x + i + 1;
                }
            }
            self.buf.len()
        }
    }

    pub fn get_first_word(&self) -> Option<Coord> {
        self.buf
            .to_bytes()
            .iter()
            .position(|&b| classify(b as char) != CharType::Space)
    }
    pub fn next_word_start(&self, x: Coord) -> Option<Coord> {
        if self.buf.len() <= x + 1 {
            None
        } else {
            let bytes = self.buf.to_bytes();
            let mut last_class = classify(bytes[x] as char);
            bytes[x + 1..]
                .iter()
                .map(|&b| b as char)
                .position(|ch| {
                    let new_class = classify(ch);
                    if new_class != last_class && new_class != CharType::Space {
                        true
                    } else {
                        last_class = new_class;
                        false
                    }
                })
                .map(|index| x + 1 + index)
        }
    }
    pub fn prev_word_break(&self, mut x: Coord) -> Coord {
        x = x.clamp(0, self.buf.len());
        if x <= 1 {
            0
        } else {
            let bytes = self.buf.to_bytes();
            let start_class = classify(bytes[x - 1] as char);
            for i in 1..=x {
                let ch = bytes[x - i];
                let next_class = classify(ch as char);
                if next_class != start_class {
                    return x - i + 1;
                }
            }
            0
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub enum CharType {
    Text,
    Punct,
    Space,
}

fn classify(ch: char) -> CharType {
    if unsafe { libc::ispunct(ch as libc::c_int) } != 0 {
        CharType::Punct
    } else if unsafe { libc::isspace(ch as libc::c_int) } != 0 {
        CharType::Space
    } else {
        CharType::Text
    }
}

impl ToBufBytes for &Row {
    fn to_bytes(&self) -> &[u8] {
        self.buf.to_bytes()
    }
}
