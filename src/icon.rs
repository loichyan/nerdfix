//! Nerd font icons infomation.

use serde::{de::Visitor, Deserialize, Deserializer, Serialize, Serializer};

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Serialize)]
pub struct Icon {
    pub name: String,
    #[serde(deserialize_with = "codepoint_de")]
    #[serde(serialize_with = "codepoint_se")]
    pub codepoint: char,
    #[serde(default)]
    pub obsolete: bool,
}

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

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Serialize)]
pub struct Cache {
    // Reserved for future compatibility
    pub version: Version,
    pub icons: Vec<Icon>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Version {
    V1,
}

fn codepoint_de<'de, D>(deserializer: D) -> Result<char, D::Error>
where
    D: Deserializer<'de>,
{
    struct CodepointVisitor;
    impl<'de> Visitor<'de> for CodepointVisitor {
        type Value = char;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            let v = u32::from_str_radix(v, 16).map_err(|_| E::custom("Invalid hex number"))?;
            char::from_u32(v).ok_or_else(|| E::custom("Invalid UTF-8 character"))
        }
    }
    deserializer.deserialize_str(CodepointVisitor)
}

fn codepoint_se<S>(t: &char, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&format!("{:x}", *t as u32))
}
