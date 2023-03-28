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
use runtime::{CheckerContext, Runtime};
use thisctx::WithContext;
use tracing::error;

use crate::runtime::YesOrNo;

static CACHED: &str = include_str!("./cached.txt");

fn main_impl() -> error::Result<()> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).context(error::Any)?;

    let args = cli::Cli::parse();
    let mut rt = Runtime::builder();
    if args.input.is_empty() {
        rt.load_cache(CACHED);
    } else {
        for path in args.input.iter() {
            rt.load_input(path)?;
        }
    }
    let rt = rt.build();
    match args.cmd {
        Command::Cache { output } => rt.save_cache(&output)?,
        Command::Check { source } => {
            let mut context = CheckerContext::default();
            for path in source.iter() {
                log_or_break!(rt.check(&mut context, path, false));
            }
        }
        // TODO: support autofix
        Command::Fix { source, mut yes } => {
            let mut context = CheckerContext::default();
            for path in source.iter() {
                log_or_break!((|| {
                    if let Some(patched) = rt.check(&mut context, path, true)? {
                        if !yes {
                            match rt.prompt_yes_or_no(
                                "Are your sure to write the patched content?",
                                None,
                            )? {
                                YesOrNo::No => return Ok(()),
                                YesOrNo::AllYes => yes = true,
                                _ => {}
                            }
                        }
                        std::fs::write(path, patched).context(error::Io(path))?;
                    }
                    Ok(())
                })());
            }
        }
        Command::Search {} => {
            rt.prompt_input_icon(None).ok();
        }
    }
    Ok(())
}

fn main() {
    if let Err(e) = main_impl() {
        log_error!(e);
    }
}
