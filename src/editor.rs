use crate::bindings::Bindings;
use crate::buf::place_cursor;
use crate::commandline::CommandLine;
use crate::consts::PROP_CMDLINE_FOCUSED;
use crate::docview::DocView;
use crate::error::not_impl;
use crate::error::Result;
use crate::keygen::ViewKeyGenerator;
use crate::plugin::PluginRef;
use crate::prelude::*;
use crate::propvalue::PropertyValue;
use crate::read::read_key;
use crate::status::Status;
use crate::trie::TrieNode;
use crate::types::{Pos, Rect};
use crate::view::{to_view, to_weak_view, ViewContext};

#[allow(dead_code)]
pub struct Editor {
    plugin: PluginRef,
    view_key: ViewKey,
    should_quit: bool,
    last_key: Option<Key>,
    view_map: HashMap<ViewKey, ViewRef>,
    view_key_gen: ViewKeyGenerator,
    root_view: Rc<RefCell<dyn View>>,
    previous_views: Vec<Weak<RefCell<dyn View>>>,
    command_line: Rc<RefCell<CommandLine>>,
    frame: Rect,
}

impl View for Editor {
    fn get_parent(&self) -> Option<Weak<RefCell<dyn View>>> {
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
        self.root_view.borrow_mut().layout(Rect {
            x: 0,
            y: 0,
            width: frame.width,
            height: frame.height - 2,
        });
        self.command_line.borrow_mut().layout(Rect {
            x: 0,
            y: frame.height - 2,
            width: frame.width,
            height: 2,
        });
    }

    fn display(&self, buf: &mut Buf, context: &dyn ViewContext) {
        // Hide the cursor.
        buf.append("\x1b[?25l");
        self.root_view.borrow().display(buf, context);
        self.command_line.borrow().display(buf, context);
        if let Some(cursor_pos) = self.focused_view().borrow().get_cursor_pos() {
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
        self.focused_view().borrow().get_cursor_pos()
    }

    fn execute_command(&mut self, name: String, args: Vec<CallArg>) -> Result<Status> {
        if name == "focus-command-line" {
            self.enter_command_mode();
            Ok(Status::Cleared)
        } else if name == "focus-previous" {
            self.goto_previous_view();
            Ok(Status::Cleared)
        } else {
            self.root_view.borrow_mut().execute_command(name, args)
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
                self.focused_view().borrow().get_view_key()
                    == self.command_line.borrow().get_view_key(),
            ))
        } else {
            self.focused_view().borrow().get_property(property)
        }
    }
}

fn build_view_map(command_line: ViewRef, views: Vec<ViewRef>) -> HashMap<ViewKey, ViewRef> {
    let mut map: HashMap<ViewKey, ViewRef> = views
        .iter()
        .map(|view| (view.borrow().get_view_key(), view.clone()))
        .collect();
    let command_line_view_key = command_line.borrow().get_view_key();
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
    fn focused_view(&self) -> Rc<RefCell<dyn View>> {
        assert!(!self.previous_views.is_empty());
        self.previous_views.last().unwrap().upgrade().unwrap()
    }
    pub fn welcome_status() -> Status {
        Status::Message {
            message: String::from("<C-c> to quit..."),
            expiry: Instant::now() + Duration::from_secs(5),
        }
    }
    pub fn new(plugin: PluginRef) -> Self {
        let mut view_key_gen = ViewKeyGenerator::new();
        let views: Vec<ViewRef> = vec![Rc::new(RefCell::new(DocView::new(
            view_key_gen.next_key(),
            plugin.clone(),
        )))];
        let focused_view = views[0].clone();
        let command_line = Rc::new(RefCell::new(CommandLine::new(
            plugin.clone(),
            view_key_gen.next_key(),
        )));
        let view_map = build_view_map(command_line.clone(), views);
        Self {
            plugin,
            view_key: view_key_gen.next_key(),
            should_quit: false,
            frame: Rect::zero(),
            last_key: None,
            view_map,
            view_key_gen,
            previous_views: vec![to_weak_view(focused_view.clone())],
            root_view: focused_view,
            command_line,
        }
    }

    pub fn set_last_key(&mut self, key: Option<Key>) {
        self.last_key = key;
    }

    pub fn set_status(&mut self, status: Status) {
        log::trace!("Status Updated: {:?}", &status);
        self.command_line.borrow_mut().set_status(status);
    }
    pub fn send_key_to_view(&mut self, view_key: Option<ViewKey>, key: Key) -> Result<Status> {
        self.get_view_or_focused_view(view_key)
            .borrow_mut()
            .send_key(key)
    }

    fn get_view_or_focused_view(&self, view_key: Option<ViewKey>) -> ViewRef {
        match view_key {
            Some(view_key) => match self.view_map.get(&view_key) {
                Some(view) => view.clone(),
                None => self.focused_view(),
            },
            None => self.focused_view(),
        }
    }

    pub fn handle_keys(&mut self, dks: &mut VecDeque<DK>) -> Result<Option<DK>> {
        let target_view = self.get_view_or_focused_view(None);
        log::trace!(
            "[Editor::handle_key] mapping keys from dks {:?} to view {:?}",
            dks,
            target_view.borrow().get_view_key()
        );
        let mut ancestor_path: Vec<ViewKey> = Default::default();
        target_view.borrow().ancestor_path(&mut ancestor_path);
        let trie: TrieNode =
            TrieNode::from_ancestor_path(ancestor_path, &self.view_map, self.view_key);
        let inbound_keys: Vec<Key> = dks
            .iter()
            .take_while(|dk| matches!(dk, DK::Key(_)))
            .map(|dk| match dk {
                DK::Key(key) if key != &Key::None => *key,
                _ => {
                    panic!("foogoo");
                }
            })
            .collect();

        match trie.longest_prefix(&inbound_keys) {
            Some((dk, remaining)) => {
                trace!("key {:?} translated into dk {:?}", inbound_keys, dk);
                (0..(inbound_keys.len() - remaining.len())).for_each(|_| {
                    dks.pop_front();
                });
                Ok(Some(dk))
            }
            None => Err(not_impl!(
                "Editor::handle_key: no handler for {:?} / {:?}",
                dks,
                inbound_keys
            )),
        }
    }
    pub fn goto_previous_view(&mut self) {
        self.previous_views.pop();
    }
    pub fn enter_command_mode(&mut self) {
        self.set_focus(to_view(&self.command_line));
    }

    fn set_focus(&mut self, view_to_focus: Rc<RefCell<dyn View>>) {
        let view_key = view_to_focus.borrow().get_view_key();
        log::trace!("focusing view '{:?}'", view_key);
        self.previous_views.retain(|view| {
            // Keep the views that still exist and that aren't the intended one so we can move it
            // to the top of the stack..
            if let Some(view) = view.upgrade() {
                view.borrow().get_view_key() != view_key
            } else {
                false
            }
        });

        self.previous_views.push(Rc::downgrade(&view_to_focus));
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        println!("Closing wim.\r\n  Screen size was {:?}\r", self.frame);
    }
}
