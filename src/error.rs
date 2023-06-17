use thisctx::WithContext;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error, WithContext)]
#[thisctx(pub(crate))]
pub enum Error {
    #[error("Io failed")]
    Io(
        #[from]
        #[source]
        std::io::Error,
    ),
    #[error("Failed when traversing directories")]
    Walkdir(
        #[from]
        #[source]
        walkdir::Error,
    ),
    #[error("{0} at line {1}")]
    CorruptedCache(String, #[thisctx(no_generic)] usize),
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
