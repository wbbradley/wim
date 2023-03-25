use crate::types::{Coord, Pos};

#[derive(Clone, Debug)]
pub struct Buf {
    b: Vec<char>,
}

impl Default for Buf {
    fn default() -> Self {
        let mut b = Vec::new();
        b.reserve(2 << 16);
        Self { b }
    }
}

pub trait ToCharVec {
    fn to_char_vec(&self) -> Vec<char>;
}

impl ToCharVec for &str {
    fn to_char_vec(&self) -> Vec<char> {
        self.chars().collect()
    }
}

impl ToCharVec for &String {
    fn to_char_vec(&self) -> Vec<char> {
        self.chars().collect()
    }
}

impl Buf {
    pub fn truncate(&mut self) {
        self.b.truncate(0);
    }
    pub fn reserve(&mut self, size: usize) {
        self.b.reserve(size);
    }
    pub fn append<T>(&mut self, text: &str) {
        self.b.extend(text.chars())
    }

    /*
    pub fn splice<T>(&mut self, range: std::ops::Range<Coord>, text: &str)
    where
        T: ToCharVec,
    {
        let bytes = text.to_char_vec();
        self.b.splice(range, bytes.iter().copied());
    }
    */

    pub fn insert_char(&mut self, x: Coord, ch: char) {
        self.b.splice(x..x, [ch].into_iter());
    }

    /*
    pub fn write_to(&self, fd: libc::c_int) {
        let ret = unsafe { libc::write(fd, self.b.as_ptr() as *const libc::c_void, self.b.len()) };
        if ret == -1 {
            crate::utils::die!("failed when calling libc::write");
        }
        assert!(ret == self.b.len() as isize);
    }
    pub fn from_bytes<T>(text: T) -> Self
    where
        T: ToCharVec,
    {
        let mut b = Vec::new();
        b.extend_from_slice(text.to_char_vec());
        Self { b }
    }
    pub fn render_from_bytes<T>(text: T) -> Self
    where
        T: ToCharVec,
    {
        // Deal with rendering Tabs.
        let mut b = Vec::new();
        let bytes = text.to_char_vec();
        let tabs = bytes.iter().copied().filter(|&x| x == b'\t').count();
        if tabs == 0 {
            b.extend_from_slice(bytes);
        } else {
            b.reserve(bytes.len() + tabs * (TAB_STOP_SIZE - 1));
            for &ch in bytes.to_char_vec() {
                if ch == b'\t' {
                    b.extend_from_slice(&BLANKS[..TAB_STOP_SIZE]);
                } else {
                    b.push(ch);
                }
            }
        }
        Self { b }
    }
    */

    pub fn len(&self) -> usize {
        self.b.len()
    }
}

pub static BLANKS: &[u8] = &[b' '; 1024 * 2];
pub static TAB_STOP_SIZE: usize = 4;

impl ToCharVec for Buf {
    fn to_char_vec(&self) -> Vec<char> {
        self.b.clone()
    }
}

impl ToCharVec for &Buf {
    fn to_char_vec(&self) -> Vec<char> {
        self.b.clone()
    }
}

pub static EMPTY: &[u8] = &[];

/*
pub fn safe_byte_slice<'a, T>(buf: &'a T, start: usize, max_len: usize) -> &'a [u8]
where
    T: ToCharVec + 'a,
{
    let bytes = buf.to_char_vec();
    if start >= bytes.len() {
        return EMPTY;
    }
    &bytes[start..std::cmp::min(bytes.len(), start + max_len)]
}
*/
macro_rules! buf_fmt {
    ($buf:expr, $($args:expr),+) => {{
        let mut stackbuf = [0u8; 1024];
        let formatted: &str = stackfmt::fmt_truncate(&mut stackbuf, format_args!($($args),+));
        $buf.append(formatted);
        formatted.len()
    }};
}
pub(crate) use buf_fmt;

pub fn place_cursor(buf: &mut Buf, pos: Pos) {
    buf_fmt!(buf, "\x1b[{};{}H", pos.y + 1, pos.x + 1);
}
