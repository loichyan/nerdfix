#[macro_use]
mod util;
mod autocomplete;
mod cli;
mod error;
mod icon;
mod parser;
mod runtime;

use clap::Parser;
use cli::Command;
use inquire::InquireError;
use runtime::{CheckerContext, Runtime};
use thisctx::WithContext;

static CACHED: &str = include_str!("./cached.txt");

fn main() -> anyhow::Result<()> {
    let args = cli::Cli::parse();
    let mut rt = Runtime::builder();
    rt.load_inline_cache(CACHED);
    for path in args.cache.iter() {
        rt.load_cache(path)?;
    }
    for path in args.cheat_sheet.iter() {
        rt.load_cheat_sheet(path)?;
    }
    let rt = rt.build();
    match args.cmd {
        Command::Cache { output } => rt.save_cache(&output)?,
        Command::Check { source } => {
            let mut context = CheckerContext::default();
            for path in source.iter() {
                rt.check(&mut context, None, path)?;
            }
        }
        Command::Fix { source } => {
            let mut context = CheckerContext::default();
            for path in source.iter() {
                let mut patched = String::default();
                if rt.check(&mut context, Some(&mut patched), path)? {
                    match inquire::Confirm::new("Are your sure to write the patched content?")
                        .with_help_message("Press <Ctrl-C> to quit")
                        .prompt()
                    {
                        Ok(true) => std::fs::write(path, patched).context(error::Io(path))?,
                        Err(InquireError::OperationInterrupted) => break,
                        _ => (),
                    }
                }
            }
        }
        Command::Search {} => {
            rt.prompt_input_icon(None).ok();
        }
    }
    Ok(())
}
