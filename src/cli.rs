//! Command line arguments parser.

use std::io::BufReader;
use std::str::FromStr;
use std::{fmt, fs, io};

use bytesize::ByteSize;
use camino::Utf8PathBuf;
use clap::{Parser, Subcommand, ValueEnum};
use clap_complete::Shell;
use derive_more::Display;
use shadow_rs::formatcp;
use thisctx::IntoError;

use crate::icon::Substitution;
use crate::input::InputReader;
use crate::{error, shadow};

const V_FORMAT: &str = "FORMAT";
const V_PATH: &str = "PATH";
const V_SHELL: &str = "SHELL";
const V_SIZE: &str = "SIZE";
const V_SOURCE: &str = "SOURCE";
const V_SUB: &str = "SUB";
const V_VERSION: &str = "VERSION";
const DEFAULT_SIZE: &str = "16MB";
const INDEX_REV: &str = include_str!("index-rev");
const CLAP_LONG_VERSION: &str = formatcp!("{}\ncheat-sheet: {}", shadow::PKG_VERSION, INDEX_REV);

const SUB_LONG_HELP: &str = "\
Perform an exact/prefix substitution.

This option accepts several substitution types of `TYPE:FROM/TO` syntax:
  * Exact substitution: replaces an icon with another when its name matches exactly. This is the \
                             default type when `TYPE` is omitted.
  * Prefix substitution: replaces the prefix of an icon name with another, and then tries to \
                             replace the icon with the one has the new name, e.g. use `--sub \
                             prefix:mdi-/md-` to replace `mdi-tab` with `md-tab`.";

#[derive(Debug, Parser)]
#[command(author, version, long_version = CLAP_LONG_VERSION)]
pub struct Cli {
    /// Path(s) to load the icons cheat sheet, indices or substitutions.
    ///
    /// Note that builtin icons and substitution lists are not loaded if any
    /// custom database is provided. You can run `nerdfix dump` to get and load
    /// them at first.
    #[arg(short, long, global = true, value_name = V_PATH)]
    pub input: Vec<IoPath>,
    /// The version of Nerd Fonts you intend to migrate to.
    #[arg(long, global = true, value_name = V_VERSION, default_value_t = NfVersion::default())]
    pub nf_version: NfVersion,
    /// Perform an exact/prefix substitution.
    #[arg(long, global = true, value_name = V_SUB, long_help = SUB_LONG_HELP)]
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
    #[arg(long, global = true, value_name = V_PATH)]
    pub substitution: Vec<IoPath>,
    /// [deprecated] Use `--sub prefix:` instead.
    #[arg(long, global = true, value_name = V_SUB)]
    pub replace: Vec<Substitution>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// [deprecated] Use `dump` instead.
    Cache {
        /// Path to save the output.
        #[arg(short, long, value_name = V_PATH,)]
        output: IoPath,
    },
    /// Dump records in current database.
    Dump {
        /// Path to save the output.
        #[arg(short, long, value_name = V_PATH)]
        output: IoPath,
    },
    /// Check for obsolete icons.
    Check {
        /// Output format of diagnostics.
        #[arg(long, value_name = V_FORMAT, default_value_t = OutputFormat::default())]
        format: OutputFormat,
        /// Recursively traverse all directories.
        #[arg(short, long)]
        recursive: bool,
        /// Do not skip binary files.
        #[arg(long)]
        include_binary: bool,
        /// Set the file size limit (0 to disable it).
        #[arg(long, value_name= V_SIZE, default_value = DEFAULT_SIZE)]
        size_limit: ByteSize,
        /// Path(s) of files to check.
        #[arg(value_name = V_PATH)]
        source: Vec<IoPath>,
    },
    /// Fix obsolete icons interactively.
    Fix {
        /// [deprecated] Use `--write` instead.
        #[arg(short, long)]
        yes: bool,
        /// Write output without confirmation.
        #[arg(short, long)]
        write: bool,
        /// Select the first (also the most similar) one for all suggestions.
        #[arg(long)]
        select_first: bool,
        /// Recursively traverse all directories.
        #[arg(short, long)]
        recursive: bool,
        /// Do not skip binary files.
        #[arg(long)]
        include_binary: bool,
        /// Set the file size limit (0 to disable it).
        #[arg(long, value_name= V_SIZE, default_value = DEFAULT_SIZE)]
        size_limit: ByteSize,
        /// Save fixed files to different paths.
        ///
        /// Each path should be paired with its corresponding source path. Use
        /// empty strings to save output directly to the source path. For
        /// example, `nerdfix fix -o output1 -o "" input1 input2` will save
        /// `input1` to `output1` and save `input2` to its original path.
        #[arg(short, long, value_name = V_PATH)]
        output: Vec<Outpath>,
        /// Path(s) of files to check.
        #[arg(value_name = V_SOURCE)]
        source: Vec<IoPath>,
    },
    /// Query icon infos from the database.
    ///
    /// If no argument is specified, it will open a prompt for interactive fuzzy
    /// search.
    Search {
        /// Search for icon of the given codepoint, returned in JSON if matches.
        #[arg(long, value_parser = crate::icon::parse_codepoint)]
        codepoint: Option<char>,
        /// Search for icon of the given name, returned in JSON if matches.
        #[arg(long, conflicts_with = "codepoint")]
        name: Option<String>,
    },
    /// Generate shell completions for your shell to stdout.
    Completions {
        #[arg(value_name = V_SHELL)]
        shell: Shell,
    },
}

#[derive(Clone, Copy, Debug, Display, Default, Eq, PartialEq, ValueEnum)]
pub enum NfVersion {
    #[default]
    #[value(name = "3.0.0")]
    #[display("3.0.0")]
    V3_0_0,
    #[value(name = "3.3.0")]
    #[display("3.3.0")]
    V3_3_0,
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
    Path(Utf8PathBuf),
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
            Self::Path(path) => path.fmt(f),
        }
    }
}

impl IoPath {
    pub fn metadata(&self) -> io::Result<Option<fs::Metadata>> {
        if let IoPath::Path(path) = self {
            fs::metadata(path).map(Some)
        } else {
            Ok(None)
        }
    }

    pub fn file_size(&self) -> io::Result<Option<u64>> {
        self.metadata().map(|t| t.map(|m| m.len()))
    }

    fn get_reader(&self) -> io::Result<Box<dyn io::BufRead>> {
        Ok(match self {
            IoPath::Stdio => Box::new(BufReader::new(io::stdin())) as _,
            IoPath::Path(path) => Box::new(BufReader::new(fs::File::open(path)?)) as _,
        })
    }

    pub fn open(&self) -> io::Result<InputReader> {
        self.get_reader().map(InputReader::new)
    }

    pub fn read_to_string(&self) -> io::Result<String> {
        self.get_reader().and_then(io::read_to_string)
    }

    pub fn write_str(&self, content: &str) -> io::Result<()> {
        match self {
            IoPath::Stdio => io::Write::write_all(&mut io::stdout(), content.as_bytes()),
            IoPath::Path(path) => fs::write(path, content),
        }
    }
}

#[derive(Clone, Copy, Debug, Display, Default, Eq, PartialEq, ValueEnum)]
pub enum OutputFormat {
    #[value(help = "Json output format")]
    #[display("json")]
    Json,
    #[default]
    #[value(help = "Human-readable output format")]
    #[display("console")]
    Console,
}

#[derive(Clone, Debug)]
pub struct Outpath(pub Option<IoPath>);

impl FromStr for Outpath {
    type Err = <IoPath as FromStr>::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(Self(None))
        } else {
            s.parse().map(Some).map(Self)
        }
    }
}

#[derive(Debug)]
pub struct Source(pub IoPath, pub Option<IoPath>);
