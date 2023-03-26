use crate::types::SafeCoordCast;

#[derive(Clone, Debug)]
pub struct Buf {
    b: Vec<u8>,
}

impl Default for Buf {
    fn default() -> Self {
        let mut b = Vec::new();
        b.reserve(256 * 1024);
        Self { b }
    }
}

pub trait ToBufBytes {
    fn to_bytes(&self) -> &[u8];
}

impl ToBufBytes for &[u8] {
    fn to_bytes(&self) -> &[u8] {
        self
    }
}

impl ToBufBytes for &str {
    fn to_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl ToBufBytes for &String {
    fn to_bytes(&self) -> &[u8] {
        self.as_bytes()
    }
}

#[allow(dead_code)]
impl Buf {
    pub fn truncate(&mut self) {
        self.b.truncate(0);
    }
    pub fn reserve(&mut self, size: usize) {
        self.b.reserve(size);
    }
    pub fn append<T>(&mut self, text: T)
    where
        T: ToBufBytes,
    {
        self.b.extend_from_slice(text.to_bytes());
    }
    pub fn append_with_max_len<T, U>(&mut self, text: T, max_len: U)
    where
        T: ToBufBytes,
        U: SafeCoordCast,
    {
        let slice = text.to_bytes();
        self.b
            .extend_from_slice(&slice[0..std::cmp::min(max_len.as_coord(), slice.len())]);
    }

    pub fn write_to(&self, fd: libc::c_int) {
        let ret = unsafe { libc::write(fd, self.b.as_ptr() as *const libc::c_void, self.b.len()) };
        if ret == -1 {
            crate::utils::die!("failed when calling libc::write");
        }
        assert!(ret == self.b.len() as isize);
    }
    pub fn from_bytes<T>(text: T) -> Self
    where
        T: ToBufBytes,
    {
        let mut b = Vec::new();
        b.extend_from_slice(text.to_bytes());
        Self { b }
    }
    pub fn render_from_bytes<T>(text: T) -> Self
    where
        T: ToBufBytes,
    {
        // Deal with rendering Tabs.
        let mut b = Vec::new();
        let bytes = text.to_bytes();
        let tabs = bytes.iter().copied().filter(|&x| x == b'\t').count();
        if tabs == 0 {
            b.extend_from_slice(bytes);
        } else {
            b.reserve(bytes.len() + tabs * (TAB_STOP_SIZE - 1));
            for &ch in bytes.to_bytes() {
                if ch == b'\t' {
                    b.extend_from_slice(&BLANKS[..TAB_STOP_SIZE]);
                } else {
                    b.push(ch);
                }
            }
        }
        Self { b }
    }

    pub fn len(&self) -> usize {
        self.b.len()
    }
    pub fn md5(&self) -> md5::Digest {
        let slice =
            unsafe { std::slice::from_raw_parts(self.b.as_ptr() as *const u8, self.b.len() * 4) };
        md5::compute(slice)
    }
}

pub static BLANKS: &[u8] = &[b' '; 1024 * 2];
pub static TAB_STOP_SIZE: usize = 4;

impl ToBufBytes for Buf {
    fn to_bytes(&self) -> &[u8] {
        &self.b
    }
}

impl ToBufBytes for &Buf {
    fn to_bytes(&self) -> &[u8] {
        &self.b
    }
}

// pub static EMPTY: &[u8] = &[];

/*
pub fn safe_byte_slice<'a, T>(buf: &'a T, start: usize, max_len: usize) -> &'a [u8]
where
    T: ToBufBytes + 'a,
{
    let bytes = buf.to_bytes();
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

/*
pub fn place_cursor(buf: &mut Buf, pos: Pos) {
    buf_fmt!(buf, "\x1b[{};{}H", pos.y + 1, pos.x + 1);
}
*/
