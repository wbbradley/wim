use crate::bindings::Bindings;
use crate::error::{Error, Result};
use crate::plugin::PluginRef;
use crate::prelude::*;
use crate::status::Status;
use crate::types::Rect;
use crate::view::{ViewContext, ViewKey};

pub struct VStack {
    plugin: PluginRef,
    view_key: ViewKey,
    view_keys: Vec<ViewKey>,
}

impl ViewContext for VStack {
    fn get_property(&self, property: &str) -> Option<Variant> {
        panic!("not implemented: property: {}", property);
    }
}
impl View for VStack {
    fn get_doc_text(&self, _view_map: &ViewMap) -> Option<String> {
        panic!("not sure what to do with vstack get_doc_text call...");
    }
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.plugin = plugin;
    }
    fn layout(&mut self, _view_map: &ViewMap, size: Size) -> Vec<(ViewKey, Rect)> {
        let expected_per_view_height = std::cmp::max(1, size.height / self.view_keys.len());
        let mut used = 0;
        self.view_keys
            .iter()
            .filter_map(|view_key| {
                if size.height - used < expected_per_view_height {
                    return None;
                }
                let view_height = if used + expected_per_view_height * 2 > size.height {
                    size.height - used
                } else {
                    expected_per_view_height
                };

                let ret = Some((
                    *view_key,
                    Rect {
                        x: 0,
                        y: used,
                        width: size.width,
                        height: view_height,
                    },
                ));
                used += view_height;
                ret
            })
            .collect()
    }
    fn get_view_key(&self) -> ViewKey {
        self.view_key
    }
    fn display(&self, _view_map: &ViewMap, _bmp: &mut BitmapView) {
        // TODO: draw borders between views.
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
