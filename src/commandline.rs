use crate::buf::{buf_fmt, Buf, BLANKS};
use crate::status::Status;
use crate::types::{Coord, Pos, Rect, SafeCoordCast};
use crate::view::{View, ViewKey};

#[allow(dead_code)]
pub struct CommandLine {
    current_filename: Option<String>,
    current_view_key: Option<ViewKey>,
    cursor: Coord,
    render_cursor: Coord,
    scroll_offset: Coord,
    text: String,
    frame: Rect,
    status: Status,
}

#[allow(dead_code)]
impl CommandLine {
    pub fn new() -> Self {
        Self {
            current_filename: None,
            current_view_key: None,
            cursor: 0,
            render_cursor: 0,
            scroll_offset: 0,
            text: String::new(),
            frame: Rect::zero(),
            status: Status::None,
        }
    }
    pub fn set_status(&mut self, status: Status) {
        log::trace!("Status Updated: {:?}", &status);
        self.status = status;
    }
    pub fn set_cur_info(&mut self, filename: Option<String>, view_key: Option<ViewKey>) {
        self.current_filename = filename;
        self.current_view_key = view_key;
    }
}

impl View for CommandLine {
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
