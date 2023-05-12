use crate::error;
use noodler::NGramSearcher;
use once_cell::unsync::OnceCell;
use std::{cell::Cell, fmt, marker::PhantomData, path::Path};

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

macro_rules! tryb {
    ($($stmt:stmt)*) => {
        #[allow(redundant_semicolons)]
        (|| -> ::core::result::Result<_, _> { $($stmt)* })()
    };
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

#[extend::ext(pub, name = NGramSearcherExt)]
impl<'i, 'a, T> NGramSearcher<'i, 'a, T> {
    fn exec_sorted_stable(self) -> <Vec<(&'i T, f32)> as IntoIterator>::IntoIter
    where
        T: noodler::Keyed + Ord,
    {
        let mut matches = self.exec().collect::<Vec<_>>();
        matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap().then_with(|| a.0.cmp(b.0)));
        matches.into_iter()
    }
}

#[extend::ext(pub(crate), name = ResultExt)]
impl<T> error::Result<T> {
    fn with_path(self, path: &Path) -> Self {
        use error::Error::*;

        self.map_err(|e| match e {
            Io(e, error::IoNone) => Io(e, path.into()),
            CorruptedCache(e, error::IoNone, i) => CorruptedCache(e, path.into(), i),
            _ => e,
        })
    }

    fn ignore_interrupted(self) -> error::Result<Option<T>> {
        use error::Error::*;

        match self {
            Ok(t) => Ok(Some(t)),
            Err(Interrupted) => Ok(None),
            Err(e) => Err(e),
        }
    }

    fn log_error(self) -> Option<T> {
        match self {
            Ok(t) => Some(t),
            Err(e) => {
                tracing::error!("{}", ErrorWithSource(e));
                None
            }
        }
    }
}
