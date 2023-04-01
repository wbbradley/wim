use crate::bindings::Bindings;
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
use crate::view::ViewContext;
use mode::*;
// use std::fs::OpenOptions;
// use std::io::{Seek, SeekFrom, Write};

pub struct DocView {
    plugin: PluginRef,
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
        let (op, pos) = self.doc.split_newline(self.cursor);

        let mut change_tracker = self.doc.new_change_tracker(self.cursor);
        change_tracker.add_op(op, pos);
        self.cursor = change_tracker.commit();
        Ok(Status::Ok)
    }
    pub fn insert_newline_above(&mut self) -> Result<Status> {
        let op = self.doc.insert_newline(self.cursor.y);
        let mut change_tracker = self.doc.new_change_tracker(self.cursor);
        change_tracker.add_op(
            op,
            Pos {
                x: 0,
                y: self.cursor.y,
            },
        );
        self.cursor = change_tracker.commit();
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
        if let Some((op, pos)) = self.doc.delete_range(range) {
            let mut change_tracker = self.doc.new_change_tracker(self.cursor);
            change_tracker.add_op(op, pos);
            self.cursor = change_tracker.commit();
            Ok(Status::Ok)
        } else {
            Ok(Status::Ok)
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
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.plugin = plugin;
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
        let mut bindings: Bindings = Default::default();
        if matches!(self.mode, Mode::Normal | Mode::Visual { .. }) {
            bindings.insert("c", command("change-motion").at_view(vk));
            bindings.insert("d", command("delete-motion").at_view(vk));
            bindings.insert("y", command("yank-motion").at_view(vk));
            bindings.insert("h", command("move").arg("left").at_view(vk));
            bindings.insert("j", command("move").arg("down").at_view(vk));
            bindings.insert("k", command("move").arg("up").at_view(vk));
            bindings.insert("l", command("move").arg("right").at_view(vk));
            bindings.insert("e", command("move-rel").arg("word").arg("end").at_view(vk));
            bindings.insert("w", command("move-rel").arg("word").arg("next").at_view(vk));
            bindings.insert("J", command("join-lines").at_view(vk));
            bindings.insert(
                "b",
                command("move-rel").arg("word").arg("prior").at_view(vk),
            );
            bindings.insert("x", command("delete").at_view(vk));
        }
        if matches!(self.mode, Mode::Visual { .. } | Mode::NormalWithOp(_)) {
            bindings.insert(
                "iw",
                DK::Sequence(vec![
                    command("internal").at_view(vk),
                    command("word").at_view(vk),
                ]),
            );
            bindings.insert(
                "aw",
                DK::Sequence(vec![command("a").at_view(vk), command("word").at_view(vk)]),
            );
        }

        match self.mode {
            Mode::Visual { .. } => {
                bindings.insert(Key::Esc, command("switch-mode").arg("normal").at_view(vk));
            }
            Mode::Insert => {
                bindings.insert(Key::Esc, command("switch-mode").arg("normal").at_view(vk));
                bindings.insert("jk", DK::Key(Key::Esc));
                bindings.insert(Key::Backspace, command("delete-backwards").at_view(vk));
                bindings.insert(Key::Enter, command("newline").at_view(vk));
            }
            Mode::Normal => {
                bindings.insert("u", command("undo").at_view(vk));
                bindings.insert(
                    Key::Ctrl('u'),
                    command("move-rel")
                        .arg("line")
                        .arg("prior")
                        .arg(44)
                        .at_view(vk),
                );
                bindings.insert(
                    Key::Ctrl('d'),
                    command("move-rel")
                        .arg("line")
                        .arg("next")
                        .arg(44)
                        .at_view(vk),
                );
                bindings.insert("c", command("operator").arg("change").at_view(vk));
                bindings.insert("s", command("save").at_view(vk));
                bindings.insert(
                    "b",
                    command("move-rel").arg("word").arg("prior").at_view(vk),
                );
                bindings.insert("v", command("switch-mode").arg("visual").at_view(vk));
                bindings.insert("i", command("switch-mode").arg("insert").at_view(vk));
                bindings.insert(
                    ":",
                    command("focus")
                        .arg(Target::Named("command-line".to_string()))
                        .at_view_map(),
                );
                bindings.insert("J", command("join-lines").at_view(vk));
                bindings.insert(
                    "o",
                    DK::Sequence(vec![
                        command("newline").arg("below").at_view(vk),
                        command("switch-mode").arg("insert").at_view(vk),
                    ]),
                );
                bindings.insert(
                    "O",
                    DK::Sequence(vec![
                        command("newline").arg("above").at_view(vk),
                        command("move-rel").arg("line").arg("begin").at_view(vk),
                        command("switch-mode").arg("insert").at_view(vk),
                    ]),
                );
                bindings.insert(
                    "x",
                    command("delete-rel").arg("char").arg("next").at_view(vk),
                );
                bindings.insert(
                    "X",
                    command("delete-rel").arg("char").arg("prior").at_view(vk),
                );
            }
            Mode::NormalWithOp(_) => {}
        }
        bindings
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
        }

        match (self.mode, name.as_ref()) {
            (Mode::Normal, "undo") => {
                if let Some(pos) = self.doc.pop_change() {
                    self.cursor = pos;
                    Ok(Status::Ok)
                } else {
                    Ok(status!("Nothing left to undo."))
                }
            }
            (Mode::Normal, "change" | "yank" | "delete") => {
                self.mode = Mode::NormalWithOp(Op::from_str(name.as_ref())?);
                Ok(Status::Ok)
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
                "DocView::execute_command needs to handle {:?} {:?}.",
                name,
                args
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
            plugin,
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
    pub enum Mode {
        Insert,
        Visual(VisualMode),
        Normal,
        NormalWithOp(Op),
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
