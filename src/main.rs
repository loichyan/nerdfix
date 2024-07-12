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
use cli::{Command, IoPath, Source};
use prompt::YesOrNo;
use runtime::{CheckerContext, Runtime};
use thisctx::WithContext;
use tracing::{error, info, warn, Level};
use tracing_subscriber::prelude::*;
use util::{LogStatus, ResultExt as _};
use walkdir::WalkDir;

static ICONS: &str = include_str!("./icons.json");
static SUBSTITUTIONS: &str = include_str!("./substitutions.json");

fn walk<'a>(
    paths: impl 'a + IntoIterator<Item = Source>,
    recursive: bool,
) -> impl 'a + Iterator<Item = error::Result<Source>> {
    if !recursive {
        Box::new(paths.into_iter().map(Ok)) as Box<dyn Iterator<Item = _>>
    } else {
        Box::new(
            paths
                .into_iter()
                .flat_map(|Source(input, output)| {
                    if let Some(output) = output {
                        warn!("Output path is ignored when `--recursive`: {}", output);
                    }
                    if let IoPath::Path(p) = input {
                        Some(WalkDir::new(p))
                    } else {
                        warn!("STDIO path is ignored when `--recursive`");
                        None
                    }
                    .into_iter()
                    .flat_map(|w| w.into_iter())
                })
                .filter_map(|entry| {
                    tri!({
                        let path = entry?.into_path();
                        if path.is_file() {
                            Ok(Some(path))
                        } else {
                            Ok(None)
                        }
                    })
                    .transpose()
                })
                .map(|e| e.map(|path| Source(IoPath::Path(path), None))),
        )
    }
}

fn main_impl() -> error::Result<()> {
    let args = cli::Cli::parse();

    let lv = match args.verbose.saturating_sub(args.quiet) {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let subscriber = tracing_subscriber::fmt()
        .compact()
        .with_file(false)
        .with_max_level(lv)
        .with_target(false)
        .with_writer(std::io::stderr)
        .without_time()
        .finish()
        .with(LogStatus);
    tracing::subscriber::set_global_default(subscriber).context(error::Any)?;

    let mut rt = Runtime::builder();
    if !args.substitution.is_empty() {
        warn!("`--substitution` is deprecated, use `--sub` instead");
    }
    if !args.replace.is_empty() {
        warn!("`--replace` is deprecated, use `--sub prefix:` instead");
    }
    // Ignore builtin database when user provides input.
    if args.input.is_empty() {
        rt.load_db(ICONS).unwrap();
        rt.load_db(SUBSTITUTIONS).unwrap();
    }
    for input in args.input.iter() {
        rt.load_db_from(input)?;
    }
    rt.with_substitutions(args.sub);

    match args.cmd {
        Command::Cache { .. } => warn!("`cache` is deprecated, use `dump` instead"),
        Command::Dump { output } => rt.build().dump_db(&output)?,
        Command::Check {
            format,
            source,
            recursive,
            include_binary,
        } => {
            let rt = rt.build();
            let mut context = CheckerContext {
                format,
                writer: Box::new(std::io::stdout()),
                include_binary,
                ..Default::default()
            };
            for source in walk(source.into_iter().map(|p| Source(p, None)), recursive) {
                tri!({
                    let source = source?;
                    rt.check(&mut context, &source.0, None)
                })
                .ignore_interrupted()
                .log_error();
            }
        }
        Command::Fix {
            yes,
            write,
            select_first,
            recursive,
            include_binary,
            source,
        } => {
            if yes {
                warn!("`--yes` is deprecated, use `--write` instead");
            }
            let rt = rt.build();
            let mut context = CheckerContext {
                write,
                select_first,
                include_binary,
                ..Default::default()
            };
            let mut buffer = String::new();
            for source in walk(source, recursive) {
                tri!({
                    let source = source?;
                    let Source(input, output) = &source;
                    let output = output.as_ref().unwrap_or(input);
                    buffer.clear();
                    if rt.check(&mut context, input, Some(&mut buffer))? {
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
                        info!("Write output to '{}'", output);
                        output.write_str(&buffer)?;
                    }
                    Ok(())
                })
                .ignore_interrupted()
                .log_error();
            }
        }
        Command::Search {} => {
            rt.build().prompt_input_icon(None).ok();
        }
    }
    Ok(())
}

fn main() {
    main_impl().log_error();
    if LogStatus::has_error() {
        std::process::exit(1);
    }
}
