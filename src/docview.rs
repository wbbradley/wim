use crate::buf::{buf_fmt, safe_byte_slice, Buf, ToBufBytes, BLANKS};
use crate::command::{Command, Direction};
use crate::dk::DK;
use crate::doc::Doc;
use crate::error::{Error, Result};
use crate::mode::Mode;
use crate::noun::Noun;
use crate::read::Key;
use crate::status::Status;
use crate::types::{Coord, Pos, Rect, RelCoord, SafeCoordCast};
use crate::utils::wcwidth;
use crate::view::{View, ViewKey};
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::time::{Duration, Instant};

pub struct DocView {
    key: ViewKey,
    cursor: Pos,
    render_cursor_x: Coord,
    doc: Doc,
    scroll_offset: Pos,
    frame: Rect,
    mode: Mode,
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
    pub fn move_cursor(&mut self, x: RelCoord, y: RelCoord) -> Result<Status> {
        self.cursor.y = (self.cursor.y as RelCoord + y).clamp(0, RelCoord::MAX) as Coord;
        self.cursor.x = (self.cursor.x as RelCoord + x).clamp(0, RelCoord::MAX) as Coord;
        self.clamp_cursor();
        Ok(Status::NothingToSay)
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
    pub fn open(&mut self, filename: String) -> Result<Status> {
        self.doc = Doc::open(filename.clone())?;
        Ok(Status::Message {
            message: format!("Opened '{}'.", filename),
            expiry: Instant::now() + Duration::from_secs(2),
        })
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
    pub fn insert_newline_above(&mut self) -> Result<Status> {
        self.doc.insert_newline(self.cursor.y);
        Ok(Status::NothingToSay)
    }
    pub fn insert_newline_below(&mut self) -> Result<Status> {
        self.doc.insert_newline(self.cursor.y + 1);
        self.move_cursor(0, 1)
    }
    pub fn insert_char(&mut self, ch: char) -> Result<Status> {
        self.doc.insert_char(self.cursor, ch);
        self.move_cursor(1, 0)
    }
    pub fn delete_forwards(&mut self, noun: Noun) -> Result<Status> {
        let (cx, cy) = self.doc.delete_forwards(self.cursor, noun);
        self.jump_cursor(cx, cy);
        Ok(Status::NothingToSay)
    }
    pub fn delete_backwards(&mut self, noun: Noun) -> Result<Status> {
        let (cx, cy) = self.doc.delete_backwards(self.cursor, noun);
        self.jump_cursor(cx, cy);
        Ok(Status::NothingToSay)
    }
    pub fn join_line(&mut self) -> Result<Status> {
        self.doc.join_lines(self.cursor.y..self.cursor.y + 1);
        Ok(Status::NothingToSay)
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
        log::trace!("docview frame is {:?}", frame);
        self.frame = frame;
        self.scroll();
    }
    fn display(&self, buf: &mut Buf) {
        log::trace!("docview displaying...");
        let rows_drawn = self.draw_rows(buf);
        log::trace!("rows_drawn={}", rows_drawn);
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
    fn dispatch_key(&mut self, key: Key) -> Result<DK> {
        match self.mode {
            Mode::Normal => {
                match key {
                    Key::Ctrl('w') => Ok(DK::CloseView),
                    Key::Ctrl('s') => Ok(DK::Command(Command::Save)),
                    Key::Del => Ok(DK::Noop),
                    Key::Left => Ok(DK::Command(Command::Move(Direction::Left))),
                    Key::Right => Ok(DK::Command(Command::Move(Direction::Right))),
                    Key::Up => Ok(DK::Command(Command::Move(Direction::Up))),
                    Key::Down => Ok(DK::Command(Command::Move(Direction::Down))),
                    Key::Ascii('i') => {
                        self.mode = Mode::Insert;
                        Ok(DK::Noop)
                    }
                    Key::Ascii('h') => Ok(DK::Command(Command::Move(Direction::Left))),
                    Key::Ascii('j') => Ok(DK::Command(Command::Move(Direction::Down))),
                    Key::Ascii('k') => Ok(DK::Command(Command::Move(Direction::Up))),
                    Key::Ascii('l') => Ok(DK::Command(Command::Move(Direction::Right))),
                    Key::Ascii('J') => Ok(DK::Command(Command::JoinLines)),
                    Key::Ascii('o') => Ok(DK::Command(Command::NewlineBelow)),
                    Key::Ascii('O') => Ok(DK::Command(Command::NewlineAbove)),
                    Key::Ascii('x') => Ok(DK::Command(Command::DeleteForwards)),
                    Key::Ascii('X') => Ok(DK::Command(Command::DeleteBackwards)),
                    // Key::PageDown => (), // { triggers.extend_from_slice(&[push(Command::Ok(Trigger::Command(Command::Moveedit.move_cursor(0, edit.screen_size.height as RelCoord),
                    /*
                    Key::PageUp => edit.move_cursor(0, -(edit.screen_size.height as RelCoord)),
                    Key::Home => edit.jump_cursor(Some(0), None),
                    Key::Ascii(':') => edit.enter_command_mode(),
                    Key::End => edit.jump_cursor(Some(Coord::MAX), None),
                    Key::Ascii(ch) => edit.insert_char(ch),
                    Key::Ctrl('u') => edit.delete_backwards(Noun::Line),
                    Key::Ctrl('k') => edit.delete_forwards(Noun::Line),
                    Key::Ctrl(_) => (),
                    Key::Function(_) => (),
                    Key::PrintScreen => (),
                    Key::Backspace => (),*/
                    _ => Err(Error::not_impl(format!(
                        "DocView: Nothing to do for {:?} in normal mode.",
                        key
                    ))),
                }
            }
            Mode::Insert => match key {
                Key::Esc => {
                    self.mode = Mode::Normal;
                    Ok(DK::Noop)
                }
                Key::Ascii(ch) => {
                    self.insert_char(ch)?;
                    Ok(DK::Noop)
                }
                Key::Backspace => Ok(DK::Command(Command::DeleteBackwards)),
                _ => Err(Error::not_impl(format!(
                    "DocView: Nothing to do for {:?} in insert mode.",
                    key
                ))),
            },
            Mode::Visual { block_mode } => Err(Error::not_impl(format!(
                "DocView: Nothing to do for {:?} in visual{} mode.",
                key,
                if block_mode { " block" } else { "" }
            ))),
        }
    }
    fn execute_command(&mut self, command: Command) -> Result<Status> {
        match command {
            Command::Open { filename } => self.open(filename.clone()),
            Command::Move(direction) => match direction {
                Direction::Up => self.move_cursor(0, -1),
                Direction::Down => self.move_cursor(0, 1),
                Direction::Left => self.move_cursor(-1, 0),
                Direction::Right => self.move_cursor(1, 0),
            },
            Command::JoinLines => self.join_line(),
            Command::NewlineAbove => self.insert_newline_above(),
            Command::NewlineBelow => self.insert_newline_below(),
            Command::DeleteForwards => self.delete_forwards(Noun::Char),
            Command::DeleteBackwards => self.delete_backwards(Noun::Char),
            _ => Err(Error::not_impl(format!(
                "DocView::execute_command needs to handle {:?}.",
                command,
            ))),
        }
    }
}

impl DocView {
    pub fn new(view_key: ViewKey) -> Self {
        Self {
            key: view_key,
            cursor: Default::default(),
            render_cursor_x: 0,
            doc: Doc::empty(),
            scroll_offset: Default::default(),
            frame: Rect::zero(),
            mode: Mode::Normal,
        }
    }

    fn draw_rows(&self, buf: &mut Buf) -> Coord {
        let frame = self.frame;
        let mut count = 0;
        for (i, row) in self.doc.iter_lines().enumerate().skip(self.scroll_offset.y) {
            if i.as_coord() - self.scroll_offset.y >= frame.height {
                break;
            }
            let slice = safe_byte_slice(row.render_buf(), self.scroll_offset.x, frame.width);
            buf_fmt!(buf, "\x1b[{};{}H", frame.y + count + 1, frame.x + 1);
            assert!(slice.len() < frame.width);
            buf.append(slice);
            let written_graphemes = wcwidth(slice);
            buf.append(&BLANKS[..frame.width - written_graphemes]);
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
