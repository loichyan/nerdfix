//! Nerd font icons infomation.

use crate::error;
use serde::{de::Visitor, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use thisctx::IntoError;

pub(crate) fn parse_codepoint(s: &str) -> error::Result<char> {
    let v = u32::from_str_radix(s, 16).map_err(|_| error::InvalidCodepoint.build())?;
    char::from_u32(v).ok_or_else(|| error::InvalidCodepoint.build())
}

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

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Indices {
    // Reserved for future compatibility
    pub metadata: Version,
    pub icons: Vec<Icon>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Version {
    V1,
}

impl<'de> Deserialize<'de> for Indices {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
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
                let mut metadata = None::<Version>;
                let mut icons = Vec::default();
                while let Some(k) = map.next_key::<&str>()? {
                    if k == "METADATA" {
                        metadata = Some(map.next_value()?);
                    } else {
                        let info = map.next_value::<IconInfo>()?;
                        icons.push(Icon {
                            name: k.to_owned(),
                            codepoint: info.codepoint,
                            obsolete: info.obsolete,
                        });
                    }
                }
                Ok(Indices {
                    metadata: metadata
                        .ok_or_else(|| serde::de::Error::missing_field("METADATA"))?,
                    icons,
                })
            }
        }
        deserializer.deserialize_map(DbVisitor)
    }
}

impl Serialize for Indices {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.icons.len() + 1))?;
        map.serialize_entry("METADATA", &self.metadata)?;
        for i in self.icons.iter() {
            map.serialize_entry(
                &i.name,
                &IconInfo {
                    codepoint: i.codepoint,
                    obsolete: i.obsolete,
                },
            )?;
        }
        map.end()
    }
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

    pub(crate) fn serialize<S>(this: &char, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&format!("{:x}", *this as u32))
    }
}
