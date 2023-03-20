use crate::bindings::Bindings;
use crate::error::{Error, Result};
use crate::plugin::PluginRef;
use crate::prelude::*;
use crate::status::Status;
use crate::types::{Pos, Rect};
use crate::view::{ViewContext, ViewKey};

pub struct VStack {
    parent: Option<ViewKey>,
    plugin: PluginRef,
    view_key: ViewKey,
    views: Vec<View>,
}

impl VStack {
    #[allow(dead_code)]
    pub fn set_parent(&mut self, parent: Option<ViewKey>) {
        self.parent = parent;
    }
}

impl ViewContext for VStack {}
impl ViewImpl for VStack {
    fn get_parent(&self) -> Option<ViewKey> {
        self.parent
    }
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.plugin = plugin;
    }
    fn layout(&mut self, frame: Rect) {
        let expected_per_view_height = std::cmp::max(1, frame.height / self.views.len());
        let mut used = 0;
        for view in &mut self.views {
            if frame.height - used < expected_per_view_height {
                break;
            }
            let view_height = if used + expected_per_view_height * 2 > frame.height {
                frame.height - used
            } else {
                expected_per_view_height
            };

            view.layout(Rect {
                x: frame.x,
                y: used,
                width: frame.width,
                height: view_height,
            });
            used += view_height;
        }
    }
    fn get_view_mode(&self) -> Mode {
        Mode::Normal
    }
    fn get_view_key(&self) -> ViewKey {
        self.view_key
    }
    fn display(&self, buf: &mut Buf, context: &dyn ViewContext) {
        self.views
            .iter()
            .for_each(|view| view.display(buf, context));
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        panic!("VStack should not be focused!");
    }
    fn execute_command(&mut self, name: String, args: Vec<CallArg>) -> Result<Status> {
        Err(Error::new(format!(
            "Command {:?} {:?} not implemented for VStack",
            name, args,
        )))
    }
    fn send_key(&mut self, key: Key) -> Result<Status> {
        panic!("why is the vstack receiving send_keys? [key={:?}]", key);
    }
    fn get_key_bindings(&self, _root_view_key: ViewKey) -> Bindings {
        Default::default()
    }
}
