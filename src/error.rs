use derive_more::{Display, From};
use std::path::{Path, PathBuf};
use thisctx::WithContext;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub(crate) use IoSource::{None as IoNone, Stdio};

#[derive(Debug, Display, From)]
pub enum IoSource {
    #[from]
    #[display(fmt = "{}", "_0.display()")]
    Path(PathBuf),
    #[display(fmt = "<STDIO>")]
    Stdio,
    #[display(fmt = "<NONE>")]
    None,
}

impl From<&Path> for IoSource {
    fn from(value: &Path) -> Self {
        value.to_owned().into()
    }
}

#[derive(Debug, Error, WithContext)]
#[thisctx(pub(crate))]
pub enum Error {
    #[error("Io failed at {1}")]
    Io(#[source] std::io::Error, IoSource),
    #[error("{0} at {1}:{2}")]
    CorruptedCache(String, IoSource, usize),
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
            Error::Io(e, IoNone) => Error::Io(e, path.into()),
            Error::CorruptedCache(e, IoNone, i) => Error::CorruptedCache(e, path.into(), i),
            _ => e,
        })
    }
}
