use crate::buf::{safe_byte_slice, Buf, ToBufBytes, BLANKS};
use crate::doc::Doc;
use crate::error::{Error, Result};
use crate::keygen::KeyGenerator;
use crate::noun::Noun;
use crate::read::{read_u8, Key};
use crate::termios::Termios;
use crate::types::{Coord, Pos, Rect, RelCoord, SafeCoordCast, Size};
use crate::utils::put;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::rc::Rc;
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

#[derive(Debug)]
pub struct CommandLine {}

#[derive(Debug)]
pub enum Mode {
    Normal(Status),
    Command(CommandLine),
}

pub struct CommandCenter {
    current_filename: Option<String>,
    current_view_key: ViewKey,
    cursor: Coord,
    render_cursor: Coord,
    scroll_offset: Coord,
    text: String,
    frame: Rect,
}

impl View for CommandCenter {
    fn layout(&mut self, frame: Rect) {
        self.frame = frame;
    }

    fn display(&self, buf: &mut Buf) {
        buf_fmt!(buf, "\x1b[{};{}H", self.frame.y + 1, self.frame.x + 1);
        buf.append("\x1b[7m");
        let mut stackbuf = [0u8; 1024];
        let mut formatted: &str = stackfmt::fmt_truncate(
            &mut stackbuf,
            format_args!(
                "{}",
                match self.current_filename {
                    Some(ref filename) => filename.as_str(),
                    None => "<no filename>",
                },
                // if self.doc.is_dirty() { "| +" } else { "" }
            ),
        );
        buf.append(formatted);
        let mut remaining_len = self.frame.width - formatted.len().as_coord();
        /*
        formatted = stackfmt::fmt_truncate(
            &mut stackbuf,
            format_args!(
                "[scroll_offset: {:?}. cursor=(line: {}, col: {})]",
                self.scroll_offset,
                self.cursor.y + 1,
                self.cursor.x + 1
            ),
        );
        remaining_len -= formatted.len().as_coord();
        */
        buf.append(&BLANKS[..remaining_len]);
        buf.append(formatted);
        buf.append("\x1b[m");
        buf_fmt!(buf, "\x1b[{};{}H", self.frame.y + 2, self.frame.x + 1);
        // TODO: render prompt...
        buf.append(&BLANKS[..self.frame.width]);
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        Some(Pos {
            x: self.frame.x + 1,
            y: self.frame.y + 2,
        })
    }
}

pub struct DocView {
    key: ViewKey,
    cursor: Pos,
    render_cursor_x: Coord,
    doc: Doc,
    scroll_offset: Pos,
    frame: Rect,
}

impl DocView {
    pub fn get_view_key(&self) -> ViewKey {
        self.key.clone()
    }
    pub fn scroll(&mut self) {
        if self.cursor.y < self.scroll_offset.y {
            self.scroll_offset.y = self.cursor.y;
        }
        if self.cursor.y >= self.scroll_offset.y + self.frame.height {
            self.scroll_offset.y = self.cursor.y - self.frame.height + 1;
        }
        if self.render_cursor_x < self.scroll_offset.x {
            self.scroll_offset.x = self.render_cursor_x;
        }
        if self.render_cursor_x >= self.scroll_offset.x + self.frame.width {
            self.scroll_offset.x = self.render_cursor_x - self.frame.width + 1;
        }
    }
    pub fn move_cursor(&mut self, x: RelCoord, y: RelCoord) {
        self.cursor.y = (self.cursor.y as RelCoord + y).clamp(0, RelCoord::MAX) as Coord;
        self.cursor.x = (self.cursor.x as RelCoord + x).clamp(0, RelCoord::MAX) as Coord;
        self.clamp_cursor();
    }

    pub fn last_valid_row(&self) -> Coord {
        self.doc.line_count().as_coord()
    }
    fn clamp_cursor(&mut self) {
        self.cursor.y = self.cursor.y.clamp(0, self.last_valid_row());
        if let Some(row) = self.doc.get_line_buf(self.cursor.y) {
            self.cursor.x = self.cursor.x.clamp(0, row.len());
            self.render_cursor_x = row.cursor_to_render_col(self.cursor.x);
        } else {
            self.cursor.x = 0;
            self.render_cursor_x = 0;
        };
    }
    pub fn jump_cursor(&mut self, x: Option<Coord>, y: Option<Coord>) {
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
    pub fn join_line(&mut self) {
        self.doc.join_lines(self.cursor.y..self.cursor.y + 1);
    }
    pub fn get_save_buffer(&self) -> Buf {
        let mut buf = Buf::default();
        for row in self.doc.iter_lines() {
            buf.append(row);
            buf.append("\n");
        }
        buf
    }
}

impl View for DocView {
    fn layout(&mut self, frame: Rect) {
        self.frame = frame;
    }
    fn display(&self, buf: &mut Buf) {
        let rows_drawn = self.draw_rows(buf, self.frame);
        for y in rows_drawn..self.frame.height {
            buf_fmt!(buf, "\x1b[{};{}H~", self.frame.y + y + 1, self.frame.x + 1);
            buf.append(&BLANKS[0..self.frame.width - 1]);
        }
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        Some(Pos {
            x: self.frame.x + self.render_cursor_x - self.scroll_offset.x + 1,
            y: self.frame.y + self.cursor.y - self.scroll_offset.y + 1,
        })
    }
}

impl DocView {
    fn draw_rows(&self, buf: &mut Buf, frame: Rect) -> Coord {
        let mut count = 0;
        for (i, row) in self.doc.iter_lines().enumerate().skip(self.scroll_offset.y) {
            if i.as_coord() - self.scroll_offset.y >= frame.height {
                break;
            }
            let slice = safe_byte_slice(row.render_buf(), self.scroll_offset.x, frame.width - 1);
            buf_fmt!(buf, "\x1b[{};{}H", frame.y + count + 1, frame.x + 1);
            buf.append_with_max_len(slice, frame.width - 1);
            buf.append("..todo..clear");
            count += 1;
        }
        for y in self.doc.line_count()..frame.height {
            buf_fmt!(buf, "\x1b[{};{}H", frame.y + count + 1, frame.x + 1);
            buf.append("~");
            buf.append(&BLANKS[0..frame.width - 1]);
            count += 1;
        }
        count
    }
}

pub trait View {
    fn layout(&mut self, frame: Rect);
    fn display(&self, buf: &mut Buf);
    fn get_cursor_pos(&self) -> Option<Pos>;
}

pub struct VStack {
    views: Vec<Rc<dyn View>>,
}

impl View for VStack {
    fn layout(&mut self, frame: Rect) {
        let per_view_height = std::cmp::max(1, frame.height / self.views.len());
        let mut remaining = frame.height;
        let mut used = 0;
        for view in self.views {
            if remaining < per_view_height {
                break;
            }
            view.layout(Rect {
                x: frame.x,
                y: used,
                width: frame.width,
                height: per_view_height,
            });
        }
    }
    fn display(&self, buf: &mut Buf) {
        self.views.iter().map(|view| view.display(buf));
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        assert!(false, "VStack should not be focused!");
        None
    }
}

impl InputHandler for DocView {
    fn dispatch(&mut self, key: Key) -> Result<()> {
        Ok(())
    }
}

pub trait InputHandler {
    fn dispatch(&mut self, key: Key) -> Result<()>;
}

pub type ViewKey = String;
#[allow(dead_code)]
pub struct Editor {
    termios: Termios,
    pub screen_size: Size,
    last_key: Key,
    views: HashMap<ViewKey, Rc<DocView>>,
    view_key_gen: ViewKeyGenerator,
    focused_view: Rc<DocView>,
    root_view: Rc<dyn View>,
    command_center: Rc<CommandCenter>,
}

fn build_view_map(views: Vec<Rc<DocView>>) -> HashMap<ViewKey, Rc<DocView>> {
    views
        .iter()
        .map(|view| (view.get_view_key(), view.clone()))
        .collect()
}

type ViewKeyGenerator = KeyGenerator;

impl Editor {
    pub fn welcome_status() -> Status {
        Status::Message {
            message: String::from("<C-w> to quit..."),
            expiry: Instant::now() + Duration::from_secs(5),
        }
    }
    pub fn new() -> Self {
        let mut view_key_gen = ViewKeyGenerator::new();

        let views = vec![Rc::new(DocView {
            key: view_key_gen.next_key_string(),
            cursor: Default::default(),
            render_cursor_x: 0,
            doc: Doc::empty(),
            scroll_offset: Default::default(),
            frame: Rect::zero(),
        })];
        let focused_view = views[0].clone();
        let mut editor = Self {
            termios: Termios::enter_raw_mode(),
            screen_size: get_window_size(),
            last_key: Key::Ascii(' '),
            views: build_view_map(views),
            view_key_gen,
            focused_view,
            root_view: focused_view,
            command_center: Rc::new(CommandCenter {
                current_filename: None,
                current_view_key: focused_view.get_view_key(),
                cursor: 0,
                render_cursor: 0,
                scroll_offset: 0,
                text: String::new(),
                frame: Rect::zero(),
            }),
        };
        editor
    }

    /*
    pub fn expired_status(&mut self) -> bool {
        match self.mode {
            Mode::Normal(Status::None) => false,
            Mode::Normal(Status::Message { message: _, expiry }) => {
                if expiry <= Instant::now() {
                    self.mode = Mode::Normal(Status::None);
                    return true;
                }
                false
            }
            Mode::Command(_) => false,
        }
    }
    */

    pub fn refresh_screen(&mut self, buf: &mut Buf) {
        self.root_view.layout(Rect {
            x: 0,
            y: 0,
            width: self.screen_size.width,
            height: self.screen_size.height,
        });

        buf.truncate();
        // Hide the cursor.
        buf.append("\x1b[?25l");

        if let Some(cursor_pos) = self.focused_view.get_cursor_pos() {
            buf_fmt!(buf, "\x1b[{};{}H", cursor_pos.y, cursor_pos.x);
        } else {
            buf_fmt!(
                buf,
                "\x1b[{};{}H",
                self.screen_size.height,
                self.screen_size.width
            );
        }
        buf.append("\x1b[?25h");
        buf.write_to(libc::STDIN_FILENO);
    }

    pub fn set_last_key(&mut self, key: Key) {
        self.last_key = key;
    }
    pub fn set_status(&mut self, status: Status) {
        self.mode = Mode::Normal(status);
        log::trace!("Status Updated: {:?}", self.mode);
    }

    pub fn enter_command_mode(&mut self) {
        match self.mode {
            Mode::Command(_) => (),
            Mode::Normal(_) => {
                self.mode = Mode::Command(CommandLine {});
            }
        }
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        println!("Closing wim.\r\n  Screen size was {:?}\r", self.screen_size);
    }
}
