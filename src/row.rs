use crate::buf::{Buf, ToBufBytes, TAB_STOP_SIZE};
use crate::types::{Coord, SafeCoordCast};

#[allow(dead_code)]
pub struct Row {
    buf: Buf,
    render: Buf,
}

#[allow(dead_code)]
impl Row {
    #[inline]
    pub fn append(&mut self, text: &str) {
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

    pub fn insert(&mut self, x: Coord, ch: char) {
        self.buf.splice(x..x, std::slice::from_ref(&(ch as u8)));
        self.render = Buf::render_from_bytes(self.buf.to_bytes());
    }
}

impl ToBufBytes for &Row {
    fn to_bytes(&self) -> &[u8] {
        self.buf.to_bytes()
    }
}
