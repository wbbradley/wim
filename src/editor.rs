use crate::buf::{safe_byte_slice, Buf, ToBufBytes, BLANKS};
use crate::command::Command;
use crate::doc::Doc;
use crate::error::{Error, Result};
use crate::keygen::KeyGenerator;
use crate::noun::Noun;
use crate::read::{read_key, Key};
use crate::status::Status;
use crate::termios::Termios;
use crate::types::{Coord, Pos, Rect, RelCoord, SafeCoordCast};
use crate::view::{View, DK};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::rc::Rc;
use std::time::{Duration, Instant};

type ViewKeyGenerator = KeyGenerator;

macro_rules! buf_fmt {
    ($buf:expr, $($args:expr),+) => {{
        let mut stackbuf = [0u8; 1024];
        let formatted: &str = stackfmt::fmt_truncate(&mut stackbuf, format_args!($($args),+));
        $buf.append(formatted);
    }};
}
pub(crate) use buf_fmt;

#[derive(Debug)]
pub struct CommandLine {}

#[allow(dead_code)]
pub struct CommandCenter {
    current_filename: Option<String>,
    current_view_key: ViewKey,
    cursor: Coord,
    render_cursor: Coord,
    scroll_offset: Coord,
    text: String,
    frame: Rect,
    status: Status,
}

#[allow(dead_code)]
impl CommandCenter {
    pub fn set_status(&mut self, status: Status) {
        log::trace!("Status Updated: {:?}", &status);
        self.status = status;
    }
}

impl View for CommandCenter {
    fn layout(&mut self, frame: Rect) {
        self.frame = frame;
    }

    fn display(&self, buf: &mut Buf) {
        buf_fmt!(buf, "\x1b[{};{}H", self.frame.y + 1, self.frame.x + 1);
        buf.append("\x1b[7m");
        let mut stackbuf = [0u8; 1024];
        let formatted: &str = stackfmt::fmt_truncate(
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
        let remaining_len = self.frame.width - formatted.len().as_coord();
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

#[allow(dead_code)]
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
        self.scroll();
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
        for _ in self.doc.line_count()..frame.height {
            buf_fmt!(buf, "\x1b[{};{}H", frame.y + count + 1, frame.x + 1);
            buf.append("~");
            buf.append(&BLANKS[0..frame.width - 1]);
            count += 1;
        }
        count
    }
}

pub struct VStack {
    views: Vec<Rc<RefCell<dyn View>>>,
}

impl View for VStack {
    fn layout(&mut self, frame: Rect) {
        let expected_per_view_height = std::cmp::max(1, frame.height / self.views.len());
        let mut used = 0;
        for view in self.views.iter() {
            if frame.height - used < expected_per_view_height {
                break;
            }
            let view_height = if used + expected_per_view_height * 2 > frame.height {
                frame.height - used
            } else {
                expected_per_view_height
            };

            view.borrow_mut().layout(Rect {
                x: frame.x,
                y: used,
                width: frame.width,
                height: view_height,
            });
            used += view_height;
        }
    }
    fn display(&self, buf: &mut Buf) {
        self.views
            .iter()
            .for_each(|view| view.borrow().display(buf));
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        panic!("VStack should not be focused!");
    }
    fn execute_command(&mut self, command: Command) -> Result<Status> {
        Err(Error::new(format!(
            "Command {:?} not implemented for VStack",
            command
        )))
    }
}

pub trait InputHandler {
    fn dispatch_key(&mut self, key: Key) -> Result<Vec<Key>>;
}

pub type ViewKey = String;
#[allow(dead_code)]
pub struct Editor {
    termios: Termios,
    last_key: Option<Key>,
    views: HashMap<ViewKey, Rc<RefCell<DocView>>>,
    view_key_gen: ViewKeyGenerator,
    focused_view: Rc<RefCell<dyn View>>,
    root_view: Rc<RefCell<dyn View>>,
    command_center: Rc<RefCell<CommandCenter>>,
    frame: Rect,
}

impl View for Editor {
    fn layout(&mut self, frame: Rect) {
        self.frame = frame;
        self.root_view.borrow_mut().layout(Rect {
            x: 0,
            y: 0,
            width: frame.width,
            height: frame.height,
        });
    }
    fn display(&self, buf: &mut Buf) {
        buf.truncate();
        // Hide the cursor.
        buf.append("\x1b[?25l");

        if let Some(cursor_pos) = self.focused_view.borrow().get_cursor_pos() {
            buf_fmt!(buf, "\x1b[{};{}H", cursor_pos.y, cursor_pos.x);
        } else {
            buf_fmt!(buf, "\x1b[{};{}H", self.frame.height, self.frame.width);
        }
        buf.append("\x1b[?25h");
        buf.write_to(libc::STDIN_FILENO);
    }

    fn get_cursor_pos(&self) -> Option<Pos> {
        assert!(false);
        None
    }
    fn execute_command(&mut self, command: Command) -> Result<Status> {
        Err(Error::not_impl(format!(
            "{} does not yet implement {:?}",
            std::any::type_name::<Self>(),
            command
        )))
    }
    fn dispatch_key(&mut self, key: Key) -> DK {
        DK::Err(Error::new(format!(
            "{} does not (yet?) handle dispatch_key [key={:?}]",
            std::any::type_name::<Self>(),
            key
        )))
    }
}

fn build_view_map(views: Vec<Rc<RefCell<DocView>>>) -> HashMap<ViewKey, Rc<RefCell<DocView>>> {
    views
        .iter()
        .map(|view| (view.borrow().get_view_key(), view.clone()))
        .collect()
}

#[allow(dead_code)]
impl Editor {
    pub fn _read_key(&mut self) -> Option<Key> {
        let key = read_key();
        self.set_last_key(key);
        key
    }
    pub fn welcome_status() -> Status {
        Status::Message {
            message: String::from("<C-w> to quit..."),
            expiry: Instant::now() + Duration::from_secs(5),
        }
    }
    pub fn new(termios: Termios) -> Self {
        let mut view_key_gen = ViewKeyGenerator::new();

        let views = vec![Rc::new(RefCell::new(DocView {
            key: view_key_gen.next_key_string(),
            cursor: Default::default(),
            render_cursor_x: 0,
            doc: Doc::empty(),
            scroll_offset: Default::default(),
            frame: Rect::zero(),
        }))];
        let focused_view = views[0].clone();
        Self {
            termios,
            frame: Rect::zero(),
            last_key: None,
            views: build_view_map(views),
            view_key_gen,
            focused_view: focused_view.clone(),
            root_view: focused_view.clone(),
            command_center: Rc::new(RefCell::new(CommandCenter {
                current_filename: None,
                current_view_key: focused_view.clone().borrow().get_view_key(),
                cursor: 0,
                render_cursor: 0,
                scroll_offset: 0,
                text: String::new(),
                frame: Rect::zero(),
                status: Status::None,
            })),
        }
    }

    pub fn dispatch_command(&mut self, command: Command) -> Result<Status> {
        self.root_view.borrow_mut().execute_command(command)
    }

    pub fn set_last_key(&mut self, key: Option<Key>) {
        self.last_key = key;
    }

    pub fn set_status(&mut self, status: Status) {
        log::trace!("Status Updated: {:?}", &status);
        self.command_center.borrow_mut().set_status(status);
    }

    pub fn enter_command_mode(&mut self) {
        self.focused_view = self.command_center.clone();
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        println!("Closing wim.\r\n  Screen size was {:?}\r", self.frame);
    }
}
