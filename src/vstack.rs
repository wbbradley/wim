use crate::buf::Buf;
use crate::command::Command;
use crate::error::{Error, Result};
use crate::plugin::PluginRef;
use crate::status::Status;
use crate::types::{Pos, Rect};
use crate::view::View;
use crate::view::{ViewContext, ViewKey};
use std::cell::RefCell;
use std::rc::Rc;

pub struct VStack {
    plugin: PluginRef,
    view_key: ViewKey,
    views: Vec<Rc<RefCell<dyn View>>>,
}

impl ViewContext for VStack {}
impl View for VStack {
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.plugin = plugin;
    }
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
