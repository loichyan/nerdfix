//! Command line arguments parser.

use crate::{error, shadow};
use clap::{Parser, Subcommand, ValueEnum};
use shadow_rs::formatcp;
use std::{path::PathBuf, str::FromStr};
use thisctx::IntoError;

const V_PATH: &str = "PATH";
const V_SOURCE: &str = "SOURCE";
const V_CLASSES: &str = "FROM,TO";
const V_FORMAT: &str = "FORMAT";
const CACHE_VERSION: &str = include_str!("cache-version");
const CLAP_LONG_VERSION: &str =
    formatcp!("{}\ncheat-sheet: {}", shadow::PKG_VERSION, CACHE_VERSION);

#[derive(Debug, Parser)]
#[command(author, version, long_version = CLAP_LONG_VERSION)]
pub struct Cli {
    /// Path(s) to load the icons cheat sheet or cached content.
    #[arg(short, long, value_name(V_PATH))]
    pub input: Vec<PathBuf>,
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Cache parsed icons.
    Cache {
        /// Path to save the cached content.
        #[arg(short, long, value_name(V_PATH))]
        output: PathBuf,
    },
    /// Check for obsolete icons.
    Check {
        /// Output format of diagnostics.
        #[arg(long, value_name(V_FORMAT), default_value("console"))]
        format: OutputFormat,
        /// Recursively traverse all directories.
        #[arg(short, long)]
        recursive: bool,
        /// Path(s) of files to check.
        #[arg(value_name(V_PATH))]
        source: Vec<PathBuf>,
    },
    /// Fix obsolete icons interactively.
    Fix {
        /// Deprecated. Use `-w/--write` instead.
        #[arg(short, long)]
        yes: bool,
        /// Write content without confirmation.
        #[arg(short, long)]
        write: bool,
        /// Select the first (and most similar) one for all suggestions.
        #[arg(long)]
        select_first: bool,
        /// Replace the prefix of an icon name with another.
        ///
        /// For example, use `--replace=nf-mdi,nf-md` to replace all `nf-mdi*`
        /// icons with the same ones in `nf-md*`.
        #[arg(long, value_name(V_CLASSES))]
        replace: Vec<Replace>,
        /// Recursively traverse all directories.
        #[arg(short, long)]
        recursive: bool,
        /// Path tuple(s) of files to read from and write to.
        ///
        /// Each tuple is an input path followed by an optional output path, e.g.
        /// `/input/as/ouput /read/from:/write/to`.
        #[arg(value_name(V_SOURCE))]
        source: Vec<Source>,
    },
    /// Fuzzy search for an icon.
    Search {},
}

pub enum UserInput {
    Candidate(usize),
    Name(String),
    Codepoint(u32),
    Char(char),
}

impl FromStr for UserInput {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err(error::InvalidInput.build())
        } else if let Some(codepoint) = s.strip_prefix("u+").or_else(|| s.strip_prefix("U+")) {
            u32::from_str_radix(codepoint, 16)
                .map_err(|_| error::InvalidInput.build())
                .map(Self::Codepoint)
        } else if matches!(s.as_bytes().first(), Some(b'0'..=b'9')) {
            usize::from_str(s)
                .map_err(|_| error::InvalidInput.build())
                .map(Self::Candidate)
        } else if s.chars().count() == 1 {
            Ok(Self::Char(s.chars().next().unwrap()))
        } else {
            Ok(Self::Name(s.to_owned()))
        }
    }
}

#[derive(Clone, Debug)]
pub struct Replace {
    pub from: String,
    pub to: String,
}

impl FromStr for Replace {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let (from, to) = s
            .split_once(',')
            .ok_or("the input should be two classes separated by a comma")?;
        Ok(Self {
            from: from.to_owned(),
            to: to.to_owned(),
        })
    }
}

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum OutputFormat {
    #[value(help("Json output format"))]
    Json,
    #[default]
    #[value(help("Human-readable output format"))]
    Console,
}

#[derive(Clone, Debug)]
pub struct Source(pub PathBuf, pub Option<PathBuf>);

impl FromStr for Source {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(s.split_once(':')
            .map(|(input, output)| Source(input.into(), Some(output.into())))
            .unwrap_or_else(|| Source(s.into(), None)))
    }
}
