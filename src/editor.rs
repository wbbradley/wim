use crate::buf::Buf;
use crate::read::{read_u8, Key};
use crate::termios::Termios;
use crate::utils::put;
use crate::VERSION;

fn get_cursor_position() -> Option<Coord> {
    let mut buf = [0u8; 32];
    let mut i: usize = 0;

    // Write the "get position" command.
    if put!("\x1b[6n") != 4 {
        return None;
    }
    loop {
        if i >= 32 - 1 {
            break;
        }
        if let Some(ch) = read_u8() {
            buf[i] = ch;
            if ch == b'R' {
                break;
            }
            i += 1;
        } else {
            return None;
        }
    }
    buf[i] = 0;
    if buf[0] != 0x1b || buf[1] != b'[' {
        return None;
    }
    let buf = &buf[2..i];
    let semicolon_position = buf.iter().position(|x| *x == b';').unwrap();
    let y: i64 = lexical::parse(&buf[0..semicolon_position]).unwrap();
    let x: i64 = lexical::parse(&buf[semicolon_position + 1..]).unwrap();
    Some(Coord { y, x })
}

fn get_window_size() -> Size {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    if unsafe {
        libc::ioctl(
            libc::STDOUT_FILENO,
            libc::TIOCGWINSZ,
            &mut ws as *mut libc::winsize as *mut libc::c_void,
        )
    } == -1
        || ws.ws_col == 0
    {
        if put!("\x1b[999C\x1b[999B") != 12 {
            read_u8();
            Size {
                width: 80,
                height: 24,
            }
        } else if let Some(coord) = get_cursor_position() {
            coord.into()
        } else {
            Size {
                width: 80,
                height: 24,
            }
        }
    } else {
        Size {
            width: ws.ws_col as i64,
            height: ws.ws_row as i64,
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub width: i64,
    pub height: i64,
}

#[allow(dead_code)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Coord {
    pub x: i64,
    pub y: i64,
}

impl From<Coord> for Size {
    fn from(coord: Coord) -> Self {
        Self {
            width: coord.x,
            height: coord.y,
        }
    }
}

macro_rules! buf_fmt {
    ($buf:expr, $($args:expr),+) => {{
        let mut stackbuf = [0u8; 1024];
        let formatted: &str = stackfmt::fmt_truncate(&mut stackbuf, format_args!($($args),+));
        $buf.append(formatted);
    }};
}
pub(crate) use buf_fmt;

#[allow(dead_code)]
#[derive(Default)]
struct Row {
    buf: Buf,
}

impl Row {
    #[inline]
    pub fn append(&mut self, text: &str) {
        self.buf.append(text)
    }
    pub fn get_buf_bytes(&self) -> &[u8] {
        return self.buf.get_bytes();
    }
}

#[allow(dead_code)]
pub struct Editor {
    termios: Termios,
    pub screen_size: Size,
    cursor: Coord,
    last_key: Key,
    row: Row,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            termios: Termios::enter_raw_mode(),
            screen_size: get_window_size(),
            cursor: Default::default(),
            last_key: Key::Ascii(' '),
            row: Default::default(),
        }
    }
    pub fn refresh_screen(&self, buf: &mut Buf) {
        buf.truncate();
        buf.append("\x1b[?25l\x1b[H");
        self.draw_rows(buf);
        buf_fmt!(buf, "Last key: {}", self.last_key);
        buf_fmt!(buf, "\x1b[{};{}H", self.cursor.y + 1, self.cursor.x + 1);
        buf.append("\x1b[?25h");
        buf.write_to(libc::STDIN_FILENO);
    }

    fn num_rows(&self) -> i64 {
        1
    }
    fn draw_rows(&self, buf: &mut Buf) {
        for y in 0..self.screen_size.height {
            if y >= self.num_rows() {
                if y == self.screen_size.height / 3 {
                    let welcome = format!("Wim editor -- version {}", VERSION);
                    let mut welcome_len = welcome.len() as i64;
                    if welcome_len > self.screen_size.width {
                        welcome_len = self.screen_size.width;
                    }
                    let mut padding = (self.screen_size.width - welcome_len) / 2;
                    if padding != 0 {
                        buf.append("~");
                        padding -= 1;
                    }
                    for _ in 0..padding {
                        buf.append(" ");
                    }
                    buf.append_with_max_len(&welcome, welcome_len as usize);
                } else {
                    buf.append("~");
                }
            } else {
                buf.append(self.row.get_buf_bytes());
            }

            buf.append("\x1b[K");
            if y < self.screen_size.height - 1 {
                buf.append("\r\n");
            }
        }
    }
    pub fn set_last_key(&mut self, key: Key) {
        self.last_key = key;
    }

    pub fn move_cursor(&mut self, x: i64, y: i64) {
        self.cursor.y = (self.cursor.y + y).clamp(0, self.screen_size.height - 1);
        self.cursor.x = (self.cursor.x + x).clamp(0, self.screen_size.width - 1);
    }
    pub fn jump_cursor(&mut self, x: Option<i64>, y: Option<i64>) {
        if let Some(y) = y {
            self.cursor.y = y.clamp(0, self.screen_size.height - 1);
        }
        if let Some(x) = x {
            self.cursor.x = x.clamp(0, self.screen_size.width - 1);
        }
    }
    pub fn open(&mut self) {
        buf_fmt!(self.row, "Hello, world!");
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        println!("Closing wim.\r\n  Screen size was {:?}\r", self.screen_size);
    }
}
