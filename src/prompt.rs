use crate::error;
use inquire::InquireError;
use std::{fmt, str::FromStr};
use thisctx::IntoError;

pub fn prompt_yes_or_no(msg: &str, help: Option<&str>) -> error::Result<YesOrNo> {
    match inquire::CustomType::<YesOrNo>::new(msg)
        .with_help_message(help.unwrap_or("(Y)es/(N)o/(A)ll yes, (Ctrl-C) to abort"))
        .prompt()
    {
        Err(InquireError::OperationInterrupted) => error::Interrupted.fail(),
        t => Ok(t?),
    }
}

#[derive(Clone, Copy, Debug)]
pub enum YesOrNo {
    Yes,
    No,
    AllYes,
}

impl fmt::Display for YesOrNo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            YesOrNo::Yes => write!(f, "yes"),
            YesOrNo::No => write!(f, "no"),
            YesOrNo::AllYes => write!(f, "all yes"),
        }
    }
}

impl FromStr for YesOrNo {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "y" | "yes" => Ok(Self::Yes),
            "n" | "no" => Ok(Self::No),
            "a" | "all" => Ok(Self::AllYes),
            _ => Err("invalid input"),
        }
    }
}
