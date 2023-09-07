//! Nerd font icons infomation.

use crate::error;
use serde::{de::Visitor, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use thisctx::IntoError;

pub(crate) fn parse_codepoint(s: &str) -> error::Result<char> {
    let v = u32::from_str_radix(s, 16).map_err(|_| error::InvalidCodepoint.build())?;
    char::from_u32(v).ok_or_else(|| error::InvalidCodepoint.build())
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Input {
    #[serde(with = "index")]
    Index(Indices),
    #[serde(with = "substitution")]
    Substitution(Substitutions),
}

pub type Indices = Vec<Icon>;
pub type Substitutions = Vec<Substitution>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Icon {
    pub name: String,
    pub codepoint: char,
    pub obsolete: bool,
}

/// A helper type to deserialize/serialize [`Icon`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct IconInfo {
    #[serde(with = "codepoint")]
    codepoint: char,
    #[serde(default)]
    obsolete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Substitution {
    pub from: String,
    pub to: Vec<String>,
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
        serializer.serialize_str(&format!("{:x}", *codepoint as u32))
    }
}

mod index {
    use super::*;

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Indices, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DbVisitor;
        impl<'de> Visitor<'de> for DbVisitor {
            type Value = Indices;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Indices")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut indices = Indices::default();
                while let Some(name) = map.next_key::<String>()? {
                    let info = map.next_value::<IconInfo>()?;
                    indices.push(Icon {
                        name,
                        codepoint: info.codepoint,
                        obsolete: info.obsolete,
                    });
                }
                Ok(indices)
            }
        }
        deserializer.deserialize_map(DbVisitor)
    }

    pub(super) fn serialize<S>(indices: &Indices, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(indices.len()))?;
        for icon in indices.iter() {
            map.serialize_entry(
                &icon.name,
                &IconInfo {
                    codepoint: icon.codepoint,
                    obsolete: icon.obsolete,
                },
            )?;
        }
        map.end()
    }
}

mod substitution {
    use super::*;

    pub(super) fn deserialize<'de, D>(deserializer: D) -> Result<Substitutions, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DbVisitor;
        impl<'de> Visitor<'de> for DbVisitor {
            type Value = Substitutions;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Indices")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut subs = Substitutions::default();
                while let Some(from) = map.next_key::<String>()? {
                    subs.push(Substitution {
                        from,
                        to: map.next_value()?,
                    });
                }
                Ok(subs)
            }
        }
        deserializer.deserialize_map(DbVisitor)
    }

    pub(super) fn serialize<S>(subs: &Substitutions, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(subs.len()))?;
        for sub in subs.iter() {
            map.serialize_entry(&sub.from, &sub.to)?;
        }
        map.end()
    }
}
