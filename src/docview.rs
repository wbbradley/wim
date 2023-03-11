use crate::buf::{place_cursor, safe_byte_slice, Buf, ToBufBytes, BLANKS};
use crate::command::{Command, Direction};
use crate::consts::{PROP_DOCVIEW_CURSOR_POS, PROP_DOC_FILENAME, PROP_DOC_IS_MODIFIED};
use crate::dk::DK;
use crate::doc::Doc;
use crate::error::{Error, Result};
use crate::key::Key;
use crate::mode::Mode;
use crate::noun::Noun;
use crate::status::Status;
use crate::types::{Coord, Pos, Rect, RelCoord, SafeCoordCast};
use crate::utils::wcwidth;
use crate::view::{PropertyValue, View, ViewContext, ViewKey};
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
            let mut f = OpenOptions::new().write(true).create(true).open(filename)?;
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
    fn display(&self, buf: &mut Buf, _context: &dyn ViewContext) {
        log::trace!("docview displaying...");
        let rows_drawn = self.draw_rows(buf);
        log::trace!("rows_drawn={}", rows_drawn);
        for y in rows_drawn..self.frame.height {
            place_cursor(
                buf,
                Pos {
                    x: self.frame.x,
                    y: self.frame.y + y,
                },
            );
            buf.append("~");
            buf.append(&BLANKS[0..self.frame.width - 1]);
        }
    }
    fn get_view_key(&self) -> &ViewKey {
        &self.key
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        Some(Pos {
            x: self.frame.x + self.render_cursor_x - self.scroll_offset.x,
            y: self.frame.y + self.cursor.y - self.scroll_offset.y,
        })
    }
    fn handle_key(&mut self, key: Key) -> Result<DK> {
        match self.mode {
            Mode::Normal => Ok(match key {
                Key::Esc => DK::Noop,
                Key::Ctrl('w') => DK::CloseView,
                Key::Ctrl('s') => DK::Command(Command::Save),
                Key::Del => DK::Noop,
                Key::Left => DK::Command(Command::Move(Direction::Left)),
                Key::Right => DK::Command(Command::Move(Direction::Right)),
                Key::Up => DK::Command(Command::Move(Direction::Up)),
                Key::Down => DK::Command(Command::Move(Direction::Down)),
                Key::Ascii('i') => {
                    self.mode = Mode::Insert;
                    DK::Noop
                }
                Key::Ascii('h') => DK::Command(Command::Move(Direction::Left)),
                Key::Ascii(':') => Command::FocusCommandLine.into(),
                Key::Ascii('j') => DK::Command(Command::Move(Direction::Down)),
                Key::Ascii('k') => DK::Command(Command::Move(Direction::Up)),
                Key::Ascii('l') => DK::Command(Command::Move(Direction::Right)),
                Key::Ascii('J') => DK::Command(Command::JoinLines),
                Key::Ascii('o') => DK::Command(Command::NewlineBelow),
                Key::Ascii('O') => DK::Command(Command::NewlineAbove),
                Key::Ascii('x') => DK::Command(Command::DeleteForwards),
                Key::Ascii('X') => DK::Command(Command::DeleteBackwards),
                _ => {
                    return Err(Error::not_impl(format!(
                        "DocView: Nothing to do for {} in normal mode.",
                        key
                    )));
                }
            }),
            Mode::Insert => Ok(match key {
                Key::Enter => {
                    self.insert_newline_below()?;
                    DK::Noop
                }
                Key::Esc => {
                    self.mode = Mode::Normal;
                    DK::Noop
                }
                Key::Ascii(ch) => {
                    self.insert_char(ch)?;
                    DK::Noop
                }
                Key::Backspace => DK::Command(Command::DeleteBackwards),
                _ => {
                    return Err(Error::not_impl(format!(
                        "DocView: Nothing to do for {:?} in insert mode.",
                        key
                    )));
                }
            }),
            Mode::Visual { block_mode } => Err(Error::not_impl(format!(
                "DocView: Nothing to do for {:?} in visual{} mode.",
                key,
                if block_mode { " block" } else { "" }
            ))),
        }
    }
    fn execute_command(&mut self, command: Command) -> Result<Status> {
        match command {
            Command::Open { filename } => self.open(filename),
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

impl ViewContext for DocView {
    fn get_property(&self, property: &str) -> Option<PropertyValue> {
        if property == PROP_DOC_IS_MODIFIED {
            Some(PropertyValue::Bool(self.doc.is_dirty()))
        } else if property == PROP_DOC_FILENAME {
            self.doc
                .get_filename()
                .map(|filename| PropertyValue::String(filename.to_string()))
        } else if property == PROP_DOCVIEW_CURSOR_POS {
            Some(PropertyValue::Pos(self.cursor))
        } else {
            log::trace!("DocView::get_property unhandled request for '{}'", property);
            None
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
            place_cursor(
                buf,
                Pos {
                    x: frame.x,
                    y: frame.y + count,
                },
            );
            assert!(slice.len() < frame.width);
            buf.append(slice);
            let written_graphemes = wcwidth(slice);
            buf.append(&BLANKS[..frame.width - written_graphemes]);
            count += 1;
        }
        for _ in self.doc.line_count()..frame.height {
            place_cursor(
                buf,
                Pos {
                    x: frame.x,
                    y: frame.y + count,
                },
            );
            buf.append("~");
            buf.append(&BLANKS[0..frame.width - 1]);
            count += 1;
        }
        count
    }
}
