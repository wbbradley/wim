use crate::bindings::Bindings;
use crate::buf::{place_cursor, safe_byte_slice, Buf, ToBufBytes, BLANKS};
use crate::command::{Command, Direction};
use crate::consts::{
    PROP_DOCVIEW_CURSOR_POS, PROP_DOCVIEW_STATUS, PROP_DOC_FILENAME, PROP_DOC_IS_MODIFIED,
};
use crate::dk::DK;
use crate::doc::Doc;
use crate::error::{not_impl, Error, Result};
use crate::key::Key;
use crate::mode::Mode;
use crate::noun::Noun;
use crate::plugin::PluginRef;
use crate::propvalue::PropertyValue;
use crate::rel::Rel;
use crate::status::Status;
use crate::types::{Coord, Pos, Rect, RelCoord, SafeCoordCast};
use crate::utils::wcwidth;
use crate::view::{View, ViewContext, ViewKey};
use std::cell::RefCell;
use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::rc::Weak;
use std::time::{Duration, Instant};

pub struct DocView {
    parent: Option<Weak<RefCell<dyn View>>>,
    plugin: PluginRef,
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
    pub fn set_parent(&mut self, parent: Option<Weak<RefCell<dyn View>>>) {
        self.parent = parent;
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
        Ok(Status::Ok)
    }
    pub fn move_cursor_rel(&mut self, noun: Noun, rel: Rel) -> Result<Status> {
        match (noun, rel) {
            (Noun::Char, Rel::Prior) => self.move_cursor(-1, 0),
            (Noun::Char, Rel::Next) => self.move_cursor(1, 0),
            (Noun::Line, Rel::Prior) => self.move_cursor(0, -1),
            (Noun::Line, Rel::Next) => self.move_cursor(0, 1),
            // (Noun::Word, Rel::Next) => self.move_cursor_next_word(),
            _ => Err(not_impl!(
                "DocView: Don't know how to handle relative motion for ({:?}, {:?}).",
                noun,
                rel
            )),
        }
    }

    pub fn last_valid_row(&self) -> Coord {
        self.doc.line_count()
    }
    fn clamp_cursor(&mut self) {
        log::trace!("clamp_cursor starts at {:?}", self.cursor);
        self.cursor.y = self.cursor.y.clamp(0, self.last_valid_row());
        if let Some(row) = self.doc.get_line_buf(self.cursor.y) {
            self.cursor.x = self.cursor.x.clamp(
                0,
                row.len() - usize::from(row.len() > 0 && self.mode == Mode::Normal),
            );
            self.render_cursor_x = row.cursor_to_render_col(self.cursor.x);
        } else {
            self.cursor.x = 0;
            self.render_cursor_x = 0;
        };
        log::trace!("clamp_cursor ends at {:?}", self.cursor);
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
        Ok(Status::Ok)
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
        Ok(Status::Ok)
    }
    pub fn delete_backwards(&mut self, noun: Noun) -> Result<Status> {
        let (cx, cy) = self.doc.delete_backwards(self.cursor, noun);
        self.jump_cursor(cx, cy);
        Ok(Status::Ok)
    }
    pub fn join_line(&mut self) -> Result<Status> {
        self.doc.join_lines(self.cursor.y..self.cursor.y + 1);
        Ok(Status::Ok)
    }
    pub fn get_save_buffer(&self) -> Buf {
        let mut buf = Buf::default();
        for row in self.doc.iter_lines() {
            buf.append(row);
            buf.append("\n");
        }
        buf
    }
    fn switch_mode(&mut self, mode: Mode) {
        self.mode = mode;
        self.clamp_cursor();
    }
}

impl View for DocView {
    fn get_parent(&self) -> Option<Weak<RefCell<dyn View>>> {
        self.parent.clone()
    }
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.plugin = plugin;
    }
    fn layout(&mut self, frame: Rect) {
        log::trace!("docview frame is {:?}", frame);
        self.frame = frame;
        self.scroll();
    }
    fn display(&self, buf: &mut Buf, _context: &dyn ViewContext) {
        let rows_drawn = self.draw_rows(buf);
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
    fn get_view_mode(&self) -> Mode {
        self.mode
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        Some(Pos {
            x: self.frame.x + self.render_cursor_x - self.scroll_offset.x,
            y: self.frame.y + self.cursor.y - self.scroll_offset.y,
        })
    }
    fn get_key_bindings(&self) -> Bindings {
        let mut bindings: Bindings = Default::default();
        match self.mode {
            Mode::Visual { .. } => {}
            Mode::Insert => {
                bindings.add(
                    &self.key,
                    vec![Key::Esc],
                    DK::Command(Command::SwitchMode(Mode::Normal)),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('j'), Key::Ascii('k')],
                    DK::Key(Key::Esc),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Backspace],
                    Command::DeleteBackwards.into(),
                );
                bindings.add(&self.key, vec![Key::Enter], Command::NewlineBelow.into());
            }
            Mode::Normal => {
                bindings.add(&self.key, vec![Key::Ctrl('w')], DK::CloseView);
                bindings.add(&self.key, vec![Key::Ctrl('s')], Command::Save.into());
                bindings.add(&self.key, vec![Key::Esc], DK::Noop);
                bindings.add(&self.key, vec![Key::Ctrl('s')], Command::Save.into());
                bindings.add(&self.key, vec![Key::Del], DK::Noop);
                bindings.add(
                    &self.key,
                    vec![Key::Left],
                    Command::Move(Direction::Left).into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Right],
                    Command::Move(Direction::Right).into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Up],
                    Command::Move(Direction::Up).into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Down],
                    Command::Move(Direction::Down).into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('b')],
                    Command::MoveRel(Noun::Word, Rel::Prior).into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('e')],
                    Command::MoveRel(Noun::Word, Rel::End).into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('w')],
                    Command::MoveRel(Noun::Word, Rel::Next).into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('i')],
                    Command::SwitchMode(Mode::Insert).into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('h')],
                    Command::Move(Direction::Left).into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii(':')],
                    Command::FocusCommandLine.into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('j')],
                    Command::Move(Direction::Down).into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('k')],
                    Command::Move(Direction::Up).into(),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('l')],
                    Command::Move(Direction::Right).into(),
                );
                bindings.add(&self.key, vec![Key::Ascii('J')], Command::JoinLines.into());
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('o')],
                    DK::Sequence(vec![
                        DK::Command(Command::NewlineBelow),
                        DK::Command(Command::SwitchMode(Mode::Insert)),
                    ]),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('O')],
                    DK::Sequence(vec![
                        DK::Command(Command::NewlineAbove),
                        DK::Command(Command::MoveRel(Noun::Line, Rel::Beginning)),
                        DK::Command(Command::SwitchMode(Mode::Insert)),
                    ]),
                );

                bindings.add(
                    &self.key,
                    vec![Key::Ascii('x')],
                    DK::Command(Command::DeleteForwards),
                );
                bindings.add(
                    &self.key,
                    vec![Key::Ascii('X')],
                    DK::Command(Command::DeleteBackwards),
                );
            }
        }
        bindings
    }

    fn send_key(&mut self, key: Key) -> Result<Status> {
        match self.mode {
            Mode::Normal | Mode::Visual { .. } => Ok(Status::Message {
                message: format!("{:?} mode is ignoring {:?}...", self.mode, key),
                expiry: Instant::now() + Duration::from_millis(250),
            }),
            Mode::Insert => match key {
                Key::Ascii(ch) => self.insert_char(ch),
                /*Key::Backspace => self.delete_backwards(Noun::Char),*/
                _ => Err(not_impl!("[DocView::send_key] unhandled {:?}", key)),
            },
        }
    }
    fn execute_command(&mut self, command: Command) -> Result<Status> {
        match command {
            Command::Open { filename } => self.open(filename),
            Command::MoveRel(noun, rel) => self.move_cursor_rel(noun, rel),
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
            Command::SwitchMode(mode) => {
                self.switch_mode(mode);
                Ok(Status::Ok)
            }
            _ => Err(not_impl!(
                "DocView::execute_command needs to handle {:?}.",
                command
            )),
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
        } else if property == PROP_DOCVIEW_STATUS {
            Some(PropertyValue::String(format!(
                "| {:?} | {}:{} ",
                self.mode,
                self.cursor.y + 1,
                self.cursor.x + 1
            )))
        } else {
            log::trace!("DocView::get_property unhandled request for '{}'", property);
            None
        }
    }
}

impl DocView {
    pub fn new(view_key: ViewKey, plugin: PluginRef) -> Self {
        Self {
            parent: None,
            plugin,
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
