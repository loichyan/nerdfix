//! Command line arguments parser.

use crate::{error, shadow};
use clap::{Parser, Subcommand, ValueEnum};
use core::fmt;
use shadow_rs::formatcp;
use std::{path::PathBuf, str::FromStr};
use thisctx::IntoError;

const V_PATH: &str = "PATH";
const V_SOURCE: &str = "SOURCE";
const V_CLASSES: &str = "FROM,TO";
const V_FORMAT: &str = "FORMAT";
const INDEX_REV: &str = include_str!("index-rev");
const CLAP_LONG_VERSION: &str = formatcp!("{}\ncheat-sheet: {}", shadow::PKG_VERSION, INDEX_REV);

#[derive(Debug, Parser)]
#[command(author, version, long_version = CLAP_LONG_VERSION)]
pub struct Cli {
    /// Path(s) to load the icons cheat sheet, indices or substitutions.
    #[arg(short, long, global = true, value_name = V_PATH)]
    pub input: Vec<InputFrom>,
    /// Replace the prefix of an icon name with another.
    ///
    /// For example, use `--replace=mdi,md` to replace all `mdi*`
    /// icons with the same ones in `md*`.
    #[arg(long, global = true, value_name = V_CLASSES)]
    pub replace: Vec<Replacement>,
    /// [deprecated] Load predfined substitutions lists used in autofix.
    ///
    /// A substitutions list is a json object whose key is icon name and whose
    /// value is a list of icons used to replace the icon.
    #[arg(long, global = true, value_name = V_PATH)]
    pub substitution: Vec<PathBuf>,
    /// Decrease log level.
    #[arg(
        short,
        long,
        global = true,
        action = clap::ArgAction::Count,
        default_value_t = 0
    )]
    pub quiet: u8,
    /// Increase log level.
    #[arg(
        short,
        long,
        global = true,
        action = clap::ArgAction::Count,
        default_value_t = 2
    )]
    pub verbose: u8,
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// [deprecated] Cache parsed icons.
    Cache {
        /// Path to save the cached content.
        #[arg(short, long, value_name = V_PATH,)]
        output: PathBuf,
    },
    /// Generate icons indices.
    Index {
        /// Path to save the output.
        #[arg(short, long, value_name = V_PATH)]
        output: PathBuf,
    },
    /// Check for obsolete icons.
    Check {
        /// Output format of diagnostics.
        #[arg(long, value_name = V_FORMAT, default_value_t = OutputFormat::Console)]
        format: OutputFormat,
        /// Recursively traverse all directories.
        #[arg(short, long)]
        recursive: bool,
        /// Path(s) of files to check.
        #[arg(value_name = V_PATH)]
        source: Vec<PathBuf>,
    },
    /// Fix obsolete icons interactively.
    Fix {
        /// [deprecated] Write content without confirmation.
        #[arg(short, long)]
        yes: bool,
        /// Write content without confirmation.
        #[arg(short, long)]
        write: bool,
        /// Select the first (and most similar) one for all suggestions.
        #[arg(long)]
        select_first: bool,
        /// Recursively traverse all directories.
        #[arg(short, long)]
        recursive: bool,
        /// Path tuple(s) of files to read from and write to.
        ///
        /// Each tuple is an input path followed by an optional output path, e.g.
        /// `/input/as/ouput /read/from:/write/to`.
        #[arg(value_name = V_SOURCE)]
        source: Vec<Source>,
    },
    /// Fuzzy search for an icon.
    Search {},
}

#[derive(Clone, Debug)]
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
pub enum InputFrom {
    Stdin,
    Indices,
    Substitutions,
    File(PathBuf),
}

impl FromStr for InputFrom {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" | "STDIN" => Ok(Self::Stdin),
            "INDICES" => Ok(Self::Indices),
            "SUBSTITUTIONS" => Ok(Self::Substitutions),
            _ => Ok(Self::File(s.into())),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Replacement {
    pub from: String,
    pub to: String,
}

impl FromStr for Replacement {
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

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json => f.write_str("json"),
            Self::Console => f.write_str("console"),
        }
    }
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
