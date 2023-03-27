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
use thisctx::{IntoError, WithContext};
use tracing::error;

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
                log_or_break!(rt.check(&mut context, None, path));
            }
        }
        // TODO: support autofix
        Command::Fix { source } => {
            let mut context = CheckerContext::default();
            for path in source.iter() {
                let mut patched = String::default();
                log_or_break!((|| {
                    if rt.check(&mut context, Some(&mut patched), path)? {
                        match inquire::Confirm::new("Are your sure to write the patched content?")
                            .with_help_message("Press <Ctrl-C> to quit")
                            .prompt()
                        {
                            Ok(true) => std::fs::write(path, patched).context(error::Io(path))?,
                            Err(InquireError::OperationInterrupted) => {
                                return error::Interrupted.fail()
                            }
                            _ => (),
                        }
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
