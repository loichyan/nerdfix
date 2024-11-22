//! Nerd font icons information.

use std::fmt;

use serde::de::Visitor;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thisctx::IntoError;

use crate::error;

pub(crate) fn parse_codepoint(s: &str) -> error::Result<char> {
    let v = u32::from_str_radix(s, 16).map_err(|_| error::InvalidCodepoint.build())?;
    char::from_u32(v).ok_or_else(|| error::InvalidCodepoint.build())
}

pub(crate) fn display_codepoint(ch: char) -> String {
    format!("{:x}", ch as u32)
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct Database {
    #[serde(default)]
    pub icons: Indices,
    #[serde(default)]
    pub substitutions: Substitutions,
}

pub type Indices = Vec<Icon>;
pub type Substitutions = Vec<Substitution>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Icon {
    pub name: String,
    #[serde(with = "codepoint")]
    pub codepoint: char,
    #[serde(default)]
    pub obsolete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Substitution {
    pub ty: SubstitutionType,
    pub from: String,
    pub to: String,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SubstitutionType {
    #[default]
    Exact,
    Prefix,
    Codepoint,
}

mod codepoint {
    use super::*;

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<char, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CodepointVisitor;
        impl<'de> Visitor<'de> for CodepointVisitor {
            type Value = char;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("codepoint")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                parse_codepoint(v).map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(CodepointVisitor)
    }

    pub(crate) fn serialize<S>(codepoint: &char, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&display_codepoint(*codepoint))
    }
}

mod substitution {
    use std::str::FromStr;

    use super::*;

    impl FromStr for SubstitutionType {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(match s {
                "exact" => SubstitutionType::Exact,
                "prefix" => SubstitutionType::Prefix,
                "codepoint" => SubstitutionType::Codepoint,
                _ => return Err("unknown substitution type"),
            })
        }
    }

    impl FromStr for Substitution {
        type Err = &'static str;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let mut cur = s.chars();
            let mut ty = None::<SubstitutionType>;
            let mut from = None::<&str>;
            let mut start = 0usize;
            while let Some(ch) = cur.next() {
                match ch {
                    ':' if ty.is_none() => {
                        start = s.len() - cur.as_str().len();
                        ty = s[..(start - 1)].parse().map(Some)?;
                    }
                    '/' => {
                        let i = start;
                        start = s.len() - cur.as_str().len();
                        from = Some(&s[i..(start - 1)]);
                    }
                    _ => {}
                }
            }
            let to = &s[start..];
            tri!({
                let from = from?;
                if from.is_empty() || to.is_empty() {
                    None
                } else {
                    Some(Self {
                        ty: ty.unwrap_or_default(),
                        from: from.into(),
                        to: to.into(),
                    })
                }
            })
            .ok_or("invalid substitution syntax")
        }
    }

    impl fmt::Display for SubstitutionType {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(match self {
                Self::Exact => "exact",
                Self::Prefix => "prefix",
                Self::Codepoint => "codepoint",
            })
        }
    }

    impl fmt::Display for Substitution {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}:{}/{}", self.ty, self.from, self.to)
        }
    }

    impl<'de> Deserialize<'de> for Substitution {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            struct SubVisitor;
            impl<'de> Visitor<'de> for SubVisitor {
                type Value = Substitution;

                fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str("substitution")
                }

                fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
                where
                    E: serde::de::Error,
                {
                    v.parse().map_err(E::custom)
                }
            }
            deserializer.deserialize_str(SubVisitor)
        }
    }

    impl Serialize for Substitution {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            serializer.serialize_str(&self.to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_codepoint() {
        assert_eq!(super::parse_codepoint("f001").unwrap(), '\u{f001}');
    }

    #[test]
    fn display_codepoint() {
        assert_eq!(super::display_codepoint('\u{f001}'), "f001");
    }

    #[test]
    fn parse_substutition() {
        let expected = Substitution {
            ty: SubstitutionType::Exact,
            from: "abc".into(),
            to: "def".into(),
        };
        assert_eq!("abc/def".parse::<Substitution>().unwrap(), expected);
        assert_eq!("exact:abc/def".parse::<Substitution>().unwrap(), expected);
        let expected = Substitution {
            ty: SubstitutionType::Prefix,
            from: "abc".into(),
            to: "def".into(),
        };
        assert_eq!("prefix:abc/def".parse::<Substitution>().unwrap(), expected);
        assert!("exact".parse::<Substitution>().is_err());
        assert!("exact:".parse::<Substitution>().is_err());
        assert!("exact:/".parse::<Substitution>().is_err());
        assert!("exact:abc/".parse::<Substitution>().is_err());
        assert!("ezact:abc/def".parse::<Substitution>().is_err());
    }

    #[test]
    fn display_substitution() {
        assert_eq!(
            Substitution {
                ty: SubstitutionType::Exact,
                from: "abc".into(),
                to: "def".into(),
            }
            .to_string(),
            "exact:abc/def",
        );
        assert_eq!(
            Substitution {
                ty: SubstitutionType::Prefix,
                from: "abc".into(),
                to: "def".into(),
            }
            .to_string(),
            "prefix:abc/def",
        );
    }
}
