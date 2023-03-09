use crate::buf::{buf_fmt, Buf, BLANKS};
use crate::status::Status;
use crate::types::{Coord, Pos, Rect, SafeCoordCast};
use crate::view::{View, ViewContext};

#[allow(dead_code)]
pub struct CommandLine {
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
            cursor: 0,
            render_cursor: 0,
            scroll_offset: 0,
            text: String::new(),
            frame: Rect::zero(),
            status: Status::Cleared,
        }
    }
    pub fn set_status(&mut self, status: Status) {
        log::trace!("Status Updated: {:?}", &status);
        self.status = status;
    }
}

impl View for CommandLine {
    fn layout(&mut self, frame: Rect) {
        self.frame = frame;
    }

    fn display(&self, buf: &mut Buf, context: &dyn ViewContext) {
        buf_fmt!(buf, "\x1b[{};{}H", self.frame.y + 1, self.frame.x + 1);
        buf.append("\x1b[7m");
        let is_dirty = context.get_property_bool("doc-is-modified?", false);
        let current_filename = context.get_property_string("doc-filename", "<no filename>");
        let cursor_pos = context.get_property_pos("docview-cursor-pos");

        let mut stackbuf = [0u8; 1024];
        let mut formatted: &str = stackfmt::fmt_truncate(
            &mut stackbuf,
            format_args!(
                "{} |{}",
                current_filename,
                if is_dirty { " (modified)" } else { "" }
            ),
        );
        buf.append(formatted);
        let mut remaining_len = self.frame.width - formatted.len().as_coord();
        if let Some(cursor_pos) = cursor_pos {
            formatted = stackfmt::fmt_truncate(
                &mut stackbuf,
                format_args!("| {}:{} ", cursor_pos.y + 1, cursor_pos.x + 1),
            );
            remaining_len -= formatted.len().as_coord();
        }
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
impl ViewContext for CommandLine {}
