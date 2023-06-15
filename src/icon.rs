//! Nerd font icons infomation.

use crate::error;
use derive_more::Display;
use serde::{de::Visitor, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use std::{fmt, str::FromStr};
use thisctx::IntoError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Icon {
    pub name: String,
    pub codepoint: char,
    pub obsolete: bool,
}

// TODO: remove unused trait impl
impl PartialOrd for Icon {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Ord for Icon {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.name.cmp(&other.name)
    }
}

/// A helper type to deserialize/serialize [`Icon`].
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct IconInfo {
    #[serde(deserialize_with = "Codepoint::de", serialize_with = "Codepoint::se")]
    codepoint: char,
    #[serde(default)]
    obsolete: bool,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Cache {
    // Reserved for future compatibility
    pub metadata: Version,
    pub icons: Vec<Icon>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Version {
    V1,
}

impl<'de> Deserialize<'de> for Cache {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CacheVisitor;
        impl<'de> Visitor<'de> for CacheVisitor {
            type Value = Cache;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("Cache")
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
                Ok(Cache {
                    metadata: metadata
                        .ok_or_else(|| serde::de::Error::missing_field("METADATA"))?,
                    icons,
                })
            }
        }
        deserializer.deserialize_map(CacheVisitor)
    }
}

impl Serialize for Cache {
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

// A helper type to deserialize/serialize [`Icon::codepoint`].
#[derive(Debug, Display)]
#[display(fmt = "{:x}", "u32::from(*_0)")]
pub(crate) struct Codepoint(pub char);

impl FromStr for Codepoint {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let v = u32::from_str_radix(s, 16).map_err(|_| error::InvalidCodepoint.build())?;
        char::from_u32(v)
            .map(Self)
            .ok_or_else(|| error::InvalidCheatSheet(0).build())
    }
}

impl Codepoint {
    pub fn de<'de, D>(deserializer: D) -> Result<char, D::Error>
    where
        D: Deserializer<'de>,
    {
        Self::deserialize(deserializer).map(|t| t.0)
    }

    fn se<S>(t: &char, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Self(*t).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Codepoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CodepointVisitor;
        impl<'de> Visitor<'de> for CodepointVisitor {
            type Value = Codepoint;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("codepoint")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                v.parse().map_err(serde::de::Error::custom)
            }
        }
        deserializer.deserialize_str(CodepointVisitor)
    }
}

impl Serialize for Codepoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
