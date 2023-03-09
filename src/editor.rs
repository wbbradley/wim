use crate::buf::{buf_fmt, Buf};
use crate::command::Command;
use crate::commandline::CommandLine;
use crate::dk::DK;
use crate::docview::DocView;
use crate::error::{Error, Result};
use crate::read::{read_key, Key};
use crate::status::Status;
use crate::termios::Termios;
use crate::types::{Pos, Rect};
use crate::view::{PropertyValue, View, ViewContext, ViewKey, ViewKeyGenerator};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

pub struct VStack {
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
    last_key: Option<Key>,
    views: HashMap<ViewKey, Rc<RefCell<DocView>>>,
    view_key_gen: ViewKeyGenerator,
    focused_view: Rc<RefCell<dyn View>>,
    root_view: Rc<RefCell<dyn View>>,
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
        if let Some(cursor_pos) = self.focused_view.borrow().get_cursor_pos() {
            buf_fmt!(buf, "\x1b[{};{}H", cursor_pos.y, cursor_pos.x);
        } else {
            buf_fmt!(buf, "\x1b[{};{}H", self.frame.height, self.frame.width);
        }
        buf.append("\x1b[?25h");
        buf.write_to(libc::STDIN_FILENO);
    }

    fn get_cursor_pos(&self) -> Option<Pos> {
        self.focused_view.borrow().get_cursor_pos()
    }

    fn execute_command(&mut self, command: Command) -> Result<Status> {
        Err(Error::not_impl(format!(
            "{} does not yet implement {:?}",
            std::any::type_name::<Self>(),
            command
        )))
    }
    fn dispatch_key(&mut self, key: Key) -> Result<DK> {
        self.focused_view.borrow_mut().dispatch_key(key)
    }
}

impl ViewContext for Editor {
    fn get_property<'a>(&self, property: &str) -> PropertyValue<'a> {
        self.focused_view.borrow().get_property(property)
    }
}

fn build_view_map(views: Vec<Rc<RefCell<DocView>>>) -> HashMap<ViewKey, Rc<RefCell<DocView>>> {
    views
        .iter()
        .map(|view| (view.borrow().get_view_key(), view.clone()))
        .collect()
}

#[allow(dead_code)]
impl Editor {
    pub fn _read_key(&mut self) -> Option<Key> {
        let key = read_key();
        self.set_last_key(key);
        key
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
            frame: Rect::zero(),
            last_key: None,
            views: build_view_map(views),
            view_key_gen,
            focused_view: focused_view.clone(),
            root_view: focused_view.clone(),
            command_line: Rc::new(RefCell::new(CommandLine::new())),
        }
        // Initialize the command line cur info.
    }

    pub fn dispatch_command(&mut self, command: Command) -> Result<Status> {
        self.root_view.borrow_mut().execute_command(command)
    }

    pub fn set_last_key(&mut self, key: Option<Key>) {
        self.last_key = key;
    }

    pub fn set_status(&mut self, status: Status) {
        log::trace!("Status Updated: {:?}", &status);
        self.command_line.borrow_mut().set_status(status);
    }

    pub fn enter_command_mode(&mut self) {
        self.focused_view = self.command_line.clone();
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        println!("Closing wim.\r\n  Screen size was {:?}\r", self.frame);
    }
}
