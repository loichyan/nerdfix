//! Command line arguments parser.

use crate::{error, icon::Substitution, shadow};
use clap::{Parser, Subcommand, ValueEnum};
use core::fmt;
use shadow_rs::formatcp;
use std::{io, path::PathBuf, str::FromStr};
use thisctx::IntoError;

const V_PATH: &str = "PATH";
const V_SOURCE: &str = "SOURCE";
const V_SUBSTITUTION: &str = "SUBSTITUTION";
const V_FORMAT: &str = "FORMAT";
const INDEX_REV: &str = include_str!("index-rev");
const CLAP_LONG_VERSION: &str = formatcp!("{}\ncheat-sheet: {}", shadow::PKG_VERSION, INDEX_REV);

// TODO: describe implicit behaviors
#[derive(Debug, Parser)]
#[command(author, version, long_version = CLAP_LONG_VERSION)]
pub struct Cli {
    /// Path(s) to load the icons cheat sheet, indices or substitutions.
    #[arg(short, long, global = true, value_name = V_PATH)]
    pub input: Vec<IoPath>,
    /// Perform an exact/prefix substitution.
    ///
    /// For example, use `--sub prefix:mdi-/md-` to replace all `mdi-*`
    /// icons with the same ones in `md-*`.
    #[arg(long, global = true, value_name = V_SUBSTITUTION)]
    pub sub: Vec<Substitution>,
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
    /// [deprecated] Use `--input` instead.
    ///
    /// A substitutions list is a json object whose key is icon name and whose
    /// value is a list of icons used to replace the icon.
    #[arg(long, global = true, value_name = V_PATH)]
    pub substitution: Vec<IoPath>,
    /// [deprecated] Use `--sub prefix:` instead.
    #[arg(long, global = true, value_name = V_SUBSTITUTION)]
    pub replace: Vec<Substitution>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// [deprecated] Use `index` instead.
    Cache {
        /// Path to save the cached content.
        #[arg(short, long, value_name = V_PATH,)]
        output: IoPath,
    },
    /// Generate icons indices.
    Index {
        /// Path to save the output.
        #[arg(short, long, value_name = V_PATH)]
        output: IoPath,
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
        source: Vec<IoPath>,
    },
    /// Fix obsolete icons interactively.
    Fix {
        /// [deprecated] Use `--write` instead.
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
        /// Each tuple is an input path followed by an optional output path,
        /// e.g. `nerdfix fix /input/as/ouput /read/from:/write/to`.
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
pub enum IoPath {
    Stdio,
    Path(PathBuf),
}

impl FromStr for IoPath {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "-" => Ok(Self::Stdio),
            _ if s.is_empty() => Err("empty path is not allowed"),
            _ => Ok(Self::Path(s.into())),
        }
    }
}

impl fmt::Display for IoPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stdio => f.write_str("STDIO"),
            Self::Path(path) => path.display().fmt(f),
        }
    }
}

impl IoPath {
    pub fn read_to_string(&self) -> io::Result<String> {
        match self {
            IoPath::Stdio => io::read_to_string(io::stdin()),
            IoPath::Path(path) => std::fs::read_to_string(path),
        }
    }

    pub fn write_str(&self, content: &str) -> io::Result<()> {
        match self {
            IoPath::Stdio => io::Write::write_all(&mut io::stdout(), content.as_bytes()),
            IoPath::Path(path) => std::fs::write(path, content),
        }
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
pub struct Source(pub IoPath, pub Option<IoPath>);

impl FromStr for Source {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if let Some((input, output)) = s.split_once(':') {
            Source(input.parse()?, Some(output.parse()?))
        } else {
            Source(s.parse()?, None)
        })
    }
}
