use std::fmt;

use noodler::NGramSearcher;

use crate::error;

pub(crate) fn parse_jsonc<T>(str: &str) -> serde_json::Result<T>
where
    T: serde::de::DeserializeOwned,
{
    // Strip comments **in place** so that any return error has the correct
    // codespans.
    let mut str = str.to_owned();
    json_strip_comments::strip_comments_in_place(&mut str, <_>::default(), true).ok();
    serde_json::from_str(&str)
}

pub(crate) struct LogStatus;

const _: () = {
    use std::sync::atomic::{AtomicBool, Ordering};

    use tracing::{Event, Level, Subscriber};
    use tracing_subscriber::layer::{Context, Layer};

    static HAS_ERROR: AtomicBool = AtomicBool::new(false);

    impl<S> Layer<S> for LogStatus
    where
        S: Subscriber,
    {
        fn on_event(&self, ev: &Event<'_>, _ctx: Context<'_, S>) {
            if ev.metadata().level() == &Level::ERROR {
                HAS_ERROR.store(true, Ordering::Release);
            }
        }
    }

    impl LogStatus {
        pub fn has_error() -> bool {
            HAS_ERROR.load(Ordering::Acquire)
        }
    }
};

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

macro_rules! __coloredmsg {
    ($fmt:literal.$color:ident, $($args:expr,)*) => {{
        let color = nu_ansi_term::Color::$color;
        eprintln!(concat!("{}", $fmt, "{}"), color.prefix() $(,$args)* ,color.suffix());
    }};
}

/// Prints an ERROR message.
macro_rules! msgerror {
    ($fmt:literal $(,$args:expr)* $(,)?) => {
        __coloredmsg!($fmt.Red, $($args,)*)
    };
}

/// Prints an INFO message.
macro_rules! msginfo {
    ($fmt:literal $(,$args:expr)* $(,)?) => {
        __coloredmsg!($fmt.Blue, $($args,)*)
    };
}

macro_rules! tri {
    ($block:block) => {
        (|| $block)()
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

#[extend::ext(pub(crate), name = NGramSearcherExt)]
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

#[cfg(test)]
macro_rules! jsonstr {
    ($tt:tt) => {
        stringify!($tt)
    };
}
