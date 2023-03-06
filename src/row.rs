use crate::buf::{Buf, ToBufBytes, TAB_STOP_SIZE};
use crate::types::{Coord, SafeCoordCast};
use std::ops::Range;

#[allow(dead_code)]
#[derive(Clone, Default)]
pub struct Row {
    buf: Buf,
    render: Buf,
}

#[allow(dead_code)]
impl Row {
    #[inline]
    pub fn append(&mut self, text: &str) {
        assert!(false);
        self.buf.append(text)
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
        let cursor = cursor as usize;
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
}

impl ToBufBytes for &Row {
    fn to_bytes(&self) -> &[u8] {
        self.buf.to_bytes()
    }
}
