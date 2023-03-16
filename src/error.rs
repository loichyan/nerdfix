use std::path::{Path, PathBuf};
use thisctx::WithContext;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

fn fmt_path_line(path: &Option<PathBuf>, line: &usize) -> String {
    if let Some(path) = path {
        format!("'{}:{line}'", path.display())
    } else {
        format!("line {line}")
    }
}

#[derive(Debug, Error, WithContext)]
#[thisctx(pub(crate), suffix(false))]
pub enum Error {
    #[error("IO failed at '{1}'")]
    Io(#[source] std::io::Error, PathBuf),
    #[error("{0} at {}", fmt_path_line(.1, .2))]
    CorruptedCache(String, Option<PathBuf>, usize),
    #[error("Failed when reporting diagnostics")]
    Reporter(#[source] codespan_reporting::files::Error),
    #[error("Failed when prompting")]
    Prompt(#[source] inquire::InquireError),
    #[error("Invalid input")]
    InvalidInput,
    #[error("Operation was interrupted by the user")]
    Interrupted,
}

impl Error {
    pub(crate) fn with_path(self, path: &Path) -> Self {
        match self {
            Self::CorruptedCache(e, _, i) => Self::CorruptedCache(e, Some(path.to_owned()), i),
            _ => self,
        }
    }
}
