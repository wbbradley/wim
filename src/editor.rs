use crate::bindings::Bindings;
use crate::buf::place_cursor;
use crate::commandline::CommandLine;
use crate::docview::DocView;
use crate::error::Result;
use crate::plugin::PluginRef;
use crate::prelude::*;
use crate::read::read_key;
use crate::status::Status;
use crate::types::{Pos, Rect};
use crate::view::ViewContext;

#[allow(dead_code)]
pub struct Editor {
    plugin: PluginRef,
    view_key: ViewKey,
    should_quit: bool,
    last_key: Option<Key>,
    command_line: ViewKey,
    frame: Rect,
}

impl ViewContext for Editor {}
impl View for Editor {
    fn as_view_context(&self) -> &dyn ViewContext {
        self
    }
    fn as_dispatch_target(&self) -> &dyn DispatchTarget {
        self
    }
    fn as_dispatch_target_mut(&mut self) -> &mut dyn DispatchTarget {
        self
    }
    fn get_parent(&self) -> Option<ViewKey> {
        None
    }
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.plugin = plugin;
    }
    fn get_view_mode(&self) -> Mode {
        Mode::Normal
    }
    fn layout(&mut self, view_map: &mut ViewMap, frame: Rect) {
        self.frame = frame;
        let view_key = view_map.get_root_view_key();
        let root_view = view_map.get_view_or_focused_view_mut(Some(view_key));
        root_view.layout(
            view_map,
            Rect {
                x: 0,
                y: 0,
                width: frame.width,
                height: frame.height - 2,
            },
        );
        view_map.get_view_mut(self.command_line).layout(
            view_map,
            Rect {
                x: 0,
                y: frame.height - 2,
                width: frame.width,
                height: 2,
            },
        );
    }

    fn display(&self, view_map: &ViewMap, buf: &mut Buf, context: &dyn ViewContext) {
        // Hide the cursor.
        buf.append("\x1b[?25l");
        view_map
            .get_view(view_map.get_root_view_key())
            .display(view_map, buf, context);
        view_map
            .get_view(self.command_line)
            .display(view_map, buf, context);
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

fn build_view_map(command_line: Box<dyn View>, views: Vec<Box<dyn View>>, view_map: &mut ViewMap) {
    views
        .into_iter()
        .for_each(|view| view_map.insert(view.get_view_key(), view, None));
    let command_line_view_key = command_line.get_view_key();
    view_map.insert(
        command_line_view_key,
        command_line,
        Some("command-line".to_string()),
    );
}

#[allow(dead_code)]
impl Editor {
    pub fn get_should_quit(&self) -> bool {
        self.should_quit
    }
    pub fn _read_key(&mut self) -> Option<Key> {
        let key = read_key();
        self.set_last_key(key);
        key
    }
    pub fn welcome_status() -> Status {
        Status::Message {
            message: String::from("<C-c> to quit..."),
            expiry: Instant::now() + Duration::from_secs(5),
        }
    }
    pub fn install(plugin: PluginRef, view_map: &mut ViewMap) -> ViewKey {
        let views: Vec<Box<dyn View>> = vec![Box::new(DocView::new(
            view_map.get_next_key(),
            plugin.clone(),
        ))];
        let focused_view_key = views[0].get_view_key();
        let command_line_key = view_map.get_next_key();
        build_view_map(
            Box::new(CommandLine::new(plugin.clone(), command_line_key)),
            views,
            view_map,
        );
        let slf = Self {
            plugin,
            view_key: view_map.get_next_key(),
            should_quit: false,
            frame: Rect::zero(),
            last_key: None,
            command_line: command_line_key,
        };
        view_map.set_focused_view(focused_view_key);
        view_map.set_root_view_key(slf.view_key);
        let vk = slf.view_key;
        view_map.insert(slf.view_key, Box::new(slf), Some("editor".to_string()));
        vk
    }

    pub fn set_last_key(&mut self, key: Option<Key>) {
        self.last_key = key;
    }

    pub fn eat_status_result(&self, view_map: &mut ViewMap, result: Result<Status>) -> Result<()> {
        match result {
            Ok(status) => Ok(self.set_status(view_map, status)),
            Err(error) => Err(error),
        }
    }

    pub fn set_status(&self, view_map: &mut ViewMap, status: Status) {
        let view = view_map.get_named_view("command-line").unwrap();
        let cmdline: &mut dyn View = view_map.get_view_mut(view);
        cmdline.set_status(status);
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
