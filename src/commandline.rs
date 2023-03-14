use crate::buf::{place_cursor, Buf};
use crate::command::Command;
use crate::consts::{
    PROP_CMDLINE_FOCUSED, PROP_DOCVIEW_STATUS, PROP_DOC_FILENAME, PROP_DOC_IS_MODIFIED,
};
use crate::dk::DK;
use crate::error::{Error, Result};
use crate::key::Key;
use crate::line::{line_fmt, Line};
use crate::plugin::PluginRef;
use crate::status::Status;
use crate::types::{Coord, Pos, Rect};
use crate::view::{View, ViewContext, ViewKey};
use std::time::Instant;

#[allow(dead_code)]
pub struct CommandLine {
    plugin: PluginRef,
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
    pub fn new(plugin: PluginRef) -> Self {
        Self {
            plugin,
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
        log::trace!("[CommandLine] Status Updated: {:?}", &status);
        self.status = status;
    }
}

impl View for CommandLine {
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.plugin = plugin;
    }
    fn layout(&mut self, frame: Rect) {
        self.frame = frame;
    }

    fn display(&self, buf: &mut Buf, context: &dyn ViewContext) {
        place_cursor(buf, self.frame.top_left());
        buf.append("\x1b[7m");
        let is_dirty = context.get_property_bool(PROP_DOC_IS_MODIFIED, false);
        let current_filename = context.get_property_string(PROP_DOC_FILENAME, "<no filename>");
        let status_text = context.get_property_string(PROP_DOCVIEW_STATUS, "");
        log::trace!("PROP_DOCVIEW_STATUS={}", status_text);
        {
            let mut line: Line = Line::new(buf, self.frame.width);
            line_fmt!(
                line,
                " {} {}|",
                current_filename,
                if is_dirty { "(modified) " } else { "" }
            );
            if let Status::Message {
                ref message,
                expiry,
            } = self.status
            {
                if expiry > Instant::now() {
                    line_fmt!(line, " {}", message);
                }
            }
            line.end_with(&status_text);
        }

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

    fn handle_key(&mut self, key: Key) -> Result<DK> {
        match key {
            Key::Ascii(ch) => {
                self.text.push(ch);
                self.cursor += 1;
                DK::Noop.into()
            }
            Key::Backspace => {
                if !self.text.is_empty() {
                    self.text.pop();
                    self.cursor -= 1;
                }
                DK::Noop.into()
            }
            Key::Esc => Command::FocusPrevious.into(),
            Key::Enter => {
                log::trace!("TODO: run command '{}'", self.text);

                Ok(Command::Many(vec![
                    Command::FocusPrevious,
                    Command::Execute(self.text.clone()),
                ])
                .into())
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
