use crate::buf::place_cursor;
use crate::error::{Error, Result};
use crate::line::{line_fmt, Line};
use crate::prelude::*;

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
    pub fn new(plugin: PluginRef, view_key: ViewKey) -> Self {
        Self {
            plugin,
            view_key,
            cursor: 0,
            render_cursor: 0,
            scroll_offset: 0,
            text: String::new(),
            frame: Rect::zero(),
            status: Status::Ok,
        }
    }
}

impl View for CommandLine {
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.plugin = plugin;
    }
    fn layout(&mut self, _view_map: &ViewMap, frame: Rect) -> Vec<(ViewKey, Rect)> {
        self.frame = frame;
        Default::default()
    }
    fn display(&self, view_map: &ViewMap, buf: &mut Buf) {
        place_cursor(buf, self.frame.top_left());
        buf.append("\x1b[7m");
        let is_dirty = view_map
            .focused_view()
            .get_property_bool(PROP_DOC_IS_MODIFIED, false);
        let current_filename = view_map
            .focused_view()
            .get_property_string(PROP_DOC_FILENAME);
        let status_text = view_map
            .focused_view()
            .get_property_string(PROP_DOCVIEW_STATUS);
        log::trace!("PROP_DOCVIEW_STATUS={:?}", status_text);
        {
            let mut line: Line = Line::new(buf, self.frame.width);
            if let Some(current_filename) = current_filename {
                line_fmt!(
                    line,
                    " {} {}|",
                    current_filename,
                    if is_dirty { "(modified) " } else { "" }
                );
            }
            if let Status::Message {
                ref message,
                expiry,
            } = self.status
            {
                if expiry > Instant::now() {
                    line_fmt!(line, " {}", message);
                }
            }
            if let Some(status_text) = status_text {
                line.end_with(&status_text);
            }
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
        if view_map.focused_view_key() == self.view_key {
            line_fmt!(line, ":{}", self.text);
        }
    }

    fn get_view_key(&self) -> ViewKey {
        self.view_key
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        Some(Pos {
            x: self.frame.x + 1 + self.cursor - self.scroll_offset,
            y: self.frame.y + 1,
        })
    }
    fn set_status(&mut self, status: Status) {
        log::trace!("[CommandLine] Status Updated: {:?}", &status);
        self.status = status;
    }
}

impl DispatchTarget for CommandLine {
    fn get_key_bindings(&self) -> Bindings {
        let vk = self.get_view_key();
        let mut bindings: Bindings = Default::default();
        bindings.insert(Key::Esc, command("focus").arg(Target::Previous).at_view(vk));
        bindings.insert(
            Key::Enter,
            DK::Sequence(vec![
                command("clear-text").at_view(vk),
                command("focus").arg(Target::Previous).at_view_map(),
                command("invoke-execute")
                    .arg(self.text.as_ref())
                    .at_focused(),
            ]),
        );
        bindings
    }

    fn send_key(&mut self, key: Key) -> Result<Status> {
        match key {
            Key::Utf8(ch) => {
                self.text.push(ch);
                self.cursor += 1;
                Ok(Status::Ok)
            }
            Key::Backspace => {
                if !self.text.is_empty() {
                    self.text.pop();
                    self.cursor -= 1;
                }
                Ok(Status::Ok)
            }
            _ => {
                panic!("[CommandLine::send_key] unhandled key {:?}.", key);
            }
        }
    }
    fn execute_command(&mut self, name: String, args: Vec<Variant>) -> Result<Status> {
        if name == "clear-text" {
            self.text.clear();
            Ok(Status::Ok)
        } else {
            Err(Error::not_impl(format!(
                "CommandLine::execute_command does not impl {:?} {:?}",
                name, args
            )))
        }
    }
}

impl ViewContext for CommandLine {
    fn get_property(&self, _property: &str) -> Option<Variant> {
        None
    }
}
