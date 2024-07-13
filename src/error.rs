use std::fmt;

use miette::{Diagnostic, LabeledSpan, MietteSpanContents, SourceCode, SpanContents};
use thisctx::WithContext;
use thiserror::Error;

use crate::cli::IoPath;
use crate::icon::Icon;
use crate::input::InputLine;
use crate::runtime::Severity;

pub type Result<T, E = Error> = std::result::Result<T, E>;

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
    #[error("Invalid cheat sheet at line {0}")]
    InvalidCheatSheet(#[thisctx(no_generic)] usize),
    #[error("Failed to parse json")]
    Json(
        #[from]
        #[source]
        serde_json::Error,
    ),
    #[error("Failed when prompting")]
    Prompt(
        #[from]
        #[source]
        inquire::InquireError,
    ),
    #[error("Invalid UTF-8 input")]
    Utf8(
        #[from]
        #[source]
        std::str::Utf8Error,
    ),
    #[error("Invalid input")]
    InvalidInput,
    #[error("Invalid codepoint")]
    InvalidCodepoint,
    #[error("Operation was interrupted by the user")]
    Interrupted,
    #[error(transparent)]
    Any(Box<dyn Send + Sync + std::error::Error>),
}

#[derive(Debug, Error)]
pub(crate) struct ObsoleteIcon<'a> {
    pub input: &'a InputLine<'a>,
    pub path: &'a IoPath,
    pub icon: &'a Icon,
    pub span: (usize, usize),
    pub candidates: &'a [&'a Icon],
}

impl fmt::Display for ObsoleteIcon<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Found obsolete icon U+{:X}", self.icon.codepoint as u32)
    }
}

impl SourceCode for ObsoleteIcon<'_> {
    fn read_span<'a>(
        &'a self,
        span: &miette::SourceSpan,
        lines_before: usize,
        lines_after: usize,
    ) -> std::result::Result<Box<dyn SpanContents<'a> + 'a>, miette::MietteError> {
        let contents = self.input.read_span(span, lines_before, lines_after)?;
        Ok(Box::new(MietteSpanContents::new_named(
            self.path.to_string(),
            contents.data(),
            *contents.span(),
            contents.line(),
            contents.column(),
            contents.line_count(),
        )))
    }
}

impl Diagnostic for ObsoleteIcon<'_> {
    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(self)
    }

    fn severity(&self) -> Option<miette::Severity> {
        Some(Severity::Info.into())
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        Some(Box::new(std::iter::once(LabeledSpan::at(
            self.span.0..self.span.1,
            format!("Icon '{}' is marked as obsolete", self.icon.name),
        ))))
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        struct DiagnosticNotes<'a>(&'a [&'a Icon]);
        impl fmt::Display for DiagnosticNotes<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                writeln!(f, "You could replace it with:")?;
                for (i, &candi) in self.0.iter().enumerate() {
                    let i = i + 1;
                    write!(
                        f,
                        "  {}. {} U+{:05X} {}",
                        i, candi.codepoint, candi.codepoint as u32, &candi.name
                    )?;
                    if i < self.0.len() {
                        f.write_str("\n")?;
                    }
                }
                Ok(())
            }
        }
        Some(Box::new(DiagnosticNotes(self.candidates)))
    }
}
