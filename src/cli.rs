//! Command line arguments parser.

use crate::error;
use clap::{Parser, Subcommand};
use std::{path::PathBuf, str::FromStr};
use thisctx::IntoError;

const V_PATH: &str = "PATH";

#[derive(Debug, Parser)]
pub struct Cli {
    /// Path(s) to load the icons cheat sheet or cached content.
    #[arg(short, long, value_name(V_PATH))]
    pub input: Vec<PathBuf>,
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
        /// Path(s) of files to check.
        #[arg(value_name(V_PATH))]
        source: Vec<PathBuf>,
    },
    /// Fix obsolete icons interactively.
    Fix {
        /// Path(s) of files to check.
        #[arg(value_name(V_PATH))]
        source: Vec<PathBuf>,
        /// Auto-confirm interactive prompts.
        #[arg(short, long)]
        yes: bool,
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
