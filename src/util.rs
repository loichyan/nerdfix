use noodler::NGramSearcher;
use once_cell::unsync::OnceCell;
use std::{cell::Cell, fmt, marker::PhantomData};

#[cfg(test)]
macro_rules! icon {
    ($name:literal, $codepoint:literal) => {
        icon!($name, $codepoint, false)
    };
    ($name:literal, $codepoint:literal, $obsolete:literal) => {
        crate::icon::Icon {
            name: $name.to_owned(),
            codepoint: char::from_u32($codepoint).unwrap(),
            obsolete: $obsolete,
        }
    };
}

/// Prints colored output, `red` color is used by default.
macro_rules! cprintln {
    ($fmt:literal $(,$args:expr)* $(,)?) => {
        cprintln!($fmt.red $(,$args)*);
    };
    ($fmt:literal.$color:ident $(,$args:expr)* $(,)?) => {
        println!("{}", format!($fmt $(,$args)*).$color());
    };
}

/// Logs an error with its source info.
macro_rules! log_error {
    ($e:expr) => {{
        tracing::error!("{}", $crate::util::ErrorWithSource($e));
    }};
}

/// Used in a loop, breaks if interrupted, logs other errors.
macro_rules! log_or_break {
    ($try:block) => {{
        match (|| $try)() {
            Err(crate::error::Error::Interrupted) => break,
            Err(e) => {
                log_error!(e);
                continue;
            }
            Ok(t) => t,
        }
    }};
}

#[derive(Debug)]
pub struct ErrorWithSource<E = crate::error::Error>(pub E);

impl<E: std::error::Error> fmt::Display for ErrorWithSource<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(src) = self.0.source() {
            write!(f, "{}, {src}", self.0)
        } else {
            write!(f, "{}", self.0)
        }
    }
}

impl<E: std::error::Error> std::error::Error for ErrorWithSource<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl From<crate::error::Error> for ErrorWithSource {
    fn from(value: crate::error::Error) -> Self {
        Self(value)
    }
}

pub struct TryLazy<T, E, F> {
    cell: OnceCell<T>,
    init: Cell<Option<F>>,
    _marker: PhantomData<E>,
}

impl<T, E, F: FnOnce() -> Result<T, E>> TryLazy<T, E, F> {
    pub fn new(f: F) -> Self {
        Self {
            cell: OnceCell::default(),
            init: Cell::new(Some(f)),
            _marker: PhantomData,
        }
    }

    pub fn get(&self) -> Result<&T, E> {
        self.cell.get_or_try_init(|| self.init.take().unwrap()())
    }
}

pub trait NGramSearcherExt<'i, 'a, T>: Sized {
    #[doc(hidden)]
    fn __into(self) -> NGramSearcher<'i, 'a, T>;

    fn exec_sorted_stable(self) -> <Vec<(&'i T, f32)> as IntoIterator>::IntoIter
    where
        T: noodler::Keyed + Ord,
    {
        let mut matches = self.__into().exec().collect::<Vec<_>>();
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then_with(|| a.0.cmp(b.0)));
        matches.into_iter()
    }
}

impl<'i, 'a, T> NGramSearcherExt<'i, 'a, T> for NGramSearcher<'i, 'a, T> {
    fn __into(self) -> NGramSearcher<'i, 'a, T> {
        self
    }
}
