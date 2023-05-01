use std::path::{Path, PathBuf};
use thisctx::WithContext;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

fn with_opt_path(path: &Option<PathBuf>) -> String {
    path.as_deref()
        .map(|p| format!(" at '{}'", p.display()))
        .unwrap_or_default()
}

fn with_opt_path_line(path: &Option<PathBuf>, line: &usize) -> String {
    path.as_deref()
        .map(|p| format!(" at '{}:{line}'", p.display()))
        .unwrap_or_else(|| format!(" at line {line}"))
}

#[derive(Debug, Error, WithContext)]
#[thisctx(pub(crate))]
pub enum Error {
    #[error("Io failed{}", with_opt_path(.1))]
    Io(#[source] std::io::Error, Option<PathBuf>),
    #[error("{0}{}", with_opt_path_line(.1, .2))]
    CorruptedCache(String, Option<PathBuf>, usize),
    #[error("Failed when reporting diagnostics")]
    Reporter(
        #[from]
        #[source]
        codespan_reporting::files::Error,
    ),
    #[error("Failed when prompting")]
    Prompt(
        #[from]
        #[source]
        inquire::InquireError,
    ),
    #[error("Invalid input")]
    InvalidInput,
    #[error("Operation was interrupted by the user")]
    Interrupted,
    #[error(transparent)]
    Any(Box<dyn Send + Sync + std::error::Error>),
}

#[extend::ext(pub(crate))]
impl<T> Result<T> {
    fn with_path(self, path: &Path) -> Self {
        self.map_err(|e| match e {
            Error::Io(e, None) => Error::Io(e, Some(path.to_owned())),
            Error::CorruptedCache(e, None, i) => Error::CorruptedCache(e, Some(path.to_owned()), i),
            _ => e,
        })
    }
}
