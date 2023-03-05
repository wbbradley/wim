use crate::buf::{safe_byte_slice, Buf, ToBufBytes, TAB};
use crate::read::{read_u8, Key};
use crate::termios::Termios;
use crate::types::{Coord, SafeCoordCast};
use crate::utils::put;
use crate::VERSION;
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

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

#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
pub struct Size {
    pub width: Coord,
    pub height: Coord,
}

#[allow(dead_code)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Pos {
    pub x: Coord,
    pub y: Coord,
}

impl From<Pos> for Size {
    fn from(coord: Pos) -> Self {
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
                render_x += (TAB.len() - 1) - render_x % TAB.len();
            }
            render_x += 1;
        }
        render_x.as_coord()
    }
}

impl ToBufBytes for &Row {
    fn to_bytes(&self) -> &[u8] {
        self.buf.to_bytes()
    }
}

#[allow(dead_code)]
pub struct Editor {
    termios: Termios,
    pub screen_size: Size,
    filename: Option<String>,
    cursor: Pos,
    render_cursor_x: Coord,
    last_key: Key,
    rows: Vec<Row>,
    scroll_offset: Pos,
    control_center: Size,
}

impl Editor {
    pub fn new() -> Self {
        Self {
            termios: Termios::enter_raw_mode(),
            screen_size: get_window_size(),
            filename: None,
            cursor: Default::default(),
            render_cursor_x: 0,
            last_key: Key::Ascii(' '),
            rows: Default::default(),
            scroll_offset: Default::default(),
            control_center: Size {
                width: 0,
                height: 2,
            },
        }
    }
    pub fn draw_control_center(&self, buf: &mut Buf) {
        buf.append("\x1b[7m");
        for _ in 0..self.screen_size.width {
            buf.append("-");
        }
        buf.append("\x1b[m");
        buf_fmt!(
            buf,
            "\r\n{} [Last key: {}. scroll_offset: {:?}. cursor=(line: {}, col: {})]\x1b[K",
            match self.filename {
                Some(ref filename) => filename.as_str(),
                None => "<no filename>",
            },
            self.last_key,
            self.scroll_offset,
            self.cursor.y + 1,
            self.cursor.x + 1
        );
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
            .rows
            .iter()
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
        for y in self.rows.len()..screen_height as usize {
            if self.rows.is_empty() && y == self.screen_size.height as usize / 3 {
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

    pub fn clamp_cursor(&mut self) {
        self.cursor.y = self.cursor.y.clamp(0, self.last_valid_row());
        if let Some(row) = self.rows.get(self.cursor.y as usize) {
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
    pub fn open(&mut self, filename: String) -> io::Result<()> {
        self.rows.truncate(0);
        let lines = read_lines(&filename)?;
        for line in lines {
            self.rows.push(Row::from_line(&line?));
        }
        self.filename = Some(filename);
        Ok(())
    }
    pub fn last_valid_row(&self) -> Coord {
        self.rows.len().as_coord()
    }
}
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

impl Drop for Editor {
    fn drop(&mut self) {
        println!("Closing wim.\r\n  Screen size was {:?}\r", self.screen_size);
    }
}
