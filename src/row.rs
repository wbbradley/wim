use crate::buf::{Buf, ToCharVec};
use crate::classify::{classify, CharType};
use crate::consts::{BLANKS, TAB_STOP_SIZE};
use crate::types::{Coord, SafeCoordCast};
use std::ops::Range;

#[allow(dead_code)]
#[derive(Clone, Default, Debug)]
pub struct Row {
    buf: Vec<char>,
    render: Vec<char>,
}

impl Row {
    #[inline]
    pub fn append(&mut self, text: &str) {
        self.buf.append(text);
    }
    pub fn render_buf(&self) -> &Buf {
        &self.render
    }
    pub fn from_line(line: &str) -> Self {
        let buf = Buf::from_str(line);
        Self {
            render: Buf::render_from(&buf),
            buf,
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
        for (i, &ch) in self.buf.to_char_vec().iter().enumerate() {
            if i == cursor {
                break;
            }
            if ch == '\t' {
                render_x += (TAB_STOP_SIZE - 1) - render_x % TAB_STOP_SIZE;
            }
            render_x += 1;
        }
        render_x.as_coord()
    }

    pub fn char_at(&self, x: Coord) -> Option<char> {
        assert!(self.buf.len() >= x);
        if x == self.buf.len() {
            Some('\n')
        } else {
            self.buf.get(x).copied()
        }
    }
    pub fn insert_char(&mut self, x: Coord, ch: char) {
        self.buf.splice(x..x, [ch].into_iter());
        self.render = render_from_bytes(self.buf);
    }
    pub fn update_render(&mut self) {
        // Deal with rendering Tabs.
        let tabs = self.buf.into_iter().filter(|&x| x == '\t').count();
        if tabs == 0 {
            self.render.truncate(0);
            self.render.extend_from_slice(&self.buf);
        } else {
            self.render
                .reserve(self.buf.len() + tabs * (TAB_STOP_SIZE - 1));
            for ch in self.buf {
                if ch == '\t' {
                    self.render.extend_from_slice(&BLANKS[..TAB_STOP_SIZE]);
                } else {
                    self.render.push(ch);
                }
            }
        }
    }

    pub fn splice<T>(&mut self, range: Range<Coord>, text: T)
    where
        T: ToCharVec,
    {
        self.buf.splice(range, text.to_char_vec());
        self.render = Buf::render_from_bytes(self.buf.to_char_vec());
    }
    pub fn append_row(&mut self, row: &Self) {
        self.buf.append(row);
        self.render = Buf::render_from_bytes(self.buf.to_char_vec());
    }
    pub fn next_word_break(&self, x: Coord) -> Coord {
        if self.buf.len() <= x + 1 {
            self.buf.len()
        } else {
            let bytes = self.buf.to_char_vec();
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
            .to_char_vec()
            .iter()
            .position(|&b| classify(b as char) != CharType::Space)
    }
    pub fn prev_word_break(&self, mut x: Coord) -> Coord {
        x = x.clamp(0, self.buf.len());
        if x <= 1 {
            0
        } else {
            let bytes = self.buf.to_char_vec();
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

impl ToCharVec for &Row {
    fn to_char_vec(&self) -> Vec<char> {
        self.buf.to_char_vec()
    }
}
