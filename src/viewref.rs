use crate::error::Result;
use crate::prelude::*;

pub fn viewref<T>(x: T) -> ViewRef
where
    T: View + 'static,
{
    ViewRef {
        ptr: Rc::new(RefCell::new(x)),
    }
}

#[derive(Clone)]
pub struct ViewRef {
    ptr: Rc<RefCell<dyn View>>,
}

impl DispatchClient for ViewRef {
    fn get_key_bindings(&self) -> Bindings {
        self.ptr.borrow().get_key_bindings()
    }
    fn execute_command(&self, name: String, args: Vec<Variant>) -> Result<Status> {
        self.ptr.borrow_mut().execute_command(name, args)
    }
    fn send_key(&self, key: Key) -> Result<Status> {
        self.ptr.borrow_mut().send_key(key)
    }
}

impl DispatchTarget for ViewRef {
    fn get_key_bindings(&self) -> Bindings {
        self.ptr.borrow().get_key_bindings()
    }
    fn execute_command(&mut self, name: String, args: Vec<Variant>) -> Result<Status> {
        self.ptr.borrow_mut().execute_command(name, args)
    }
    fn send_key(&mut self, key: Key) -> Result<Status> {
        self.ptr.borrow_mut().send_key(key)
    }
}

impl ViewContext for ViewRef {
    fn get_property(&self, property: &str) -> Option<Variant> {
        self.ptr.borrow().get_property(property)
    }
}

impl View for ViewRef {
    fn get_parent(&self) -> Option<ViewKey> {
        self.ptr.borrow().get_parent()
    }
    fn install_plugins(&mut self, plugin: PluginRef) {
        self.ptr.borrow_mut().install_plugins(plugin)
    }
    fn layout(&mut self, view_map: &ViewMap, frame: Rect) -> Vec<(ViewKey, Rect)> {
        self.ptr.borrow_mut().layout(view_map, frame)
    }
    fn display(&self, view_map: &ViewMap, buf: &mut Buf, context: &dyn ViewContext) {
        self.ptr.borrow().display(view_map, buf, context)
    }
    fn get_view_key(&self) -> ViewKey {
        self.ptr.borrow().get_view_key()
    }
    fn get_cursor_pos(&self) -> Option<Pos> {
        self.ptr.borrow().get_cursor_pos()
    }
    fn get_view_mode(&self) -> Mode {
        self.ptr.borrow().get_view_mode()
    }
    fn set_status(&mut self, status: Status) {
        self.ptr.borrow_mut().set_status(status)
    }
}
