use rune::termcolor::{ColorChoice, StandardStream};
use rune::{Context, Diagnostics, FromValue, Source, Sources, Vm};
use std::sync::Arc;

#[derive(Debug)]
pub struct Plugin {}

pub fn load_plugin() -> anyhow::Result<Plugin> {
    let filename = "wimrc.rn";
    println!("[wim] Loading plugin {}...", filename);
    let rune_context = Context::with_default_modules()?;
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
    let mut vm = Vm::new(rune_runtime, Arc::new(unit));
    let output = vm.call(["add"], (10i64, 20i64))?;
    let output = i64::from_value(output)?;

    println!("{}", output);
    Ok(Plugin {})
}
