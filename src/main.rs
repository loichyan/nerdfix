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
use cli::{Command, Source};
use prompt::YesOrNo;
use runtime::{CheckerContext, Runtime};
use thisctx::WithContext;
use tracing::{error, info, warn, Level};
use util::ResultExt;
use walkdir::WalkDir;

static INDICES: &str = include_str!("./index.json");
static SUBSTITUTIONS: &str = include_str!("./substitution.json");

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
                        warn!(
                            // TODO: replace 'if' with 'when'
                            "Output path is ignored if '--recursive': {}",
                            output.display()
                        );
                    }

                    WalkDir::new(input)
                })
                .filter_map(|entry| {
                    tryb!({
                        let path = entry?.into_path();
                        if path.is_file() {
                            Ok(Some(path))
                        } else {
                            Ok(None)
                        }
                    })
                    .transpose()
                })
                .map(|e| e.map(|path| Source(path, None))),
        )
    }
}

fn main_impl() -> error::Result<()> {
    let args = cli::Cli::parse();

    let lv = match args.verbose - args.quiet {
        0 => Level::ERROR,
        1 => Level::WARN,
        2 => Level::INFO,
        3 => Level::DEBUG,
        _ => Level::TRACE,
    };
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_writer(std::io::stderr)
        .with_max_level(lv)
        .without_time()
        .finish();
    tracing::subscriber::set_global_default(subscriber).context(error::Any)?;

    let mut rt = Runtime::builder();
    rt.with_replacements(args.replace);
    if args.input.is_empty() {
        rt.load_input(INDICES).unwrap();
    } else {
        for path in args.input.iter() {
            rt.load_input_file(path)?;
        }
    }
    if args.substitution.is_empty() {
        rt.load_substitutions(SUBSTITUTIONS).unwrap();
    } else {
        for path in args.substitution.iter() {
            rt.load_substitutions_file(path)?;
        }
    }

    match args.cmd {
        Command::Cache { .. } => warn!("'cache' is deprecated, use 'index' instead"),
        Command::Index { output } => rt.build().generate_indices(&output)?,
        Command::Check {
            format,
            source,
            recursive,
        } => {
            let rt = rt.build();
            let mut context = CheckerContext {
                format,
                ..Default::default()
            };
            for source in walk(source.into_iter().map(|p| Source(p, None)), recursive) {
                tryb!({
                    let source = source?;
                    rt.check(&mut context, &source.0, false)
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
            source,
        } => {
            if yes {
                warn!("'-y/--yes' is deprecated, use '-w/--write' instead");
            }
            let rt = rt.build();
            let mut context = CheckerContext {
                write,
                select_first,
                ..Default::default()
            };
            for source in walk(source, recursive) {
                tryb!({
                    let source = source?;
                    let Source(input, output) = &source;
                    let output = output.as_ref().unwrap_or(input);
                    if let Some(patched) = rt.check(&mut context, input, true)? {
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
                        info!("Write output to '{}'", output.display());
                        std::fs::write(output, patched)?;
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
}
