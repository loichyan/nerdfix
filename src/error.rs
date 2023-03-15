use std::path::PathBuf;

use thisctx::WithContext;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, WithContext)]
#[thisctx(pub(crate), suffix(false))]
pub enum Error {
    #[error("IO failed at '{1}'")]
    Io(#[source] std::io::Error, PathBuf),
    #[error("{0} at '{1}:{2}'")]
    CorruptedCache(String, PathBuf, usize),
    #[error("Failed when reporting diagnostics")]
    Reporter(#[source] codespan_reporting::files::Error),
    #[error("Failed when prompting")]
    Prompt(#[source] inquire::InquireError),
    #[error("Invalid input")]
    InvalidInput,
    #[error("Operation was interrupted by the user")]
    Interrupted,
}
