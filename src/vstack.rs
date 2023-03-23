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
    view_keys: Vec<ViewKey>,
}

impl VStack {
    #[allow(dead_code)]
    pub fn set_parent(&mut self, parent: Option<ViewKey>) {
        self.parent = parent;
    }
}

impl ViewContext for VStack {}
impl View for VStack {
    fn get_parent(&self) -> Option<ViewKey> {
        self.parent
    }
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.plugin = plugin;
    }
    fn layout(&mut self, _view_map: &ViewMap, frame: Rect) -> Vec<(ViewKey, Rect)> {
        let expected_per_view_height = std::cmp::max(1, frame.height / self.view_keys.len());
        let mut used = 0;
        self.view_keys
            .iter()
            .filter_map(|view_key| {
                if frame.height - used < expected_per_view_height {
                    return None;
                }
                let view_height = if used + expected_per_view_height * 2 > frame.height {
                    frame.height - used
                } else {
                    expected_per_view_height
                };

                let ret = Some((
                    *view_key,
                    Rect {
                        x: frame.x,
                        y: used,
                        width: frame.width,
                        height: view_height,
                    },
                ));
                used += view_height;
                ret
            })
            .collect()
    }
    fn get_view_mode(&self) -> Mode {
        Mode::Normal
    }
    fn get_view_key(&self) -> ViewKey {
        self.view_key
    }
    fn display(&self, view_map: &ViewMap, buf: &mut Buf, context: &dyn ViewContext) {
        self.view_keys
            .iter()
            .cloned()
            .for_each(|view_key| view_map.get_view(view_key).display(view_map, buf, context));
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        panic!("VStack should not be focused!");
    }
}

impl DispatchTarget for VStack {
    fn get_key_bindings(&self) -> Bindings {
        Default::default()
    }
    fn execute_command(&mut self, name: String, args: Vec<Variant>) -> Result<Status> {
        Err(Error::new(format!(
            "Command {:?} {:?} not implemented for VStack",
            name, args,
        )))
    }
    fn send_key(&mut self, key: Key) -> Result<Status> {
        panic!("why is the vstack receiving send_keys? [key={:?}]", key);
    }
}
