use crate::command::Command;
use crate::dk::DK;
use crate::key::Key;
use crate::mode::Mode;
use crate::noun::Noun;
use crate::rel::Rel;
use rune::termcolor::{ColorChoice, StandardStream};
use rune::{Context, ContextError, Diagnostics, FromValue, Module, Source, Sources, Vm};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Plugin {
    vm: Vm,
}
pub type PluginRef = Rc<RefCell<Plugin>>;

impl Plugin {
    pub fn handle_editor_key(
        &mut self,
        mode: Mode,
        keys: &[Key],
    ) -> std::result::Result<Option<DK>, rune::runtime::VmError> {
        let keys: Vec<Key> = keys.iter().copied().collect();
        let output = self.vm.call(["handle_key"], (mode, keys))?;
        <Option<DK>>::from_value(output)
    }
}

pub fn make_builtins_module() -> Result<Module, ContextError> {
    let mut module = Module::new();
    module.ty::<Noun>()?;
    module.ty::<Command>()?;
    module.ty::<Rel>()?;
    module.ty::<DK>()?;
    module.ty::<Mode>()?;
    module.ty::<Key>()?;
    module.function(&["noun_char"], || Noun::Char)?;
    Ok(module)
}

pub fn load_plugin() -> anyhow::Result<PluginRef> {
    let builtins = make_builtins_module()?;
    let filename = "wimrc.rn";
    println!("[wim] Loading plugin {}...", filename);
    let mut rune_context = Context::with_default_modules()?;
    rune_context.install(&builtins)?;
    let rune_runtime = Arc::new(rune_context.runtime());
    let mut sources = Sources::new();
    sources.insert(Source::new(filename, std::fs::read_to_string(filename)?));

    let mut diagnostics = Diagnostics::new();

    let result = rune::prepare(&mut sources)
        .with_context(&rune_context)
        .with_diagnostics(&mut diagnostics)
        .build();

    if !diagnostics.is_empty() {
        let mut writer = StandardStream::stderr(ColorChoice::Always);
        diagnostics.emit(&mut writer, &sources)?;
    }

    let unit = result?;
    let vm = Vm::new(rune_runtime, Arc::new(unit));
    Ok(Rc::new(RefCell::new(Plugin { vm })))
}
