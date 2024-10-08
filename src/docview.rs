use crate::bindings::{Bindings, BindingsBuilder};
use crate::consts::{
    PROP_DOCVIEW_CURSOR_POS, PROP_DOCVIEW_STATUS, PROP_DOC_FILENAME, PROP_DOC_IS_MODIFIED,
};
use crate::doc::Doc;
use crate::error::{ensure, Result};
use crate::plugin::PluginRef;
use crate::prelude::*;
use crate::rel::Rel;
use crate::status::Status;
use crate::types::{Coord, Pos, Rect, RelCoord};
use crate::undo::ChangeOp;
use crate::view::ViewContext;
use mode::*;
// use std::fs::OpenOptions;
// use std::io::{Seek, SeekFrom, Write};

pub struct DocView {
    _plugin: PluginRef,
    key: ViewKey,
    cursor: Pos,
    sel: Option<Sel>,
    render_cursor_x: Coord,
    doc: Doc,
    scroll_offset: Pos,
    mode: Mode,
}

#[allow(dead_code)]
impl DocView {
    pub fn scroll(&mut self, size: Size) {
        if self.cursor.y < self.scroll_offset.y {
            self.scroll_offset.y = self.cursor.y;
        }
        if self.cursor.y >= self.scroll_offset.y + size.height {
            self.scroll_offset.y = self.cursor.y - size.height + 1;
        }
        if self.render_cursor_x < self.scroll_offset.x {
            self.scroll_offset.x = self.render_cursor_x;
        }
        if self.render_cursor_x >= self.scroll_offset.x + size.width {
            self.scroll_offset.x = self.render_cursor_x - size.width + 1;
        }
    }
    pub fn move_cursor(&mut self, x: RelCoord, y: RelCoord) -> Result<Status> {
        self.cursor.y = (self.cursor.y as RelCoord + y).clamp(0, RelCoord::MAX) as Coord;
        self.cursor.x = (self.cursor.x as RelCoord + x).clamp(0, RelCoord::MAX) as Coord;
        if matches!(self.mode, Mode::Visual(VisualMode::Char)) {
            if let Some(ref mut sel) = self.sel {
                sel.end = self.cursor;
            } else {
                panic!("There should be a sel right now!");
            }
        }
        self.clamp_cursor();
        Ok(Status::Ok)
    }
    pub fn get_rel_cursor_pos(&self, x: RelCoord, y: RelCoord) -> Pos {
        self.clamped_pos(Pos {
            y: (self.cursor.y as RelCoord + y).clamp(0, RelCoord::MAX) as Coord,
            x: (self.cursor.x as RelCoord + x).clamp(0, RelCoord::MAX) as Coord,
        })
    }

    pub fn do_op_to_range(&mut self, op: Op, range: impl RangeBounds<Pos>) -> Result<Status> {
        match op {
            Op::Delete => {
                let ret = self.delete_range(range);
                self.switch_mode(Mode::Normal);
                ret
            }
            Op::Change => {
                let ret = self.delete_range(range);
                self.switch_mode(Mode::Insert);
                ret
            }
            Op::Yank => {
                panic!("yank range not impl");
            }
        }
    }

    pub fn do_op_rel(&mut self, op: Op, noun: Noun, rel: Rel) -> Result<Status> {
        trace!("do_op_rel({:?}, {:?}, {:?})", op, noun, rel);
        let end_pos: Option<Pos> = match (noun, rel) {
            (Noun::Char, Rel::Prior) => Some(self.get_rel_cursor_pos(-1, 0)),
            (Noun::Char, Rel::Next) => Some(self.get_rel_cursor_pos(1, 0)),
            (Noun::Line, Rel::Prior) => Some(self.get_rel_cursor_pos(0, -1)),
            (Noun::Line, Rel::Next) => Some(self.get_rel_cursor_pos(0, 1)),
            (Noun::Word, Rel::Next) => self.doc.get_next_word_pos(self.cursor),
            (Noun::Word, Rel::Prior) => self.doc.get_prior_word_pos(self.cursor),
            (Noun::Word, Rel::End) => self.doc.get_word_end(self.cursor),
            _ => {
                return Err(not_impl!(
                    "DocView: Don't know how to handle relative motion for ({:?}, {:?}).",
                    noun,
                    rel
                ));
            }
        };
        if let Some(end_pos) = end_pos {
            self.do_op_to_range(op, self.cursor..=end_pos)
        } else {
            Err(error!("couldn't get an end pos?!"))
        }
    }
    pub fn move_cursor_rel(&mut self, noun: Noun, rel: Rel) -> Result<Status> {
        trace!("move_cursor_rel({:?}, {:?})", noun, rel);
        match (noun, rel) {
            (Noun::Char, Rel::Prior) => self.move_cursor(-1, 0),
            (Noun::Char, Rel::Next) => self.move_cursor(1, 0),
            (Noun::Line, Rel::Prior) => self.move_cursor(0, -1),
            (Noun::Line, Rel::Next) => self.move_cursor(0, 1),
            (Noun::Word, Rel::Next) => {
                self.jump_cursor_pos(self.doc.get_next_word_pos(self.cursor));
                Ok(Status::Ok)
            }
            (Noun::Word, Rel::Prior) => {
                if let Some(pos) = self.doc.get_prior_word_pos(self.cursor) {
                    self.jump_cursor(Some(pos.x), Some(pos.y));
                }
                Ok(Status::Ok)
            }
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
    pub fn clamped_pos(&self, mut pos: Pos) -> Pos {
        pos.y = pos.y.clamp(0, self.last_valid_row());
        if let Some(row) = self.doc.get_row(pos.y) {
            pos.x = pos.x.clamp(
                0,
                row.len() - usize::from(!row.is_empty() && self.mode == Mode::Normal),
            );
        } else {
            pos.x = 0;
        };
        pos
    }
    fn clamp_cursor(&mut self) {
        log::trace!("clamp_cursor starts at {:?}", self.cursor);
        self.cursor.y = self.cursor.y.clamp(0, self.last_valid_row());
        if let Some(row) = self.doc.get_row(self.cursor.y) {
            self.cursor.x = self.cursor.x.clamp(
                0,
                row.len() - usize::from(!row.is_empty() && self.mode == Mode::Normal),
            );
            self.render_cursor_x = row.cursor_to_render_col(self.cursor.x);
        } else {
            self.cursor.x = 0;
            self.render_cursor_x = 0;
        };
        log::trace!("clamp_cursor ends at {:?}", self.cursor);
    }
    fn apply_op_pos(&mut self, op_pos: (ChangeOp, Pos)) -> Result<Status> {
        let (op, pos) = op_pos;
        let mut change_tracker = self.doc.new_change_tracker(self.cursor);
        change_tracker.add_op(op, pos);
        let cursor = Some(change_tracker.commit());
        self.jump_cursor_pos(cursor);
        Ok(Status::Ok)
    }
    pub fn jump_cursor_pos(&mut self, pos: Option<Pos>) {
        if let Some(pos) = pos {
            self.jump_cursor(Some(pos.x), Some(pos.y));
        }
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
        Err(not_impl!("come back"))
        /*
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
        */
    }
    pub fn split_newline(&mut self) -> Result<Status> {
        self.apply_op_pos(self.doc.split_newline(self.cursor))
    }
    pub fn insert_newline_above(&mut self) -> Result<Status> {
        let op = self.doc.insert_newline(self.cursor.y);
        let pos = Pos {
            x: 0,
            y: self.cursor.y,
        };
        self.apply_op_pos((op, pos))
    }
    pub fn insert_newline_below(&mut self) -> Result<Status> {
        let op = self.doc.insert_newline(self.cursor.y + 1);
        let pos = Pos {
            x: 0,
            y: self.cursor.y + 1,
        };
        self.apply_op_pos((op, pos))
    }
    pub fn insert_char(&mut self, ch: char) -> Result<Status> {
        self.apply_op_pos(self.doc.insert_char(self.cursor, ch))
    }
    pub fn delete_sel(&mut self) -> Result<Status> {
        if let Some(sel) = self.sel {
            let ret = self.delete_range(sel.start..=sel.end);
            self.switch_mode(Mode::Normal);
            ret
        } else {
            Err(error!("invalid sel?!"))
        }
    }
    pub fn delete_rel(&mut self, noun: Noun, rel: Rel) -> Result<Status> {
        let (start, end) = self.doc.find_range(self.cursor, noun, rel);
        self.delete_range(start..end)
    }
    fn delete_range(&mut self, range: impl RangeBounds<Pos>) -> Result<Status> {
        match self.doc.delete_range(range) {
            Some(op_pos) => self.apply_op_pos(op_pos),
            None => Ok(Status::Ok),
        }
    }
    pub fn join_line(&mut self) -> Result<Status> {
        self.delete_range(
            Pos {
                x: self.doc.get_row(self.cursor.y).unwrap().len(),
                y: self.cursor.y,
            }..Pos {
                x: 0,
                y: self.cursor.y + 1,
            },
        )
    }
    /*
    pub fn write<T>(&self, buf: std::buf::BufWriter<W>)
    where
        W: Write,
    {
        let mut buf = Buf::default();
        for row in self.doc.iter_lines() {
            buf.append(row);
            buf.append("\n");
        }
        buf
    }
    */
    fn switch_mode(&mut self, mode: Mode) {
        self.mode = mode;
        if matches!(self.mode, Mode::Visual(VisualMode::Char)) {
            self.sel = Some(Sel::from_pos(self.cursor));
        } else {
            self.sel = None;
        }
        self.clamp_cursor();
    }
    fn get_line_fmt_spans(
        &self,
        mut screen_pos: Pos,
        mut render_start: Pos,
        width: usize,
    ) -> Vec<Span> {
        let mut spans = Vec::new();
        if let Some(sel) = self.sel {
            let (mut start, mut end) = (sel.start, sel.end);
            if start > end {
                std::mem::swap(&mut start, &mut end);
                end.x += 1;
            }
            if start.y == render_start.y {
                if end.y == render_start.y {
                    if start.x > render_start.x {
                        let chars = self.doc.render_line_slice(
                            render_start,
                            std::cmp::min(start.x - render_start.x, width - screen_pos.x),
                        );
                        spans.push(Span {
                            screen_pos,
                            chars,
                            format: Format::none(),
                        });
                        screen_pos.x += chars.len();
                        render_start.x += chars.len();
                    }
                    if screen_pos.x < width && end.x > render_start.x {
                        // Span on a single line.
                        let chars = self.doc.render_line_slice(
                            render_start,
                            std::cmp::min(end.x - render_start.x, width - screen_pos.x),
                        );
                        spans.push(Span {
                            screen_pos,
                            chars,
                            format: Format::selected(),
                        });
                        screen_pos.x += chars.len();
                        render_start.x += chars.len();
                    }
                    if screen_pos.x < width {
                        // Span on a single line.
                        let chars = self
                            .doc
                            .render_line_slice(render_start, width - screen_pos.x);
                        spans.push(Span {
                            screen_pos,
                            chars,
                            format: Format::none(),
                        });
                        screen_pos.x += chars.len();
                        render_start.x += chars.len();
                    }
                }
                return spans;
            }
        }
        vec![Span {
            screen_pos,
            chars: self.doc.render_line_slice(render_start, width),
            format: Format::none(),
        }]
    }
}
struct Span<'a> {
    screen_pos: Pos,
    chars: &'a [char],
    format: Format,
}

impl View for DocView {
    fn get_doc_text(&self, _view_map: &ViewMap) -> Option<String> {
        Some(self.doc.to_string())
    }
    fn install_plugins(&mut self, plugin: PluginRef) {
        self._plugin = plugin;
    }
    fn layout(&mut self, _view_map: &ViewMap, size: Size) -> Vec<(ViewKey, Rect)> {
        log::trace!("docview size is {:?}", size);
        self.scroll(size);
        vec![]
    }
    fn display(&self, _view_map: &ViewMap, bmp: &mut BitmapView) {
        let mut y = 0;
        let size = bmp.get_size();
        let offset_line_count = if self.scroll_offset.y >= self.doc.line_count() {
            self.doc.line_count()
        } else {
            self.doc.line_count() - self.scroll_offset.y
        };
        loop {
            if y >= size.height || y >= offset_line_count {
                break;
            }
            let spans = self.get_line_fmt_spans(
                Pos { x: 0, y },
                Pos {
                    x: self.scroll_offset.x,
                    y: self.scroll_offset.y + y,
                },
                size.width,
            );

            for span in spans {
                bmp.append_chars_at(span.screen_pos, span.chars.iter().copied(), span.format);
            }
            y += 1;
        }
        loop {
            if y >= size.height {
                break;
            }

            bmp.set_glyph(Pos { x: 0, y }, '~'.into());
            y += 1;
        }
    }

    fn get_view_key(&self) -> ViewKey {
        self.key
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        Some(Pos {
            x: self.render_cursor_x - self.scroll_offset.x,
            y: self.cursor.y - self.scroll_offset.y,
        })
    }
}
impl DispatchTarget for DocView {
    fn get_key_bindings(&self) -> Bindings {
        let vk = self.get_view_key();
        let mut builder = BindingsBuilder::new(vk);
        if matches!(self.mode, Mode::Normal | Mode::Visual { .. }) {
            builder.insert("c", command("motion").arg("change"));
            builder.insert("d", command("motion").arg("delete"));
            builder.insert("y", command("motion").arg("yank"));
            builder.insert("h", command("move").arg("left"));
            builder.insert("j", command("move").arg("down"));
            builder.insert("k", command("move").arg("up"));
            builder.insert("l", command("move").arg("right"));
            builder.insert("e", command("move-rel").arg("word").arg("end"));
            builder.insert("w", command("move-rel").arg("word").arg("next"));
            builder.insert("J", command("join-lines"));
            builder.insert("b", command("move-rel").arg("word").arg("prior"));
        }

        if matches!(self.mode, Mode::Visual { .. } | Mode::NormalWithOp(_)) {
            builder.insert("i", command("inner"));
            builder.insert("a", command("a"));
        }

        match self.mode {
            Mode::NormalWithOp(op) => {
                builder.insert(Key::Esc, command("switch-mode").arg("normal"));
                builder.insert(op.as_str(), command("line"));
                builder.insert("h", command("move-rel").arg("char").arg("prior"));
                builder.insert("j", command("move-rel").arg("line").arg("next"));
                builder.insert("k", command("move-rel").arg("line").arg("prior"));
                builder.insert("l", command("move-rel").arg("char").arg("next"));
                builder.insert("e", command("move-rel").arg("word").arg("end"));
                builder.insert("w", command("move-rel").arg("word").arg("end"));
                builder.insert("b", command("move-rel").arg("word").arg("prior"));
                builder.insert("J", command("join-lines"));
            }
            Mode::NormalWithOpObjMode(_op, _obj_mode) => {
                builder.insert("w", command("word"));
            }
            Mode::Visual { .. } => {
                builder.insert(Key::Esc, command("switch-mode").arg("normal"));
                builder.insert("x", command("delete"));
            }
            Mode::Insert => {
                builder.insert(Key::Esc, command("switch-mode").arg("normal"));
                builder.insert("jk", DK::Key(Key::Esc));
                builder.insert(Key::Backspace, command("delete-backwards"));
                builder.insert(Key::Enter, command("newline"));
            }
            Mode::Normal => {
                builder.insert("u", command("undo"));
                builder.insert(Key::Ctrl('r'), command("redo"));
                builder.insert(
                    Key::Ctrl('u'),
                    command("move-rel").arg("line").arg("prior").arg(44),
                );
                builder.insert(
                    Key::Ctrl('d'),
                    command("move-rel").arg("line").arg("next").arg(44),
                );
                builder.insert("s", command("save"));
                builder.insert("v", command("switch-mode").arg("visual"));
                builder.insert("i", command("switch-mode").arg("insert"));
                builder.insert(
                    ":",
                    command("focus")
                        .arg(Target::Named("command-line".to_string()))
                        .at_view_map(),
                );
                builder.insert(
                    "o",
                    DK::Sequence(vec![
                        command("newline").arg("below").at_view(vk),
                        command("switch-mode").arg("insert").at_view(vk),
                    ]),
                );
                builder.insert(
                    "O",
                    DK::Sequence(vec![
                        command("newline").arg("above").at_view(vk),
                        command("move-rel").arg("line").arg("begin").at_view(vk),
                        command("switch-mode").arg("insert").at_view(vk),
                    ]),
                );
                builder.insert("x", command("delete-rel").arg("char").arg("next"));
                builder.insert("X", command("delete-rel").arg("char").arg("prior"));
            }
        }
        builder.get_bindings()
    }
    fn send_key(&mut self, key: Key) -> Result<Status> {
        match self.mode {
            Mode::Normal | Mode::Visual { .. } => Ok(Status::Message {
                message: format!("No mapping found for {:?} in {:?} mode.", key, self.mode),
                expiry: Instant::now() + Duration::from_millis(5000),
            }),
            Mode::Insert => match key {
                Key::Utf8(ch) => self.insert_char(ch),
                _ => Ok(Status::Message {
                    message: format!("No mapping found for {:?} in {:?} mode.", key, self.mode),
                    expiry: Instant::now() + Duration::from_millis(2500),
                }),
            },
            _ => Err(error!(
                "don't yet know how to handle send_keys in mode '{:?}'",
                self.mode
            )),
        }
    }
    fn execute_command(&mut self, name: String, mut args: Vec<Variant>) -> Result<Status> {
        if name.as_str() == "switch-mode" {
            ensure!(args.len() == 1);
            if let Variant::String(arg) = args.remove(0) {
                self.switch_mode(Mode::from_str(&arg)?);
                return Ok(Status::Ok);
            } else {
                return Err(error!("'switch-mode' expects a string"));
            }
        } else if name.as_str() == "quit" {
            return Ok(Status::Quit);
        }

        match (self.mode, name.as_ref()) {
            (Mode::Normal, "motion") => {
                if let Some(Variant::String(op)) = args.first() {
                    self.mode = Mode::NormalWithOp(Op::from_str(op)?);
                }
                Ok(Status::Ok)
            }
            (Mode::Normal, "undo") => {
                let pos = self.doc.undo_change();
                self.jump_cursor_pos(pos);
                if pos.is_none() {
                    Ok(status!("Nothing left to undo."))
                } else {
                    Ok(Status::Ok)
                }
            }
            (Mode::Normal, "redo") => {
                let pos = self.doc.redo_change();
                self.jump_cursor_pos(pos);
                if pos.is_none() {
                    Ok(status!("Nothing left to redo."))
                } else {
                    Ok(Status::Ok)
                }
            }
            (_mode, "switch-mode") => {
                ensure!(args.len() == 1);
                if let Variant::String(arg) = args.remove(0) {
                    self.switch_mode(Mode::from_str(&arg)?);
                    Ok(Status::Ok)
                } else {
                    Err(error!("'switch-mode' expects a string"))
                }
            }
            (_, "join-lines") => {
                ensure!(args.is_empty());
                self.join_line()
            }
            (_, "delete-backwards") => {
                ensure!(args.is_empty());
                self.delete_rel(Noun::Char, Rel::Prior)
            }
            (_, "delete-forwards") => {
                ensure!(args.is_empty());
                self.delete_rel(Noun::Char, Rel::Next)
            }
            (Mode::Visual(VisualMode::Char), "delete") => self.delete_sel(),
            (Mode::Normal, "open") => {
                ensure!(args.len() == 1);
                if let Variant::String(arg) = args.remove(0) {
                    self.open(arg)
                } else {
                    Err(error!("'open' expects a filename"))
                }
            }
            (Mode::Normal | Mode::Visual(VisualMode::Char), "move") => {
                ensure!(args.len() == 1);
                if let Variant::String(arg) = args.remove(0) {
                    match arg.as_str() {
                        "up" => self.move_cursor(0, -1),
                        "down" => self.move_cursor(0, 1),
                        "left" => self.move_cursor(-1, 0),
                        "right" => self.move_cursor(1, 0),
                        _ => Err(error!("'move' expects one of {{up,down,left,right}}")),
                    }
                } else {
                    Err(error!("'move' expects a direction"))
                }
            }
            (_, "newline") => match args.as_slice() {
                [] => self.split_newline(),
                [Variant::String(arg)] => match arg.as_str() {
                    "above" => self.insert_newline_above(),
                    "below" => self.insert_newline_below(),
                    _ => Err(error!("'newline' expects one of {{above,below}}")),
                },
                _ => Err(error!(
                    "'newline' encountered unexpected options. [args={:?}]",
                    args
                )),
            },
            (Mode::NormalWithOp(op), "move-rel") => {
                ensure!((2..=3).contains(&args.len()));
                let (noun, rel, count) = pull_noun_rel_count(args)?;
                for _ in 0..count {
                    self.do_op_rel(op, noun, rel)?;
                }
                Ok(Status::Ok)
            }
            (_, "move-rel") => {
                ensure!((2..=3).contains(&args.len()));
                let (noun, rel, count) = pull_noun_rel_count(args)?;
                for _ in 0..count {
                    self.move_cursor_rel(noun, rel)?;
                }
                Ok(Status::Ok)
            }
            (Mode::Normal, "delete-rel") => {
                let (noun, rel, count) = pull_noun_rel_count(args)?;
                for _ in 0..count {
                    self.delete_rel(noun, rel)?;
                }
                Ok(Status::Ok)
            }
            _ => Err(not_impl!(
                "DocView::execute_command needs to handle {:?} {:?} in mode {:?}.",
                name,
                args,
                self.mode
            )),
        }
    }
}

fn pull_noun_rel_count(args: Vec<Variant>) -> Result<(Noun, Rel, i64)> {
    let (noun, rel, count) = match args.as_slice() {
        [Variant::String(noun), Variant::String(rel), Variant::Int(count)] => (noun, rel, *count),
        [Variant::String(noun), Variant::String(rel)] => (noun, rel, 1),
        _ => return Err(error!("'delete-rel' expects a pair (noun, rel)")),
    };

    match (Noun::from_str(noun), Rel::from_str(rel)) {
        (Ok(noun), Ok(rel)) => Ok((noun, rel, count)),
        _ => Err(error!("'move-rel' expects a pair (noun, rel)")),
    }
}

impl ViewContext for DocView {
    fn get_property(&self, property: &str) -> Option<Variant> {
        if property == PROP_DOC_IS_MODIFIED {
            Some(Variant::Bool(self.doc.is_dirty()))
        } else if property == PROP_DOC_FILENAME {
            self.doc
                .get_filename()
                .map(|filename| Variant::String(filename.to_string()))
        } else if property == PROP_DOCVIEW_CURSOR_POS {
            Some(Variant::Pos(self.cursor))
        } else if property == PROP_DOCVIEW_STATUS {
            Some(Variant::String(format!(
                "▐ {:?} ▐ {}:{} ",
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
            _plugin: plugin,
            key: view_key,
            cursor: Default::default(),
            sel: None,
            render_cursor_x: 0,
            doc: Doc::empty(),
            scroll_offset: Default::default(),
            mode: Mode::Normal,
        }
    }
}

mod mode {
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    #[allow(dead_code)]
    pub enum ObjMod {
        Inner,
        A,
    }
    #[allow(dead_code)]
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Mode {
        Insert,
        Visual(VisualMode),
        Normal,
        NormalWithOp(Op),
        #[allow(clippy::enum_variant_names)]
        NormalWithOpObjMode(Op, ObjMod),
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum TextObj {}

    impl std::str::FromStr for Mode {
        type Err = crate::error::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "insert" => Ok(Self::Insert),
                "normal" => Ok(Self::Normal),
                "visual" => Ok(Self::Visual(VisualMode::Char)),
                "visual-line" => Ok(Self::Visual(VisualMode::Line)),
                "visual-block" => Ok(Self::Visual(VisualMode::Block)),
                missing => Err(Self::Err::new(format!(
                    "{} is not a valid editing Mode",
                    missing
                ))),
            }
        }
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum Op {
        Change,
        Delete,
        Yank,
    }

    impl Op {
        pub fn as_str(&self) -> &'static str {
            match self {
                Self::Change => "c",
                Self::Delete => "d",
                Self::Yank => "y",
            }
        }
    }

    impl std::str::FromStr for Op {
        type Err = crate::error::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "change" => Ok(Self::Change),
                "delete" => Ok(Self::Delete),
                "yank" => Ok(Self::Yank),
                missing => Err(Self::Err::new(format!("{} is not a valid Op", missing))),
            }
        }
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub enum VisualMode {
        Char,
        Line,
        Block,
    }

    impl std::str::FromStr for VisualMode {
        type Err = crate::error::Error;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "char" => Ok(Self::Char),
                "line" => Ok(Self::Line),
                "block" => Ok(Self::Block),
                missing => Err(Self::Err::new(format!(
                    "{} is not a valid visual Mode",
                    missing
                ))),
            }
        }
    }
}
