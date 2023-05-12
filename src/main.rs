#[macro_use]
mod util;
mod autocomplete;
mod cli;
mod error;
mod icon;
mod parser;
mod prompt;
mod runtime;

shadow_rs::shadow!(shadow);

use clap::Parser;
use cli::Command;
use prompt::YesOrNo;
use runtime::{CheckerContext, Runtime};
use std::path::PathBuf;
use thisctx::WithContext;
use tracing::{error, warn, Level};
use util::ResultExt;
use walkdir::WalkDir;

static CACHED: &str = include_str!("./cached.txt");

fn walk<'a>(
    paths: impl 'a + IntoIterator<Item = PathBuf>,
    recursive: bool,
) -> impl 'a + Iterator<Item = error::Result<PathBuf>> {
    if !recursive {
        Box::new(paths.into_iter().map(Ok)) as Box<dyn Iterator<Item = _>>
    } else {
        Box::new(
            paths
                .into_iter()
                .flat_map(WalkDir::new)
                .map(|entry| Ok(entry?.into_path()))
                .filter(|p| if let Ok(p) = p { p.is_file() } else { true }),
        )
    }
}

fn main_impl() -> error::Result<()> {
    let args = cli::Cli::parse();

    let lv = match args.verbose {
        0 => Level::WARN,
        1 => Level::INFO,
        2 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_writer(std::io::stderr)
        .with_max_level(lv)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).context(error::Any)?;

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
        Command::Check {
            format,
            source,
            recursive,
        } => {
            let mut context = CheckerContext {
                format,
                ..Default::default()
            };
            for path in walk(source, recursive) {
                tryb! {
                    rt.check(&mut context, &path?, false)
                }
                .ignore_interrupted()
                .log_error();
            }
        }
        Command::Fix {
            yes,
            write,
            select_first,
            replace,
            recursive,
            source,
        } => {
            if yes {
                warn!("'-y/--yes' is deprecated, use '-w/--write' instead");
            }
            let mut context = CheckerContext {
                replace,
                write,
                select_first,
                ..Default::default()
            };
            for path in walk(source, recursive) {
                tryb! {
                    let path = path?;
                    let path = &path;
                    if let Some(patched) = rt.check(&mut context, path, true)? {
                        if !context.write {
                            match prompt::prompt_yes_or_no(
                                "Are your sure to write the patched content?",
                                None,
                            )? {
                                YesOrNo::No => return Ok(()),
                                YesOrNo::AllYes => context.write = true,
                                _ => {}
                            }
                        }
                        std::fs::write(path, patched).context(error::Io(path))?;
                    }
                    Ok(())
                }
                .ignore_interrupted()
                .log_error();
            }
        }
        Command::Search {} => {
            rt.prompt_input_icon(None).ok();
        }
    }
    Ok(())
}

fn main() {
    main_impl().log_error();
}
