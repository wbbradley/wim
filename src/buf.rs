use crate::types::{Coord, SafeCoordCast};

pub struct Buf {
    b: Vec<u8>,
}

impl Default for Buf {
    fn default() -> Self {
        let mut b = Vec::new();
        b.reserve(2 << 16);
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

impl Buf {
    pub fn truncate(&mut self) {
        self.b.truncate(0);
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
            .extend_from_slice(&slice[0..std::cmp::min(max_len.as_coord() as usize, slice.len())]);
    }

    pub fn splice<T>(&mut self, range: std::ops::Range<Coord>, text: T)
    where
        T: ToBufBytes,
    {
        // Note: `<=` because it's valid to insert after everything
        // which would be equivalent to push.
        let bytes = text.to_bytes();
        self.b.splice(
            range.start as usize..range.end as usize,
            bytes.iter().copied(),
        );
    }

    pub fn write_to(&self, fd: libc::c_int) {
        unsafe { libc::write(fd, self.b.as_ptr() as *const libc::c_void, self.b.len()) };
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
                    b.extend_from_slice(&TAB[..TAB_STOP_SIZE]);
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
}

pub static TAB: &[u8] = &[b' '; 32];
pub static TAB_STOP_SIZE: usize = 4;

impl ToBufBytes for Buf {
    fn to_bytes(&self) -> &[u8] {
        &self.b
    }
}

pub static EMPTY: &[u8] = &[];

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
