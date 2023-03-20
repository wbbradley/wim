use crate::bindings::Bindings;
use crate::buf::place_cursor;
use crate::commandline::CommandLine;
use crate::consts::PROP_CMDLINE_FOCUSED;
use crate::docview::DocView;
use crate::error::Result;
use crate::keygen::ViewKeyGenerator;
use crate::plugin::PluginRef;
use crate::prelude::*;
use crate::propvalue::PropertyValue;
use crate::read::read_key;
use crate::status::Status;
use crate::trie::{Mapping, TrieNode};
use crate::types::{Pos, Rect};
use crate::view::{ancestor_path, ViewContext};

#[allow(dead_code)]
pub struct Editor {
    plugin: PluginRef,
    view_key: ViewKey,
    should_quit: bool,
    last_key: Option<Key>,
    view_map: HashMap<ViewKey, View>,
    view_key_gen: ViewKeyGenerator,
    root_view: ViewKey,
    previous_views: Vec<ViewKey>,
    command_line: ViewKey,
    frame: Rect,
}

impl ViewMapper for Editor {
    fn get_view(&self, view_key: ViewKey) -> &View {
        match self.view_map.get(&view_key) {
            Some(view) => &view,
            None => panic!(
                "can't find view with view_key={:?}\nviews:\n{:?}",
                view_key,
                self.view_map.keys()
            ),
        }
    }
    fn get_view_mut(&mut self, view_key: ViewKey) -> &mut View {
        match self.view_map.get_mut(&view_key) {
            Some(view) => view,
            None => panic!("can't find mutable view with view_key={:?}", view_key,),
        }
    }
}

impl ViewImpl for Editor {
    fn get_parent(&self) -> Option<ViewKey> {
        None
    }
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.plugin = plugin;
    }
    fn get_view_mode(&self) -> Mode {
        Mode::Normal
    }
    fn layout(&mut self, frame: Rect) {
        self.frame = frame;
        let root_view = self.get_view_or_focused_view_mut(Some(self.root_view));
        root_view.layout(Rect {
            x: 0,
            y: 0,
            width: frame.width,
            height: frame.height - 2,
        });
        self.get_view_mut(self.command_line).layout(Rect {
            x: 0,
            y: frame.height - 2,
            width: frame.width,
            height: 2,
        });
    }

    fn display(&self, buf: &mut Buf, context: &dyn ViewContext) {
        // Hide the cursor.
        buf.append("\x1b[?25l");
        self.get_view(self.root_view).display(buf, context);
        self.get_view(self.command_line).display(buf, context);
        if let Some(cursor_pos) = self.focused_view().get_cursor_pos() {
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
        self.focused_view().get_cursor_pos()
    }

    fn execute_command(&mut self, name: String, args: Vec<CallArg>) -> Result<Status> {
        if name == "focus-command-line" {
            self.enter_command_mode();
            Ok(Status::Cleared)
        } else if name == "focus-previous" {
            self.goto_previous_view();
            Ok(Status::Cleared)
        } else {
            self.get_view_mut(self.root_view)
                .execute_command(name, args)
        }
    }

    fn send_key(&mut self, key: Key) -> Result<Status> {
        panic!("why is the editor receiving send_keys? [key={:?}]", key);
    }
    fn get_key_bindings(&self, _root_view_key: ViewKey) -> Bindings {
        Default::default()
    }
}

impl ViewContext for Editor {
    fn get_property(&self, property: &str) -> Option<PropertyValue> {
        // log::trace!("Editor::get_property({}) called...", property);
        if property == PROP_CMDLINE_FOCUSED {
            Some(PropertyValue::Bool(
                self.focused_view().get_view_key() == self.command_line,
            ))
        } else {
            self.focused_view().get_property(property)
        }
    }
}

fn build_view_map(command_line: View, views: Vec<View>) -> HashMap<ViewKey, View> {
    let mut map: HashMap<ViewKey, View> = views
        .into_iter()
        .map(|view| (view.get_view_key(), view))
        .collect();
    let command_line_view_key = command_line.get_view_key();
    map.insert(command_line_view_key, command_line);
    map
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
    fn focused_view_key(&self) -> ViewKey {
        assert!(!self.previous_views.is_empty());
        *self.previous_views.last().unwrap()
    }
    fn focused_view(&self) -> &View {
        self.get_view(self.focused_view_key())
    }
    fn focused_view_mut(&mut self) -> &mut View {
        assert!(!self.previous_views.is_empty());
        self.get_view_mut(*self.previous_views.last().unwrap())
    }
    pub fn welcome_status() -> Status {
        Status::Message {
            message: String::from("<C-c> to quit..."),
            expiry: Instant::now() + Duration::from_secs(5),
        }
    }
    pub fn new(plugin: PluginRef) -> Self {
        let mut view_key_gen = ViewKeyGenerator::new();
        let views: Vec<View> = vec![View::DocView(DocView::new(
            view_key_gen.next_key(),
            plugin.clone(),
        ))];
        let focused_view_key = views[0].get_view_key();
        let command_line_key = view_key_gen.next_key();
        let view_map = build_view_map(
            View::CommandLine(CommandLine::new(plugin.clone(), command_line_key)),
            views,
        );
        Self {
            plugin,
            view_key: view_key_gen.next_key(),
            should_quit: false,
            frame: Rect::zero(),
            last_key: None,
            view_map,
            view_key_gen,
            previous_views: vec![focused_view_key],
            root_view: focused_view_key,
            command_line: command_line_key,
        }
    }

    pub fn set_last_key(&mut self, key: Option<Key>) {
        self.last_key = key;
    }

    pub fn eat_status_result(&mut self, result: Result<Status>) -> Result<()> {
        match result {
            Ok(status) => Ok(self.set_status(status)),
            Err(error) => Err(error),
        }
    }

    pub fn set_status(&mut self, status: Status) {
        log::trace!("Status Updated: {:?}", &status);
        if let View::CommandLine(cmdline) = self.get_view_mut(self.command_line) {
            cmdline.set_status(status);
        }
    }

    pub fn send_key_to_view(&mut self, view_key: Option<ViewKey>, key: Key) -> Result<()> {
        match self.get_view_or_focused_view_mut(view_key).send_key(key) {
            Ok(status) => {
                self.set_status(status);
                Ok(())
            }
            Err(error) => Err(error),
        }
    }

    pub fn get_view_or_focused_view(&self, view_key: Option<ViewKey>) -> &View {
        match view_key {
            Some(view_key) => self.get_view(view_key),
            None => self.focused_view(),
        }
    }
    pub fn get_view_or_focused_view_mut(&mut self, view_key: Option<ViewKey>) -> &mut View {
        match view_key {
            Some(view_key) => self.get_view_mut(view_key),
            None => self.focused_view_mut(),
        }
    }

    pub(crate) fn handle_keys(&mut self, dks: &mut VecDeque<DK>) -> HandleKey {
        let path: Vec<ViewKey> = ancestor_path(self, self.focused_view_key());
        let trie: TrieNode = TrieNode::from_ancestor_path(path, &self.view_map, self.view_key);
        let inbound_keys: Vec<Key> = dks
            .iter()
            .take_while(|dk| matches!(dk, DK::Key(_)))
            .map(|dk| match dk {
                DK::Key(key) => *key,
                _ => {
                    panic!("foogoo");
                }
            })
            .collect();
        if inbound_keys.is_empty() {
            return HandleKey::DK(dks.pop_front().unwrap());
        }
        trace!("inbound_keys of dks === {:?} of {:?}", inbound_keys, dks);
        assert!(!inbound_keys.is_empty());
        match trie.longest_prefix(&inbound_keys) {
            Mapping::Bound { dk, remaining } => {
                trace!(
                    "keys {:?} translated into dk={:?}, leaving remaining={:?}",
                    inbound_keys,
                    dk,
                    remaining
                );
                (0..(inbound_keys.len() - remaining.len())).for_each(|_| {
                    dks.pop_front();
                });
                HandleKey::DK(dk)
            }
            Mapping::Choices(choices) => {
                trace!("found choices {:?}", choices);
                assert!(!choices.is_empty());
                HandleKey::Choices(choices.iter().map(|(key, _)| key).cloned().collect())
            }
            Mapping::None => {
                trace!("no mapping found, returning SendKey({:?})", inbound_keys[0]);
                dks.pop_front();
                HandleKey::DK(DK::Dispatch(
                    Some(self.focused_view_key()),
                    Message::SendKey(inbound_keys[0]),
                ))
            }
        }
    }
    pub fn goto_previous_view(&mut self) {
        self.previous_views.pop();
    }
    pub fn enter_command_mode(&mut self) {
        self.set_focus(self.command_line);
    }

    fn set_focus(&mut self, view_key_to_focus: ViewKey) {
        log::trace!("focusing view '{:?}'", view_key_to_focus);
        self.previous_views.retain(|vk| {
            // Keep the views that still exist and that aren't the intended one so we can move it
            // to the top of the stack..
            *vk != view_key_to_focus
        });

        self.previous_views.push(view_key_to_focus);
    }
}

#[derive(Debug)]
pub(crate) enum HandleKey {
    DK(DK),
    Choices(Vec<Key>),
}

impl Drop for Editor {
    fn drop(&mut self) {
        println!("Closing wim.\r\n  Screen size was {:?}\r", self.frame);
    }
}
