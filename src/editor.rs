use crate::bindings::Bindings;
use crate::commandline::CommandLine;
use crate::docview::DocView;
use crate::error::Result;
use crate::plugin::PluginRef;
use crate::prelude::*;
use crate::status::Status;
use crate::types::{Pos, Rect};
use crate::view::ViewContext;

#[allow(dead_code)]
pub struct Editor {
    plugin: PluginRef,
    should_quit: bool,
    last_key: Option<Key>,
    view_key: ViewKey,
    top_view_key: Option<ViewKey>,
    command_line_key: ViewKey,
    frame: Rect,
}

impl ViewContext for Editor {
    fn get_property(&self, property: &str) -> Option<Variant> {
        if property == PROP_EDITOR_SHOULD_QUIT {
            Some(self.get_should_quit().into())
        } else {
            panic!("unhandled get_property '{}'", property);
        }
    }
}
impl View for Editor {
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.plugin = plugin;
    }
    fn layout(&mut self, _view_map: &ViewMap, frame: Rect) -> Vec<(ViewKey, Rect)> {
        self.frame = frame;
        let mut ret = Vec::new();

        if let Some(top_view_key) = self.top_view_key {
            ret.push((
                top_view_key,
                Rect {
                    x: 0,
                    y: 0,
                    width: frame.width,
                    height: frame.height - 2,
                },
            ));
        } else {
            trace!("there is no top_view_key in the editor");
        }
        ret.push((
            self.command_line_key,
            Rect {
                x: 0,
                y: frame.height - 2,
                width: frame.width,
                height: 2,
            },
        ));
        ret
    }

    fn display(&self, view_map: &ViewMap, buf: &mut BitmapView) {
        // Hide the cursor.
        buf.append("\x1b[?25l");
        if let Some(top_view_key) = self.top_view_key {
            view_map.get_view(top_view_key).display(view_map, buf);
        }
        view_map
            .get_view(self.command_line_key)
            .display(view_map, buf);

        if let Some(cursor_pos) = view_map.focused_view().get_cursor_pos() {
            place_cursor(buf, cursor_pos);
        } else {
            place_cursor(
                buf,
                Pos {
                    x: self.frame.width - 1,
                    y: self.frame.height - 1,
                },
            );
        }
        buf.append("\x1b[?25h");
        buf.write_to(libc::STDIN_FILENO);
    }

    fn get_view_key(&self) -> ViewKey {
        self.view_key
    }

    fn get_cursor_pos(&self) -> Option<Pos> {
        panic!("? get_cursor_pos")
        //None
        // self.focused_view().get_cursor_pos()
    }
}

impl DispatchTarget for Editor {
    fn get_key_bindings(&self) -> Bindings {
        Default::default()
    }
    fn execute_command(&mut self, name: String, args: Vec<Variant>) -> Result<Status> {
        panic!(
            "what to do with this command? {:?} {:?} send to editor",
            name, args
        );
    }

    fn send_key(&mut self, key: Key) -> Result<Status> {
        panic!("why is the editor receiving send_keys? [key={:?}]", key);
    }
}

#[allow(dead_code)]
impl Editor {
    pub fn get_should_quit(&self) -> bool {
        self.should_quit
    }
    pub fn install(plugin: PluginRef, view_map: &mut ViewMap) -> ViewKey {
        let command_line_key = view_map.get_next_key();
        let command_line = viewref(CommandLine::new(plugin.clone(), command_line_key));
        let editor_view_key = view_map.get_next_key();
        let docview = viewref(DocView::new(view_map.get_next_key(), plugin.clone()));
        let focused_view_key = docview.get_view_key();
        let slf = Self {
            plugin,
            view_key: editor_view_key,
            should_quit: false,
            frame: Rect::zero(),
            last_key: None,
            top_view_key: Some(focused_view_key),
            command_line_key,
        };
        let vk = slf.view_key;
        view_map.insert(viewref(slf), None, Some("editor".to_string()));
        view_map.insert(docview, Some(editor_view_key), None);
        view_map.insert(
            command_line,
            Some(editor_view_key),
            Some("command-line".to_string()),
        );
        view_map.set_focused_view(focused_view_key);
        view_map.set_root_view_key(editor_view_key);
        vk
    }
    pub fn set_top_view_key(&mut self, top_view_key: Option<ViewKey>) {
        self.top_view_key = top_view_key;
    }

    pub fn set_last_key(&mut self, key: Option<Key>) {
        self.last_key = key;
    }

    pub fn eat_status_result(&self, view_map: &mut ViewMap, result: Result<Status>) -> Result<()> {
        match result {
            Ok(status) => {
                self.set_status(view_map, status);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn set_status(&self, view_map: &mut ViewMap, status: Status) {
        let mut view = view_map.get_named_view("command-line").unwrap();
        view.set_status(status);
    }

    /*
    pub fn enter_command_mode(&mut self) {
        self.set_focus(self.command_line);
    }*/
}

impl Drop for Editor {
    fn drop(&mut self) {
        println!("Closing wim.\r\n  Screen size was {:?}\r", self.frame);
    }
}
