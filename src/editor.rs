use crate::buf::{safe_byte_slice, Buf, ToBufBytes};
use crate::doc::Doc;
use crate::error::{Error, Result};
use crate::noun::Noun;
use crate::read::{read_u8, Key};
use crate::termios::Termios;
use crate::types::{Coord, Pos, SafeCoordCast, Size};
use crate::utils::put;
use crate::VERSION;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::time::{Duration, Instant};

fn get_cursor_position() -> Option<Pos> {
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
    let y: Coord = lexical::parse(&buf[0..semicolon_position]).unwrap();
    let x: Coord = lexical::parse(&buf[semicolon_position + 1..]).unwrap();
    Some(Pos { y, x })
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
            width: ws.ws_col.as_coord(),
            height: ws.ws_row.as_coord(),
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

#[derive(Debug)]
pub enum Status {
    Message { message: String, expiry: Instant },
    None,
}

#[allow(dead_code)]
pub struct Editor {
    termios: Termios,
    pub screen_size: Size,
    cursor: Pos,
    render_cursor_x: Coord,
    last_key: Key,
    doc: Doc,
    scroll_offset: Pos,
    control_center: Size,
    _status: Status,
}

impl Editor {
    pub fn welcome_status() -> Status {
        Status::Message {
            message: String::from("<C-w> to quit..."),
            expiry: Instant::now() + Duration::from_secs(5),
        }
    }
    pub fn new() -> Self {
        Self {
            termios: Termios::enter_raw_mode(),
            screen_size: get_window_size(),
            cursor: Default::default(),
            render_cursor_x: 0,
            last_key: Key::Ascii(' '),
            doc: Doc::empty(),
            scroll_offset: Default::default(),
            control_center: Size {
                width: 0,
                height: 2,
            },
            _status: Self::welcome_status(),
        }
    }
    pub fn draw_control_center(&self, buf: &mut Buf) {
        buf.append("\x1b[7m");
        let mut stackbuf = [0u8; 1024];
        let mut formatted: &str = stackfmt::fmt_truncate(
            &mut stackbuf,
            format_args!(
                "{}{}",
                match self.doc.get_filename() {
                    Some(ref filename) => filename.as_str(),
                    None => "<no filename>",
                },
                if self.doc.is_dirty() { "| +" } else { "" }
            ),
        );
        buf.append(formatted);
        let mut remaining_len = self.screen_size.width - formatted.len().as_coord();
        formatted = stackfmt::fmt_truncate(
            &mut stackbuf,
            format_args!(
                "[Last key: {}. scroll_offset: {:?}. cursor=(line: {}, col: {})]",
                self.last_key,
                self.scroll_offset,
                self.cursor.y + 1,
                self.cursor.x + 1
            ),
        );
        remaining_len -= formatted.len().as_coord();
        for _ in 0..remaining_len {
            buf.append(" ");
        }
        buf.append(formatted);
        buf.append("\x1b[m\r\n");

        if let Status::Message {
            ref message,
            expiry,
        } = self._status
        {
            if expiry > Instant::now() {
                buf.append(message);
            }
        }
        buf.append("\x1b[K");
    }

    pub fn expired_status(&mut self) -> bool {
        if let Status::Message { message: _, expiry } = self._status {
            if expiry <= Instant::now() {
                self._status = Status::None;
                return true;
            }
        }
        false
    }

    pub fn refresh_screen(&self, buf: &mut Buf) {
        buf.truncate();
        buf.append("\x1b[?25l\x1b[H");
        let rows_drawn = self.draw_rows(buf);
        for _ in rows_drawn..self.screen_size.height - self.control_center.height {
            buf.append("~\x1b[K\r\n");
        }
        self.draw_control_center(buf);

        buf_fmt!(
            buf,
            "\x1b[{};{}H",
            self.cursor.y - self.scroll_offset.y + 1,
            self.render_cursor_x - self.scroll_offset.x + 1
        );
        buf.append("\x1b[?25h");
        buf.write_to(libc::STDIN_FILENO);
    }

    fn draw_rows(&self, buf: &mut Buf) -> Coord {
        let screen_height = self.screen_size.height - self.control_center.height;
        let mut count = 0;
        for (i, row) in self
            .doc
            .iter_lines()
            .enumerate()
            .skip(self.scroll_offset.y as usize)
        {
            if i.as_coord() - self.scroll_offset.y >= screen_height {
                break;
            }
            let slice = safe_byte_slice(
                row.render_buf(),
                self.scroll_offset.x as usize,
                self.screen_size.width as usize - 1,
            );
            buf.append_with_max_len(slice, self.screen_size.width - 1);
            buf.append("\x1b[K\r\n");
            count += 1;
        }
        for y in self.doc.line_count()..screen_height as usize {
            if self.doc.is_empty() && y == self.screen_size.height as usize / 3 {
                let welcome = format!("Wim editor -- version {}", VERSION);
                let mut welcome_len = welcome.len().as_coord();
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

            buf.append("\x1b[K");
            if y < screen_height as usize - 1 {
                buf.append("\r\n");
                count += 1;
            }
        }
        count
    }
    pub fn scroll(&mut self) {
        if self.cursor.y < self.scroll_offset.y {
            self.scroll_offset.y = self.cursor.y;
        }
        if self.cursor.y
            >= self.scroll_offset.y + self.screen_size.height - self.control_center.height
        {
            self.scroll_offset.y =
                self.cursor.y - (self.screen_size.height - self.control_center.height) + 1;
        }
        if self.render_cursor_x < self.scroll_offset.x {
            self.scroll_offset.x = self.render_cursor_x;
        }
        if self.render_cursor_x >= self.scroll_offset.x + self.screen_size.width {
            self.scroll_offset.x = self.render_cursor_x - self.screen_size.width + 1;
        }
    }

    pub fn set_last_key(&mut self, key: Key) {
        self.last_key = key;
    }

    pub fn move_cursor(&mut self, x: Coord, y: Coord) {
        self.cursor.y += y;
        self.cursor.x += x;
        self.clamp_cursor();
    }

    fn clamp_cursor(&mut self) {
        self.cursor.y = self.cursor.y.clamp(0, self.last_valid_row());
        if let Some(row) = self.doc.get_line_buf(self.cursor.y) {
            self.cursor.x = self.cursor.x.clamp(0, row.len() as i64);
            self.render_cursor_x = row.cursor_to_render_col(self.cursor.x);
        } else {
            self.cursor.x = 0;
            self.render_cursor_x = 0;
        };
    }
    pub fn jump_cursor(&mut self, x: Option<i64>, y: Option<i64>) {
        if let Some(y) = y {
            self.cursor.y = y;
        }
        if let Some(x) = x {
            self.cursor.x = x;
        }
        self.clamp_cursor();
    }
    pub fn open(&mut self, filename: String) -> Result<()> {
        self.doc = Doc::open(filename)?;
        Ok(())
    }
    pub fn last_valid_row(&self) -> Coord {
        self.doc.line_count().as_coord()
    }
    pub fn get_save_buffer(&self) -> Buf {
        let mut buf = Buf::default();
        for row in self.doc.iter_lines() {
            buf.append(row);
            buf.append("\n");
        }
        buf
    }
    pub fn set_status(&mut self, status: Status) {
        self._status = status;
        log::trace!("Status Updated: {:?}", self._status);
    }
    pub fn save_file(&mut self) -> Result<Status> {
        // TODO: write + rename.
        let save_buffer = self.get_save_buffer();
        if let Some(filename) = self.doc.get_filename() {
            let mut f = OpenOptions::new()
                .write(true)
                .create(true)
                .open(&filename)?;
            f.set_len(0)?;
            f.seek(SeekFrom::Start(0))?;
            let bytes = save_buffer.to_bytes();
            f.write_all(bytes)?;
            f.flush()?;
            Ok(Status::Message {
                message: format!("{} saved [{}b]!", filename, bytes.len()),
                expiry: Instant::now() + Duration::from_secs(2),
            })
        } else {
            Err(Error::new("no filename specified!"))
        }
    }
    pub fn insert_newline_above(&mut self) {
        self.doc.insert_newline(self.cursor.y);
    }
    pub fn insert_newline_below(&mut self) {
        self.doc.insert_newline(self.cursor.y + 1);
        self.move_cursor(0, 1);
    }
    pub fn insert_char(&mut self, ch: char) {
        self.doc.insert_char(self.cursor, ch);
        self.move_cursor(1, 0);
    }
    pub fn delete_forwards(&mut self, noun: Noun) {
        let (cx, cy) = self.doc.delete_forwards(self.cursor, noun);
        self.jump_cursor(cx, cy);
    }
    pub fn delete_backwards(&mut self, noun: Noun) {
        let (cx, cy) = self.doc.delete_backwards(self.cursor, noun);
        self.jump_cursor(cx, cy);
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        println!("Closing wim.\r\n  Screen size was {:?}\r", self.screen_size);
    }
}
