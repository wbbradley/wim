use crate::buf::{place_cursor, Buf, BLANKS};
use crate::consts::{
    PROP_CMDLINE_FOCUSED, PROP_DOCVIEW_CURSOR_POS, PROP_DOC_FILENAME, PROP_DOC_IS_MODIFIED,
};
use crate::dk::DK;
use crate::error::{Error, Result};
use crate::key::Key;
use crate::line::{line_fmt, Line};
use crate::status::Status;
use crate::types::{Coord, Pos, Rect, SafeCoordCast};
use crate::view::{View, ViewContext, ViewKey};

#[allow(dead_code)]
pub struct CommandLine {
    view_key: ViewKey,
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
            view_key: "command-line".to_string(),
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
        place_cursor(buf, self.frame.top_left());
        buf.append("\x1b[7m");
        let is_dirty = context.get_property_bool(PROP_DOC_IS_MODIFIED, false);
        let current_filename = context.get_property_string(PROP_DOC_FILENAME, "<no filename>");
        let cursor_pos = context.get_property_pos(PROP_DOCVIEW_CURSOR_POS);

        let mut stackbuf = [0u8; 1024];
        let mut formatted: &str = stackfmt::fmt_truncate(
            &mut stackbuf,
            format_args!(
                " {} {}|",
                current_filename,
                if is_dirty { "(modified) " } else { "" }
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
        place_cursor(
            buf,
            Pos {
                x: self.frame.x,
                y: self.frame.y + 1,
            },
        );
        let mut line: Line = Line::new(buf, self.frame.width);
        // TODO: render prompt...
        if context.get_property_bool(PROP_CMDLINE_FOCUSED, false) {
            line_fmt!(line, ":{}", self.text);
        }
    }

    fn dispatch_key(&mut self, key: Key) -> Result<DK> {
        match key {
            Key::Ascii(ch) => {
                self.text.push(ch);
                self.cursor += 1;
                Ok(DK::Noop)
            }
            _ => Err(Error::not_impl(format!(
                "command line doesn't yet support {} key",
                key
            ))),
        }
    }
    fn get_view_key(&self) -> &ViewKey {
        &self.view_key
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        Some(Pos {
            x: self.frame.x + 1 + self.cursor - self.scroll_offset,
            y: self.frame.y + 1,
        })
    }
}
impl ViewContext for CommandLine {}
