use crate::classify::classify;
use crate::consts::{BLANKS, TAB_STOP_SIZE};
use crate::types::{Coord, SafeCoordCast};
use std::ops::{Range, RangeBounds};

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
    pub fn from_buf(buf: Vec<char>) -> Self {
        Self {
            render: Self::renderize(&buf),
            buf,
        }
    }
    pub fn from_line(line: &str) -> Self {
        let buf: Vec<char> = line.chars().collect();
        Self {
            render: Self::renderize(&buf),
            buf,
        }
    }
    pub fn joined_rows(
        first: &Self,
        second: &Self,
        first_index: Coord,
        second_index: Coord,
    ) -> Self {
        let mut buf = first.buf[..first_index].to_vec();
        buf.extend(&second.buf[second_index..]);
        Self {
            render: Self::renderize(&buf),
            buf,
        }
    }
    pub fn from_chars(chs: &[char]) -> Self {
        let mut slf = Self {
            buf: chs.to_vec(),
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
    #[allow(dead_code)]
    pub fn truncate(&self, len: usize) -> Self {
        Self::from_chars(&self.buf[0..len])
    }
    pub fn split_at(&self, x: usize) -> [Self; 2] {
        [
            Self::from_chars(&self.buf[..x]),
            Self::from_chars(&self.buf[x..]),
        ]
    }
    // pub fn col_len(&self) -> usize {
    //     self.render.len()
    // }

    /// Adjust the render column to account for tabs.
    pub fn cursor_to_render_col(&self, cursor: Coord) -> Coord {
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

    pub fn insert_char(&self, x: Coord, ch: char) -> Self {
        let mut buf = self.buf.clone();
        buf.splice(x..x, [ch]);
        Self::from_buf(buf)
    }

    pub fn get_slice(&self, range: Range<usize>) -> &[char] {
        &self.buf[range.start..range.end.clamp(range.start, self.buf.len())]
    }

    pub fn as_slice(&self) -> &[char] {
        &self.buf
    }

    pub fn update_render(&mut self) {
        Self::renderize_in_place(&self.buf, &mut self.render);
    }
    fn renderize(buf: &[char]) -> Vec<char> {
        let mut render = Default::default();
        Self::renderize_in_place(buf, &mut render);
        render
    }
    fn renderize_in_place(buf: &[char], render: &mut Vec<char>) {
        // Deal with rendering Tabs.
        let tabs = buf.iter().copied().filter(|&x| x == '\t').count();
        if tabs == 0 {
            render.truncate(0);
            render.extend_from_slice(buf);
        } else {
            render.reserve(buf.len() + tabs * (TAB_STOP_SIZE - 1));
            for ch in buf.iter().copied() {
                if ch == '\t' {
                    render.extend_from_slice(&BLANKS[..TAB_STOP_SIZE]);
                } else {
                    render.push(ch);
                }
            }
        }
    }

    pub fn splice(&self, range: impl RangeBounds<Coord>, text: &str) -> Self {
        let mut buf = self.buf.clone();
        buf.splice(range, text.chars());
        Self::from_buf(buf)
    }

    /*
    pub fn append_str(&mut self, s: &str) {
        self.buf.extend(s.chars());
        self.update_render();
    }

    pub fn append_row(&mut self, row: Self) {
        self.buf.extend(&row.buf);
        self.update_render();
    }
    */

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

    #[allow(dead_code)]
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
