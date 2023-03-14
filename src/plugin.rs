use rune::termcolor::{ColorChoice, StandardStream};
use rune::{Context, ContextError, Diagnostics, Module, Source, Sources, Vm};
use std::sync::Arc;

use crate::command::Command;
use crate::dk::DK;
use crate::key::Key;
use crate::mode::Mode;
use crate::noun::Noun;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Plugin {
    vm: Vm,
}
pub type PluginRef = Arc<Plugin>;

impl Plugin {
    pub fn handle_editor_key(_mode: Mode, _key: Key) -> Option<DK> {
        Some(Command::NewlineBelow.into())
    }
}

pub fn make_builtins_module() -> Result<Module, ContextError> {
    let mut module = Module::new();
    module.ty::<Noun>()?;
    module.ty::<Key>()?;
    module.function(&["noun_char"], || Noun::Char)?;
    Ok(module)
}

pub fn load_plugin() -> anyhow::Result<Arc<Plugin>> {
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
    /*
    let output = vm.call(["add"], (10i64, 20i64))?;
    let output = i64::from_value(output)?;

    println!("{}", output);
    */
    Ok(Arc::new(Plugin { vm }))
}
