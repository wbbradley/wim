use crate::types::Coord;

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
    pub fn from_str(s: &str) -> Self {
        Self {
            b: s.chars().collect(),
        }
    }
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
    */
    pub fn render_from(buf: &Self) -> Self {
        // Deal with rendering Tabs.
        let mut b = Vec::new();
        let tabs = buf.b.iter().copied().filter(|&x| x == '\t').count();
        if tabs == 0 {
            b = buf.b.clone();
        } else {
            b.reserve(buf.b.len() + tabs * (TAB_STOP_SIZE - 1));
            for &ch in buf.b.iter() {
                if ch == '\t' {
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
}

pub static BLANKS: &[char] = &[' '; 1024 * 2];
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
