use crate::classify::classify;
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
    // #[inline]
    // pub fn append(&mut self, text: &str) {
    //     self.buf.extend(text.chars());
    // }
    // pub fn render_buf(&self) -> &[char] {
    //     &self.render
    // }
    pub fn from_line(line: &str) -> Self {
        let mut slf = Self {
            buf: line.chars().collect(),
            render: Vec::new(),
        };
        slf.update_render();
        slf
    }
    pub fn len(&self) -> usize {
        self.buf.len()
    }
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }
    // pub fn col_len(&self) -> usize {
    //     self.render.len()
    // }

    /// Adjust the render column to account for tabs.
    pub fn cursor_to_render_col(&self, cursor: Coord) -> Coord {
        let cursor = cursor;
        let mut render_x: usize = 0;
        for (i, &ch) in self.buf.iter().enumerate() {
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
        self.update_render();
    }
    pub fn update_render(&mut self) {
        // Deal with rendering Tabs.
        let tabs = self.buf.iter().copied().filter(|&x| x == '\t').count();
        if tabs == 0 {
            self.render.truncate(0);
            self.render.extend_from_slice(&self.buf);
        } else {
            self.render
                .reserve(self.buf.len() + tabs * (TAB_STOP_SIZE - 1));
            for ch in self.buf.iter().copied() {
                if ch == '\t' {
                    self.render.extend_from_slice(&BLANKS[..TAB_STOP_SIZE]);
                } else {
                    self.render.push(ch);
                }
            }
        }
    }

    pub fn splice(&mut self, range: Range<Coord>, text: &str) {
        self.buf.splice(range, text.chars());
        self.update_render();
    }

    pub fn append_str(&mut self, s: &str) {
        self.buf.extend(s.chars());
        self.update_render();
    }
    pub fn append_row(&mut self, row: Self) {
        self.buf.extend(&row.buf);
        self.update_render();
    }
    pub fn next_word_break(&self, x: Coord) -> Coord {
        if self.buf.len() <= x + 1 {
            self.buf.len()
        } else {
            let start_class = classify(self.buf[x]);
            for (i, &ch) in self.buf[x + 1..].iter().enumerate() {
                let next_class = classify(ch);
                if next_class != start_class {
                    return x + i + 1;
                }
            }
            self.buf.len()
        }
    }

    pub fn prev_word_break(&self, mut x: Coord) -> Coord {
        x = x.clamp(0, self.buf.len());
        if x <= 1 {
            0
        } else {
            let start_class = classify(self.buf[x - 1]);
            for i in 1..=x {
                let ch = self.buf[x - i];
                let next_class = classify(ch);
                if next_class != start_class {
                    return x - i + 1;
                }
            }
            0
        }
    }
    // pub fn chars_from(&self, mut x: Coord) -> std::slice::Iter<'_, char> {
    //     x = x.clamp(0, self.buf.len());
    //     self.buf[x..].iter()
    // }
    pub fn render_chars_from(&self, mut x: Coord) -> std::slice::Iter<'_, char> {
        x = x.clamp(0, self.render.len());
        self.render[x..].iter()
    }
}
