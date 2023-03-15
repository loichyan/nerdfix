//! Database of nerd font icons infomation.

use std::{borrow::Borrow, fmt, str::FromStr};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Icon {
    pub name: String,
    pub codepoint: char,
    pub obsolete: bool,
}

pub struct CachedIcon<T = Icon>(pub T);

impl FromStr for CachedIcon {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut splited = s.split_ascii_whitespace();
        let name = splited.next().ok_or("Miss field 'name'")?.to_owned();
        let codepoint = splited.next().ok_or("Miss field 'codepoint'")?;
        let codepoint =
            u32::from_str_radix(codepoint, 16).map_err(|_| "Invalid field 'codepoint'")?;
        let codepoint = char::from_u32(codepoint).ok_or_else(|| "Invalid field 'codepoint'")?;
        let obsolete = match splited.next() {
            Some(s) if s == "obsolete" => true,
            Some(_) => return Err("Invalid field 'obsolete'"),
            None => false,
        };
        Ok(Self(Icon {
            name,
            codepoint,
            obsolete,
        }))
    }
}

impl<T: Borrow<Icon>> fmt::Display for CachedIcon<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let icon: &Icon = self.0.borrow();
        write!(f, "{} {:x}", icon.name, icon.codepoint as u32)?;
        if icon.obsolete {
            write!(f, " obsolete")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icon_to_str() {
        assert_eq!(
            CachedIcon(icon!("nf-test", 0xf500)).to_string(),
            "nf-test f500",
        );
        assert_eq!(
            CachedIcon(icon!("nf-test", 0xf500, true)).to_string(),
            "nf-test f500 obsolete",
        );
    }

    #[test]
    fn icon_from_str() {
        assert_eq!(
            icon!("nf-test", 0xf500),
            "nf-test f500".parse::<CachedIcon>().unwrap().0,
        );
        assert_eq!(
            icon!("nf-test", 0xf500, true),
            "nf-test f500 obsolete".parse::<CachedIcon>().unwrap().0,
        );
    }
}
