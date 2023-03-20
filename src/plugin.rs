use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
pub struct Plugin {}
impl Plugin {
    pub fn new() -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {}))
    }
}

pub type PluginRef = Rc<RefCell<Plugin>>;
