use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

/// In rare cases, the XML and JSON representations can contain numbers inside quotes (e.g. `"32"`
/// instead of `32`). To fix this, we try to parse all numbers using this special struct
/// with a dedicated serialization methods.
#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub(crate) struct QuoteNum(u32);

impl From<u32> for QuoteNum {
    fn from(value: u32) -> Self {
        QuoteNum(value)
    }
}

impl From<QuoteNum> for u32 {
    fn from(value: QuoteNum) -> Self {
        value.0
    }
}

impl Serialize for QuoteNum {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        u32::serialize(&self.0, serializer)
    }
}

impl<'de> Deserialize<'de> for QuoteNum {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: String = String::deserialize(deserializer)?;
        let trimmed = s.trim_matches('"');
        let value = u32::from_str(trimmed).map_err(serde::de::Error::custom)?;
        Ok(Self(value))
    }
}
