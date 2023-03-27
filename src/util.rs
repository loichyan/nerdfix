use std::fmt;

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
    ($res:expr) => {{
        match $res {
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
