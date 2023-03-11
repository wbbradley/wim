use crate::buf::{place_cursor, Buf};
use crate::command::Command;
use crate::commandline::CommandLine;
use crate::consts::PROP_CMDLINE_FOCUSED;
use crate::dk::DK;
use crate::docview::DocView;
use crate::error::{Error, Result};
use crate::key::Key;
use crate::read::read_key;
use crate::status::Status;
use crate::termios::Termios;
use crate::types::{Pos, Rect};
use crate::view::{
    to_view, to_weak_view, PropertyValue, View, ViewContext, ViewKey, ViewKeyGenerator,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};
use std::time::{Duration, Instant};

pub struct VStack {
    view_key: ViewKey,
    views: Vec<Rc<RefCell<dyn View>>>,
}

impl ViewContext for VStack {}
impl View for VStack {
    fn layout(&mut self, frame: Rect) {
        let expected_per_view_height = std::cmp::max(1, frame.height / self.views.len());
        let mut used = 0;
        for view in self.views.iter() {
            if frame.height - used < expected_per_view_height {
                break;
            }
            let view_height = if used + expected_per_view_height * 2 > frame.height {
                frame.height - used
            } else {
                expected_per_view_height
            };

            view.borrow_mut().layout(Rect {
                x: frame.x,
                y: used,
                width: frame.width,
                height: view_height,
            });
            used += view_height;
        }
    }
    fn get_view_key(&self) -> &ViewKey {
        &self.view_key
    }
    fn display(&self, buf: &mut Buf, context: &dyn ViewContext) {
        self.views
            .iter()
            .for_each(|view| view.borrow().display(buf, context));
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        panic!("VStack should not be focused!");
    }
    fn execute_command(&mut self, command: Command) -> Result<Status> {
        Err(Error::new(format!(
            "Command {:?} not implemented for VStack",
            command
        )))
    }
}

#[allow(dead_code)]
pub struct Editor {
    termios: Termios,
    view_key: ViewKey,
    last_key: Option<Key>,
    views: HashMap<ViewKey, Rc<RefCell<DocView>>>,
    view_key_gen: ViewKeyGenerator,
    root_view: Rc<RefCell<dyn View>>,
    previous_views: Vec<Weak<RefCell<dyn View>>>,
    command_line: Rc<RefCell<CommandLine>>,
    frame: Rect,
}

impl View for Editor {
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

    fn get_view_key(&self) -> &ViewKey {
        &self.view_key
    }

    fn get_cursor_pos(&self) -> Option<Pos> {
        self.focused_view().borrow().get_cursor_pos()
    }

    fn execute_command(&mut self, command: Command) -> Result<Status> {
        match command {
            Command::FocusCommandLine => {
                self.enter_command_mode();
                Ok(Status::Cleared)
            }
            Command::FocusPrevious => {
                self.goto_previous_view();
                Ok(Status::Cleared)
            }
            _ => self.root_view.borrow_mut().execute_command(command),
        }
    }
    fn handle_key(&mut self, key: Key) -> Result<DK> {
        log::trace!(
            "[Editor::handle_key] sending key to view {}",
            self.focused_view().borrow().get_view_key()
        );
        self.focused_view().borrow_mut().handle_key(key)
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

fn build_view_map(views: Vec<Rc<RefCell<DocView>>>) -> HashMap<ViewKey, Rc<RefCell<DocView>>> {
    views
        .iter()
        .map(|view| (view.borrow().get_view_key().to_string(), view.clone()))
        .collect()
}

#[allow(dead_code)]
impl Editor {
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
            message: String::from("<C-w> to quit..."),
            expiry: Instant::now() + Duration::from_secs(5),
        }
    }
    pub fn new(termios: Termios) -> Self {
        let mut view_key_gen = ViewKeyGenerator::new();
        let views = vec![Rc::new(RefCell::new(DocView::new(
            view_key_gen.next_key_string(),
        )))];
        let focused_view = views[0].clone();
        Self {
            termios,
            view_key: view_key_gen.next_key_string(),
            frame: Rect::zero(),
            last_key: None,
            views: build_view_map(views),
            view_key_gen,
            previous_views: vec![to_weak_view(focused_view.clone())],
            root_view: focused_view,
            command_line: Rc::new(RefCell::new(CommandLine::new())),
        }
        // Initialize the command line cur info.
    }

    pub fn set_last_key(&mut self, key: Option<Key>) {
        self.last_key = key;
    }

    pub fn set_status(&mut self, status: Status) {
        log::trace!("Status Updated: {:?}", &status);
        self.command_line.borrow_mut().set_status(status);
    }

    pub fn goto_previous_view(&mut self) {
        self.previous_views.pop();
    }
    pub fn enter_command_mode(&mut self) {
        self.set_focus(to_view(&self.command_line));
    }

    fn set_focus(&mut self, view_to_focus: Rc<RefCell<dyn View>>) {
        let view_key = view_to_focus.borrow().get_view_key().clone();
        log::trace!("focusing view '{}'", view_key);
        self.previous_views.retain(|view| {
            // Keep the views that still exist and that aren't the intended one so we can move it
            // to the top of the stack..
            if let Some(view) = view.upgrade() {
                *view.borrow().get_view_key() != view_key
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
